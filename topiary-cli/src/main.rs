mod cli;
mod error;
mod fs;
mod io;
mod language;
mod visualisation;

use std::{
    io::{BufReader, BufWriter, Write},
    process::ExitCode,
};

use error::Benign;
use rootcause::{markers::Local, prelude::ResultExt};
use tabled::{Table, settings::Style};
use topiary_config::source::Source;
use topiary_core::{Operation, SpanAttachment, check_query_coverage, formatter};

use crate::{
    cli::Commands,
    error::{CLIResult, exit_code, print_error},
    io::{Inputs, OutputFile, process_inputs, read_input},
};

use miette::{NamedSource, Report};

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        if !e.benign() {
            log::error!("{e:?}");
        }
        return exit_code(e);
    }

    ExitCode::SUCCESS
}

async fn run() -> CLIResult<(), Local> {
    let args = cli::get_args()?;

    let file_config = &args.global.configuration;
    let (config, nickel_config) =
        topiary_config::Configuration::fetch(args.global.merge_configuration, file_config)
            .local_context_to()?;

    // Delegate by subcommand
    match args.command {
        Commands::Format {
            tolerate_parsing_errors,
            skip_idempotence,
            inputs,
        } => {
            let inputs = Inputs::new(&config, &inputs);

            process_inputs(inputs, move |input, language| {
                let output = OutputFile::try_from(&input)?;

                log::info!(
                    "Formatting {}, as {} using {}, to {}",
                    input.source(),
                    input.language().name,
                    input.query(),
                    output
                );

                let mut buf_output = BufWriter::new(output);

                {
                    // NOTE This newly opened scope is important! `buf_input` takes
                    // ownership of `input`, which -- upon reading -- contains an
                    // open file handle. We need to close this file, by dropping
                    // `buf_input`, before we attempt to persist our output.
                    // Otherwise, we get an exclusive lock problem on Windows.
                    let mut buf_input = BufReader::new(input);

                    formatter(
                        &mut buf_input,
                        &mut buf_output,
                        &language,
                        Operation::Format {
                            skip_idempotence,
                            tolerate_parsing_errors,
                        },
                    )
                    .context_to()?;
                }

                buf_output.into_inner().context_to()?.persist()?;

                CLIResult::Ok(())
            })
            .await?;
        }

        Commands::CheckGrammar { inputs } => {
            let inputs = Inputs::new(&config, &inputs);

            process_inputs(inputs, |mut input, language| {
                let input_content = read_input(&mut input).context_to()?;
                log::debug!(
                    "Checking {}, as {} for grammar correctness",
                    input.source(),
                    input.language().name,
                );

                topiary_core::parse(&input_content, &language.grammar, false).context_to()?;

                Ok(())
            })
            .await?;
        }

        Commands::Visualise { format, input } => {
            // We are guaranteed (by clap) to have exactly one input, so it's safe to unwrap
            let input = Inputs::new(&config, &input).next().unwrap()?;
            let output = OutputFile::Stdout;

            // We don't need a `LanguageDefinitionCache` when there's only one input,
            // which saves us the thread-safety overhead
            let language = input.to_language().await?;

            log::info!(
                "Visualising {}, as {}, to {}",
                input.source(),
                input.language().name,
                output
            );

            let mut buf_input = BufReader::new(input);
            let mut buf_output = BufWriter::new(output);

            formatter(
                &mut buf_input,
                &mut buf_output,
                &language,
                Operation::Visualise {
                    output_format: format.into(),
                },
            )
            .map_err(|e| {
                if let Some(filepath) = buf_input.get_ref().source().location().to_path() {
                    return e.attach_filepath(filepath);
                }
                e
            })
            .context_to()?;
        }

        Commands::Config {
            command: Some(cli::ConfigCommand::ShowSources),
        } => {
            let bool_emoji = |b: bool| {
                match b {
                    true => "\u{2705}",  // Check Mark
                    false => "\u{274C}", // Cross Mark
                }
            };
            let sources = Source::config_sources(file_config)
                .map(|(hint, source)| {
                    let languages_exists = bool_emoji(source.languages_exists());
                    let queries_exists =
                    // Should Source::Builtin always return true for queries?
                        bool_emoji(source.queries_dir().map(|p| p.exists()).unwrap_or(true));
                    (hint, format!("{source}"), languages_exists, queries_exists)
                })
                .collect::<Vec<_>>();

            let mut table = Table::builder(sources);
            table.remove_record(0);
            table.insert_record(0, ["source", "path", "languages.ncl", "queries"]);
            println!("{}", table.build().with(Style::modern_rounded()));
        }

        Commands::Config { command: None } => {
            // Output the collated nickel configuration.
            // Don't fail on error but merely log the event since the original `nickel_config` is
            // already valid.
            #[cfg(feature = "nickel")]
            if let Err(e) = io::format_config(&config, &nickel_config).await {
                log::error!("Config formatting error: {}", e);
            } else {
                return Ok(());
            }
            println!("{nickel_config}");
        }

        Commands::Prefetch { force, language } => match language {
            Some(l) => config.prefetch_language(l, force).local_context_to()?,
            _ => config.prefetch_languages(force).local_context_to()?,
        },

        Commands::Coverage { input } => {
            // We are guaranteed (by clap) to have exactly one input, so it's safe to unwrap
            let input = Inputs::new(&config, &input).next().unwrap()?;
            let output = OutputFile::Stdout;

            // We don't need a `LanguageDefinitionCache` when there's only one input,
            // which saves us the thread-safety overhead
            let language = input.to_language().await?;

            log::info!(
                "Checking query coverage of {}, as {}",
                input.source(),
                input.language().name,
            );

            let mut buf_input = BufReader::new(input);
            let mut buf_output = BufWriter::new(output);

            let input_content = read_input(&mut buf_input).context_to()?;

            let coverage_data =
                check_query_coverage(&input_content, &language.query, &language.grammar)
                    .map_err(|e| e.attach_source(&input_content))
                    .context_to()?;
            let coverage_res = coverage_data.get_result();

            let query_source = NamedSource::new(
                buf_input.get_ref().query.to_string(),
                language.query.query_content,
            )
            .with_language(&language.name);
            write!(
                &mut buf_output,
                "{:?}",
                Report::new(coverage_data).with_source_code(query_source)
            )
            .context_to()?;

            coverage_res.context_to()?;
        }

        Commands::Completion { shell } => {
            // The CLI parser fails if no shell is provided/detected, so it's safe to unwrap here
            cli::completion(shell.unwrap());
        }
    }

    Ok(())
}
