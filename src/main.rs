use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod errors;
mod fs_copy;
mod git;
mod ops;
mod registry;
mod utilities;

#[derive(Parser)]
#[command(name = "templative")]
#[command(about = "Instantiate project templates from local directories")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Copy template into PATH and run git init + initial commit
    Init {
        /// Template name (as registered with add)
        template_name: String,
        /// Target path (default: current directory)
        #[arg(default_value = ".")]
        target_path: PathBuf,
    },
    /// Register a directory as a template by absolute path
    Add {
        /// Path to template directory (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Template name (default: basename of path)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Remove a template from the registry
    Remove {
        /// Template name
        template_name: String,
    },
    /// List registered templates and their paths
    List,
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init {
            template_name,
            target_path,
        } => ops::cmd_init(template_name, target_path),
        Command::Add { path, name } => ops::cmd_add(path, name),
        Command::Remove { template_name } => ops::cmd_remove(template_name),
        Command::List => ops::cmd_list(),
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{:#}", err);
        std::process::exit(1);
    }
}
