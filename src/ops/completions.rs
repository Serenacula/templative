mod bash;
mod fish;
mod powershell;
mod zsh;

use std::path::PathBuf;

use anyhow::{Context, Result};

#[derive(clap::ValueEnum, Clone)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
    #[value(name = "powershell")]
    PowerShell,
}

pub fn cmd_completions(shell: Shell, check: Option<PathBuf>) -> Result<()> {
    let (script, version) = match shell {
        Shell::Zsh        => (zsh::SCRIPT,        zsh::VERSION),
        Shell::Bash       => (bash::SCRIPT,        bash::VERSION),
        Shell::Fish       => (fish::SCRIPT,        fish::VERSION),
        Shell::PowerShell => (powershell::SCRIPT,  powershell::VERSION),
    };

    match check {
        None => print!("{}", script),
        Some(path) => {
            let contents = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            match parse_version(&contents) {
                None => anyhow::bail!(
                    "no version comment found in {}; unable to verify",
                    path.display()
                ),
                Some(installed) if installed < version => anyhow::bail!(
                    "completion script is outdated (installed: v{}, current: v{})\n\
                     re-run: templative completions {} > <path>",
                    installed, version,
                    shell_name(&shell),
                ),
                Some(installed) if installed > version => anyhow::bail!(
                    "completion script version v{} is newer than current v{} \
                     (installed from a newer version of templative?)",
                    installed, version,
                ),
                Some(installed) => {
                    println!("completion script is up to date (v{})", installed);
                }
            }
        }
    }
    Ok(())
}

fn parse_version(contents: &str) -> Option<u32> {
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("# templative-completions-version: ") {
            return rest.trim().parse().ok();
        }
    }
    None
}

fn shell_name(shell: &Shell) -> &'static str {
    match shell {
        Shell::Zsh        => "zsh",
        Shell::Bash       => "bash",
        Shell::Fish       => "fish",
        Shell::PowerShell => "powershell",
    }
}
