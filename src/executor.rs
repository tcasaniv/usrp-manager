use console::style;
use std::process::{Command, Stdio};

pub struct CmdResult {
    pub success: bool,
    #[allow(dead_code)]
    pub exit_code: i32,
    pub output: String,
}

/// Run a command, print it and its output visibly (like the reference screenshot).
pub fn run_visible(program: &str, args: &[&str]) -> CmdResult {
    // Print command line in yellow
    let cmd_display = format!("{} {}", program, args.join(" "));
    println!();
    println!("{} {}", style("●").yellow().bold(), style(&cmd_display).yellow());

    let result = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = if stderr.is_empty() {
                stdout.clone()
            } else if stdout.is_empty() {
                stderr.clone()
            } else {
                format!("{}\n{}", stdout, stderr)
            };

            let exit_code = output.status.code().unwrap_or(-1);
            println!(
                "  {} Command exited with code {}.",
                style("└").dim(),
                if output.status.success() {
                    style(exit_code.to_string()).green()
                } else {
                    style(exit_code.to_string()).red()
                }
            );

            if !combined.trim().is_empty() {
                println!("  {} Command output:", style("└").dim());
                for line in combined.lines() {
                    println!("    {}", line);
                }
            }
            println!();

            CmdResult {
                success: output.status.success(),
                exit_code,
                output: combined,
            }
        }
        Err(e) => {
            println!(
                "  {} Failed to execute: {}",
                style("└").dim(),
                style(e.to_string()).red()
            );
            println!();
            CmdResult {
                success: false,
                exit_code: -1,
                output: e.to_string(),
            }
        }
    }
}

/// Run a command silently (no output printed), returns stdout or None on failure.
pub fn run_silent(program: &str, args: &[&str]) -> Option<String> {
    let result = Command::new(program).args(args).output().ok()?;
    if result.status.success() {
        Some(String::from_utf8_lossy(&result.stdout).trim().to_string())
    } else {
        // Try stderr as well
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        if !stderr.is_empty() {
            Some(stderr)
        } else {
            None
        }
    }
}

/// Run a command silently, return output regardless of exit code.
#[allow(dead_code)]
pub fn run_silent_any(program: &str, args: &[&str]) -> Option<String> {
    let result = Command::new(program).args(args).output().ok()?;
    let out = String::from_utf8_lossy(&result.stdout).trim().to_string();
    let err = String::from_utf8_lossy(&result.stderr).trim().to_string();
    let combined = if !out.is_empty() { out } else { err };
    if combined.is_empty() { None } else { Some(combined) }
}
