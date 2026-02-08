use std::{
    ffi::OsString,
    fmt::{self, Display},
    fs::File,
    io::{self, BufWriter, Read, Result, Seek, Write},
    path::PathBuf,
    sync::Arc,
};

use nickel_lang_core::eval::value::NickelValue;
use tempfile::tempfile;
use topiary_config::Configuration;
use topiary_core::{Language, Operation, TopiaryQuery, formatter};

use crate::{
    cli::{AtLeastOneInput, ExactlyOneInput, FromStdin},
    error::{CLIError, CLIResult, TopiaryError, print_error},
    language::LanguageDefinitionCache,
};

#[derive(Debug, Clone, Hash)]
pub enum QuerySource {
    Path(PathBuf),
    BuiltIn(String),
}

impl From<PathBuf> for QuerySource {
    fn from(path: PathBuf) -> Self {
        QuerySource::Path(path)
    }
}

impl From<&PathBuf> for QuerySource {
    fn from(path: &PathBuf) -> Self {
        QuerySource::Path(path.clone())
    }
}

impl From<&str> for QuerySource {
    fn from(string: &str) -> Self {
        QuerySource::BuiltIn(String::from(string))
    }
}

impl Display for QuerySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuerySource::Path(p) => write!(f, "{}", p.display()),
            QuerySource::BuiltIn(_) => write!(f, "built-in query"),
        }
    }
}

impl QuerySource {
    async fn get_content(&self) -> CLIResult<String> {
        let contents = match self {
            Self::Path(query) => tokio::fs::read_to_string(query).await?,
            Self::BuiltIn(contents) => contents.to_owned(),
        };
        Ok(contents)
    }
}

/// Unified interface for input sources. We either have input from:
/// * Standard input, in which case we need to specify the language and, optionally, query override
/// * A sequence of files
///
/// These are captured by the CLI parser, with `cli::AtLeastOneInput` and `cli::ExactlyOneInput`.
/// We use this struct to normalise the interface for downstream (using `From` implementations).
pub enum InputFrom {
    Stdin(String, Option<QuerySource>),
    Files(Vec<PathBuf>),
}

impl From<&ExactlyOneInput> for InputFrom {
    fn from(input: &ExactlyOneInput) -> Self {
        match input {
            ExactlyOneInput {
                stdin: Some(FromStdin { language, query }),
                ..
            } => InputFrom::Stdin(language.to_owned(), query.as_ref().map(|p| p.into())),

            ExactlyOneInput {
                file: Some(path), ..
            } => InputFrom::Files(vec![path.to_owned()]),

            _ => unreachable!("Clap guarantees input is always one of the above"),
        }
    }
}

impl From<&AtLeastOneInput> for InputFrom {
    fn from(input: &AtLeastOneInput) -> Self {
        match input {
            AtLeastOneInput {
                stdin: Some(FromStdin { language, query }),
                ..
            } => InputFrom::Stdin(language.to_owned(), query.as_ref().map(|p| p.into())),

            AtLeastOneInput { files, .. } => InputFrom::Files(files.to_owned()),
        }
    }
}

/// Each `InputFile` needs to locate its source (standard input or disk), such that its `io::Read`
/// implementation can do the right thing.
#[derive(Debug)]
pub enum InputSource {
    Stdin,
    Disk(Arc<PathBuf>, Option<File>),
}

impl InputSource {
    pub fn location(&self) -> InputLocation {
        match self {
            InputSource::Stdin => InputLocation(None),
            InputSource::Disk(path, _) => InputLocation(Some(path.clone())),
        }
    }
}

impl fmt::Display for InputSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdin => write!(f, "standard input"),
            Self::Disk(path, _) => write!(f, "{}", path.display()),
        }
    }
}

/// A location for a given [InputSource], `None` represents standard input
#[derive(Debug)]
pub struct InputLocation(Option<Arc<PathBuf>>);

impl fmt::Display for InputLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => write!(f, "standard input"),
            Some(ref path) => write!(f, "{}", path.display()),
        }
    }
}

/// An `InputFile` is the unit of input for Topiary, encapsulating everything needed for downstream
/// processing. It implements `io::Read`, so it can be passed directly to the Topiary API.
#[derive(Debug)]
pub struct InputFile<'cfg> {
    source: InputSource,
    language: &'cfg topiary_config::language::Language,
    pub(crate) query: QuerySource,
}

impl InputFile<'_> {
    /// Convert our `InputFile` into language definition values that Topiary can consume
    #[allow(clippy::result_large_err)]
    pub async fn to_language(&self) -> CLIResult<Language> {
        let grammar = self.language().grammar()?;
        let query_contents = self.query.get_content().await?;
        let query = TopiaryQuery::new(&grammar, &query_contents)?;

        Ok(Language {
            name: self.language.name.clone(),
            query,
            grammar,
            indent: self.language().indent(),
        })
    }

    /// Expose input source
    pub fn source(&self) -> &InputSource {
        &self.source
    }

    /// Expose language for input
    pub fn language(&self) -> &topiary_config::language::Language {
        self.language
    }

    /// Expose query path for input
    pub fn query(&self) -> &QuerySource {
        &self.query
    }
}

