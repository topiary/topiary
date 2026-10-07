//! Builds the runner that `topiary-core` uses to delegate a language to a
//! configured external formatter.
//!
//! Core decides *when* to delegate -- and keeps injection handling around it --
//! while this module knows *how*: it spawns the configured command and pipes
//! the input through it. Keeping process spawning here means `topiary-core`
//! stays independent of the host platform.

use std::process::{Command, Stdio};

use rootcause::report;
use topiary_config::language::Command as ConfiguredCommand;
use topiary_core::{ExternalFormatter, FormatterError, FormatterResult};

/// Build an [`ExternalFormatter`] that pipes the input through `command`.
pub(crate) fn runner(command: &ConfiguredCommand) -> ExternalFormatter {
    let program = command.command.clone();
    let args = command.args.clone();

    ExternalFormatter::new(move |input| run(&program, &args, input))
}

/// Run `program` with `args`, writing `input` to its standard input and
/// returning what it writes to standard output.
fn run(program: &str, args: &[String], input: &str) -> FormatterResult<String> {
    use std::io::Write;

    let failed = |message: String| {
        report!(FormatterError::ExternalFormatter {
            command: program.to_owned(),
            message,
        })
    };

    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| failed(format!("could not be started: {err}")))?;

    // Write the input on a separate thread. Otherwise a formatter that streams
    // its output could fill the stdout pipe and block before it has finished
    // reading stdin, deadlocking us.
    let mut stdin = child.stdin.take().expect("stdin was piped");
    let owned_input = input.to_owned();
    let writer = std::thread::spawn(move || -> std::io::Result<()> {
        stdin.write_all(owned_input.as_bytes())?;
        stdin.flush()
    });

    let output = child
        .wait_with_output()
        .map_err(|err| failed(format!("could not be waited on: {err}")))?;

    let write_result = writer
        .join()
        .map_err(|_| failed("panicked while writing to its standard input".to_owned()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        let message = if stderr.is_empty() {
            format!("exited with {}", output.status)
        } else {
            format!("exited with {}: {stderr}", output.status)
        };
        return Err(failed(message));
    }

    // A broken pipe is expected when the program bails out early, so only
    // surface a write error once we know the program itself succeeded.
    write_result.map_err(|err| failed(format!("could not be written to: {err}")))?;

    String::from_utf8(output.stdout).map_err(|err| failed(format!("produced invalid UTF-8: {err}")))
}
