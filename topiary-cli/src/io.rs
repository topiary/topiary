use std::{
    ffi::OsString,
    fmt::{self, Display},
    fs::File,
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use rootcause::{
    Report,
    markers::{ObjectMarkerFor, SendSync},
    prelude::ResultExt,
    report,
    report_collection::ReportCollection,
};
use rootcause_preformat::PreformatReportExt;
use tempfile::tempfile;
use topiary_core::{
    ErrorSpan, FormatterError, InjectionQuery, Language, SpanAttachment, TopiaryQuery,
};

use crate::cli::{AtLeastOneInput, ExactlyOneInput, FromStdin};
use crate::config::Configuration;
use crate::error::{CLIResult, ResultPreformat, TopiaryError};

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
    pub(crate) fn filepath(&self) -> Option<&Path> {
        match self {
            QuerySource::Path(p) => Some(p.as_path()),
            QuerySource::BuiltIn(_) => None,
        }
    }

    pub(crate) fn get_content_sync(&self) -> CLIResult<String> {
        let contents = match self {
            Self::Path(query) => std::fs::read_to_string(query)?,
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

    fn filepath(&self) -> Option<&Path> {
        match self {
            InputSource::Stdin => None,
            InputSource::Disk(path, _) => Some(path.as_ref()),
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

impl InputLocation {
    pub(crate) fn to_path(&self) -> Option<&Path> {
        self.0.as_ref().map(|p| p.as_path())
    }
}

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
    pub(crate) formatting_query: QuerySource,
    pub(crate) injection_query: Option<QuerySource>,
}

impl InputFile<'_> {
    /// Convert our `InputFile` into a language definition values with blocking I/O.
    pub fn to_language_sync(&self) -> CLIResult<Language> {
        let grammar = self.language().grammar()?;
        let query_contents = self.formatting_query.get_content_sync()?;
        let injection_query = match &self.injection_query {
            Some(source) => {
                let contents = source.get_content_sync()?;
                Some(InjectionQuery::new(&grammar, &contents).attach_filepath(source.filepath())?)
            }
            None => None,
        };
        let formatting_query = TopiaryQuery::new(&grammar, &query_contents)
            .attach_filepath(self.formatting_query.filepath())
            .context(FormatterError::Parsing)?;

        Ok(Language {
            name: self.language.name.clone(),
            formatting_query,
            injection_query,
            grammar,
            indent: self.language().indent(),
        })
    }

    /// Expose input source
    pub fn source(&self) -> &InputSource {
        &self.source
    }

    pub(crate) fn filepath(&self) -> Option<&Path> {
        self.source().filepath()
    }

    /// Expose language for input
    pub fn language(&self) -> &topiary_config::language::Language {
        self.language
    }

    /// Expose formatting query path for input
    pub fn formatting_query(&self) -> &QuerySource {
        &self.formatting_query
    }

    /// Expose optional injection query path for input
    pub fn injection_query(&self) -> Option<&QuerySource> {
        self.injection_query.as_ref()
    }
}

/// Simple helper function to read the full content of an io Read stream
pub(crate) fn read_input(input: &mut dyn io::Read) -> CLIResult<String> {
    let mut content = String::new();
    input.read_to_string(&mut content)?;
    Ok(content)
}

impl Read for InputFile<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
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
pub struct Inputs<'cfg>(Vec<CLIResult<InputFile<'cfg>>>);

impl<'cfg, 'i> Inputs<'cfg> {
    pub fn new<T>(config: &'cfg Configuration, inputs: &'i T) -> Self
    where
        &'i T: Into<InputFrom>,
    {
        let inputs = match inputs.into() {
            InputFrom::Stdin(language_name, query) => {
                vec![(|| {
                    let language = config
                        .get_language_cfg(&language_name)
                        .map_err(|e| report!(e).preformat())
                        .context(TopiaryError::Config)?;
                    let query_source: QuerySource = match query {
                        // The user specified a query file
                        Some(p) => p,
                        // The user did not specify a file, try the default locations
                        None => config
                            .get_query_source(&language_name, topiary_queries::FORMATTING_QUERY)?,
                    };
                    let injection_query = config
                        .get_query_source(&language_name, topiary_queries::INJECTIONS_QUERY)
                        .ok();
                    Ok(InputFile {
                        source: InputSource::Stdin,
                        language,
                        formatting_query: query_source,
                        injection_query,
                    })
                })()]
            }
            InputFrom::Files(files) => files
                .into_iter()
                .map(|path| {
                    let language = config.detect(&path).preformat_context()?;
                    let language_name = language.name.clone();
                    let query: QuerySource = config
                        .get_query_source(&language_name, topiary_queries::FORMATTING_QUERY)?;
                    let injection_query = config
                        .get_query_source(&language_name, topiary_queries::INJECTIONS_QUERY)
                        .ok();

                    Ok(InputFile {
                        source: InputSource::Disk(path.into(), None),
                        language,
                        formatting_query: query,
                        injection_query,
                    })
                })
                .collect(),
        };

        Self(inputs)
    }
}

impl<'cfg> Iterator for Inputs<'cfg> {
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
    pub fn new(path: &str) -> CLIResult<Self> {
        match path {
            "-" => Ok(Self::Stdout),
            file => Ok(Self::Disk {
                staged: tempfile().context(TopiaryError::Config)?,
                output: file.into(),
            }),
        }
    }

    // This function must be called to persist the output to disk
    pub fn persist(self) -> CLIResult<()> {
        if let Self::Disk { mut staged, output } = self {
            // Rewind to the beginning of the staged output
            staged.flush()?;
            staged.rewind()?;

            // Open the actual output for writing and copy the staged contents
            let mut writer = File::create(&output)?;
            let bytes = io::copy(&mut staged, &mut writer)?;

            log::debug!("Wrote {bytes} bytes to {}", output.display());
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
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdout => io::stdout().lock().write(buf),
            Self::Disk { staged, .. } => staged.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
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
    type Error = Report;

    fn try_from(input: &InputFile) -> CLIResult<Self> {
        match &input.source {
            InputSource::Stdin => Ok(Self::Stdout),
            InputSource::Disk(path, _) => Self::new(path.to_string_lossy().as_ref()),
        }
    }
}

// meant to be used in scenarios where multiple inputs are possible
pub(crate) async fn process_inputs<F>(
    inputs: Inputs<'_>,
    process_fn: F,
    config: Arc<crate::config::Configuration>,
) -> CLIResult<()>
where
    F: Fn(InputFile, Arc<Language>, Arc<Configuration>) -> Result<(), Report>
        + Send
        + Sync
        + 'static,
    ErrorSpan: ObjectMarkerFor<SendSync>,
{
    let (_, mut results) = async_scoped::TokioScope::scope_and_block(|scope| {
        for input in inputs {
            let process_fn = &process_fn;
            let config = config.clone();
            scope.spawn(async move {
                // This happens when the input resolver cannot establish an input
                // source, language or query file.
                let input = input?;
                let location = input.source().location();
                tokio::task::block_in_place(|| {
                    let language_name = input.language().name.clone();
                    let language = Arc::new(
                        config
                            .get_language(&language_name)
                            .attach_filepath(location.to_path())?,
                    );
                    process_fn(input, language, config)
                        .map_err(|e| e.attach_filepath(location.to_path()))
                })
            });
        }
    });

    if results.len() == 1 {
        // If we just had one input, then handle errors as normal
        return results.swap_remove(0)?;
    }

    // use `.count()` here to ensure eager evaluation of iterator
    let errs: ReportCollection = results
        .into_iter()
        .filter_map(|r| r.map_err(|e| report!(e).into_dynamic()).flatten().err())
        .collect();

    if !errs.is_empty() {
        return Err(report!(errs).into_dynamic());
    }
    Ok(())
}