pub(crate) async fn to_language_from_config<T: AsRef<str>>(
    config: &Configuration,
    name: T,
) -> CLIResult<Language> {
    let config_language = config.get_language(name.as_ref())?;
    let grammar = config_language.grammar()?;
    let query_content = to_query_from_language(config_language)?
        .get_content()
        .await?;
    let query = TopiaryQuery::new(&grammar, &query_content)?;

    Ok(Language {
        name: name.as_ref().to_string(),
        query,
        grammar,
        indent: config_language.indent(),
    })
}

/// Simple helper function to read the full content of an io Read stream
pub(crate) fn read_input(input: &mut dyn io::Read) -> Result<String> {
    let mut content = String::new();
    input.read_to_string(&mut content)?;
    Ok(content)
}

impl Read for InputFile<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match &mut self.source {
            InputSource::Stdin => io::stdin().lock().read(buf),

            InputSource::Disk(path, fd) => {
                if fd.is_none() {
                    *fd = Some(File::open(path.as_ref())?);
                }

                fd.as_mut().unwrap().read(buf)
            }
        }
    }
}

/// `Inputs` is an iterator of fully qualified `InputFile`s, each wrapped in `CLIResult`, which is
/// populated by its constructor from any type that implements `Into<InputFrom>`
#[allow(clippy::result_large_err)]
pub struct Inputs<'cfg>(Vec<CLIResult<InputFile<'cfg>>>);

impl<'cfg, 'i> Inputs<'cfg> {
    pub fn new<T>(config: &'cfg Configuration, inputs: &'i T) -> Self
    where
        &'i T: Into<InputFrom>,
    {
        let inputs = match inputs.into() {
            InputFrom::Stdin(language_name, query) => {
                vec![(|| {
                    let language = config.get_language(&language_name)?;
                    let query_source: QuerySource = match query {
                        // The user specified a query file
                        Some(p) => p,
                        // The user did not specify a file, try the default locations
                        None => to_query_from_language(language)?,
                    };

                    Ok(InputFile {
                        source: InputSource::Stdin,
                        language,
                        query: query_source,
                    })
                })()]
            }

            InputFrom::Files(files) => files
                .into_iter()
                .map(|path| {
                    let language = config.detect(&path)?;
                    let query: QuerySource = to_query_from_language(language)?;

                    Ok(InputFile {
                        source: InputSource::Disk(path.into(), None),
                        language,
                        query,
                    })
                })
                .collect(),
        };

        Self(inputs)
    }
}

#[allow(clippy::result_large_err)]
fn to_query_from_language(language: &topiary_config::language::Language) -> CLIResult<QuerySource> {
    let query: QuerySource = match language.find_query_file() {
        Ok(p) => p.into(),
        // For some reason, Topiary could not find any
        // matching file in a default location. As a final attempt, try the
        // builtin ones. Store the error, return that if we
        // fail to find anything, because the builtin error might be unexpected.
        Err(e) => {
            log::warn!(
                "No query files found in any of the expected locations. Falling back to compile-time included files."
            );
            to_query(&language.name).map_err(|_| e)?
        }
    };
    Ok(query)
}

impl<'cfg> Iterator for Inputs<'cfg> {
    #[allow(clippy::result_large_err)]
    type Item = CLIResult<InputFile<'cfg>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

/// An `OutputFile` is the unit of output for Topiary, differentiating between standard output and
/// disk (which uses temporary files to perform atomic updates in place). It implements
/// `io::Write`, so it can be passed directly to the Topiary API.
///
/// NOTE When writing to disk, the `persist` function must be called to perform the in place write.
#[derive(Debug)]
pub enum OutputFile {
    Stdout,
    Disk {
        // NOTE We stage to a file, rather than writing
        // to memory (e.g., Vec<u8>), to ensure atomicity
        staged: File,
        output: OsString,
    },
}

impl OutputFile {
    #[allow(clippy::result_large_err)]
    pub fn new(path: &str) -> CLIResult<Self> {
        match path {
            "-" => Ok(Self::Stdout),
            file => Ok(Self::Disk {
                staged: tempfile()?,
                output: file.into(),
            }),
        }
    }

    // This function must be called to persist the output to disk
    #[allow(clippy::result_large_err)]
    pub fn persist(self) -> CLIResult<()> {
        if let Self::Disk { mut staged, output } = self {
            // Rewind to the beginning of the staged output
            staged.flush()?;
            staged.rewind()?;

            // Open the actual output for writing and copy the staged contents
            let mut writer = File::create(&output)?;
            let bytes = io::copy(&mut staged, &mut writer)?;

            log::debug!("Wrote {bytes} bytes to {}", &output.display());
        }

        Ok(())
    }
}

impl fmt::Display for OutputFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdout => write!(f, "standard output"),
            Self::Disk { output, .. } => write!(f, "{}", output.display()),
        }
    }
}

