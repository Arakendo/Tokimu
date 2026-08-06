//! Plain-text local adapter for the Observation Shell corpus.
//!
//! This binary intentionally avoids terminal-specific behavior. Standard input
//! and piped scripts both call `CliSession::execute_line`, so they exercise the
//! same shell invocation and projection path.

use std::io::{self, BufRead, Write};

mod session;

use session::ShellFixture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CliTermination {
    EndOfFile,
    ExitCommand,
    InterruptedInput,
}

fn run_adapter<R: BufRead, W: Write>(
    reader: R,
    stdout: &mut W,
    session: &mut ShellFixture,
) -> io::Result<CliTermination> {
    writeln!(
        stdout,
        "Tokimu Observation Shell CLI\nplain-text adapter; type HELP or send one command per line; EOF exits"
    )?;

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                writeln!(stdout, "input interrupted; session closed")?;
                stdout.flush()?;
                return Ok(CliTermination::InterruptedInput);
            }
            Err(error) => return Err(error),
        };
        if matches!(line.trim().to_ascii_lowercase().as_str(), "exit" | "quit") {
            writeln!(stdout, "session closed")?;
            stdout.flush()?;
            return Ok(CliTermination::ExitCommand);
        }

        if let Some(projection) = session.execute_line(&line) {
            writeln!(stdout, "> {line}\n{projection}\n")?;
        }
        stdout.flush()?;
    }

    writeln!(stdout, "input closed; session closed")?;
    stdout.flush()?;
    Ok(CliTermination::EndOfFile)
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let mut session = ShellFixture::new();
    let _termination = run_adapter(stdin.lock(), &mut stdout, &mut session)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{run_adapter, CliTermination, ShellFixture};

    #[test]
    fn piped_and_line_oriented_input_share_the_same_shell_path() {
        let script = ["help", "inspect world", "list entities"];
        let mut piped = ShellFixture::new();
        let piped_output = script
            .iter()
            .filter_map(|line| piped.execute_line(line))
            .collect::<Vec<_>>();

        let mut interactive = ShellFixture::new();
        let interactive_output = script
            .iter()
            .map(|line| {
                interactive
                    .execute_line(line)
                    .expect("known command projects")
            })
            .collect::<Vec<_>>();

        assert_eq!(piped_output, interactive_output);
        assert!(piped_output[1].contains("world"));
    }

    #[test]
    fn blank_lines_do_not_create_shell_history() {
        let mut session = ShellFixture::new();
        assert_eq!(session.execute_line("  \t"), None);
        assert!(session.shell.history().is_empty());
    }

    #[test]
    fn eof_terminates_only_the_plain_text_adapter() {
        let mut session = ShellFixture::new();
        let mut output = Vec::new();

        let termination = run_adapter(Cursor::new("inspect world\n"), &mut output, &mut session)
            .expect("bounded scripted input should execute");

        assert_eq!(termination, CliTermination::EndOfFile);
        let output = String::from_utf8(output).expect("adapter output is UTF-8");
        assert!(output.contains("input closed; session closed"));
        assert_eq!(session.shell.history().len(), 1);
    }

    #[test]
    fn exit_command_stops_the_adapter_without_executing_following_input() {
        let mut session = ShellFixture::new();
        let mut output = Vec::new();

        let termination = run_adapter(
            Cursor::new("help\nexit\ninspect world\n"),
            &mut output,
            &mut session,
        )
        .expect("bounded scripted input should execute");

        assert_eq!(termination, CliTermination::ExitCommand);
        assert_eq!(session.shell.history().len(), 1);
    }
}
