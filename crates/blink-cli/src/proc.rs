use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitStatus};

/// Run `command` through the platform shell in `cwd`, inheriting stdio so the
/// user sees the underlying tool's output live. Blink is a thin orchestrator
/// here — it never hides what actually ran.
pub fn run_shell(command: &str, cwd: &Path) -> std::io::Result<ExitStatus> {
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(command);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(command);
        c
    };
    cmd.current_dir(cwd).status()
}

/// Ask a yes/no question on the terminal, defaulting to "no". Returns `false`
/// on any read error or non-affirmative answer, so a piped/non-interactive
/// invocation never proceeds with a destructive action by accident.
pub fn confirm(prompt: &str) -> bool {
    print!("{prompt} [y/N] ");
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}