impl Write for OutputFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            Self::Stdout => io::stdout().lock().write(buf),
            Self::Disk { staged, .. } => staged.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            Self::Stdout => io::stdout().lock().flush(),
            Self::Disk { staged, .. } => staged.flush(),
        }
    }
}

// Convenience conversion:
// * stdin maps to stdout
// * Files map to themselves (i.e., for in-place updates)
impl TryFrom<&InputFile<'_>> for OutputFile {
    type Error = TopiaryError;

    #[allow(clippy::result_large_err)]
    fn try_from(input: &InputFile) -> CLIResult<Self> {
        match &input.source {
            InputSource::Stdin => Ok(Self::Stdout),
            InputSource::Disk(path, _) => Self::new(path.to_string_lossy().as_ref()),
        }
    }
}

#[allow(clippy::result_large_err)]
fn to_query<T>(name: T) -> CLIResult<QuerySource>
where
    T: AsRef<str> + fmt::Display,
{
    match name.as_ref() {
        #[cfg(feature = "bash")]
        "bash" => Ok(topiary_queries::bash().into()),

        #[cfg(feature = "css")]
        "css" => Ok(topiary_queries::css().into()),

        #[cfg(feature = "json")]
        "json" => Ok(topiary_queries::json().into()),

        #[cfg(feature = "nickel")]
        "nickel" => Ok(topiary_queries::nickel().into()),

        #[cfg(feature = "ocaml")]
        "ocaml" => Ok(topiary_queries::ocaml().into()),

        #[cfg(feature = "ocaml_interface")]
        "ocaml_interface" => Ok(topiary_queries::ocaml_interface().into()),

        #[cfg(feature = "ocamllex")]
        "ocamllex" => Ok(topiary_queries::ocamllex().into()),

        #[cfg(feature = "openscad")]
        "openscad" => Ok(topiary_queries::openscad().into()),

        #[cfg(feature = "rust")]
        "rust" => Ok(topiary_queries::rust().into()),

        #[cfg(feature = "sdml")]
        "sdml" => Ok(topiary_queries::sdml().into()),

        #[cfg(feature = "toml")]
        "toml" => Ok(topiary_queries::toml().into()),

        #[cfg(feature = "tree_sitter_query")]
        "tree_sitter_query" => Ok(topiary_queries::tree_sitter_query().into()),

        #[cfg(feature = "wit")]
        "wit" => Ok(topiary_queries::wit().into()),

        name => Err(TopiaryError::Bin(
            format!("The specified language is unsupported: {name}"),
            Some(CLIError::UnsupportedLanguage(name.to_string())),
        )),
    }
}

// convenience function to bundle nickel config formatting errors in one return value
pub(crate) async fn format_config(
    config: &Configuration,
    nickel_term: &NickelValue,
) -> CLIResult<()> {
    let nickel_config = format!("{nickel_term}");
    let mut formatted_config = BufWriter::new(OutputFile::Stdout);
    // if errors are encountered in formatting, return
    let language = to_language_from_config(config, "nickel").await?;

    formatter(
        &mut nickel_config.as_bytes(),
        &mut formatted_config,
        &language,
        Operation::Format {
            skip_idempotence: true,
            tolerate_parsing_errors: false,
        },
    )?;

    Ok(())
}

// meant to be used in scenarios where multiple inputs are possible
pub(crate) async fn process_inputs<F>(inputs: Inputs<'_>, process_fn: F) -> CLIResult<()>
where
    F: Fn(InputFile, Arc<Language>) -> CLIResult<()> + Send + Sync + 'static,
{
    let cache = LanguageDefinitionCache::new();
    let (_, mut results) = async_scoped::TokioScope::scope_and_block(|scope| {
        for input in inputs {
            scope.spawn(async {
                // This happens when the input resolver cannot establish an input
                // source, language or query file.
                let input = input?;
                let location = input.source().location();
                let language = cache.fetch(&input).await?;
                process_fn(input, language).map_err(|e| {
                    let TopiaryError::Lib(fmt_err) = e else {
                        return e;
                    };
                    fmt_err.with_location(location.to_string()).into()
                })
            });
        }
    });

    if results.len() == 1 {
        // If we just had one input, then handle errors as normal
        return results.swap_remove(0)?;
    }

    // use `.count()` here to ensure eager evaluation of iterator
    let errs = results
        .into_iter()
        .filter_map(|r| r.map_err(TopiaryError::from).flatten().err())
        .inspect(|e| print_error(&e))
        .count();
    if errs > 0 {
        // For multiple inputs, bail out if any failed with a "multiple errors" failure
        return Err(TopiaryError::Bin(
            "Processing of some inputs failed; see warning logs for details".into(),
            Some(CLIError::Multiple),
        ));
    }
    Ok(())
}
