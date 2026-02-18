use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod config;
mod errors;
mod fs_copy;
mod git;
mod ops;
mod registry;
mod utilities;

#[derive(Parser)]
#[command(name = "templative")]
#[command(about = "Instantiate project templates from local directories")]
#[command(version, disable_version_flag = true)]
struct Cli {
    #[arg(short = 'v', long, action = clap::ArgAction::Version)]
    version: Option<bool>,
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
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Remove a template from the registry
    Remove {
        /// Template name
        template_name: String,
    },
    /// Update fields on a registered template
    Change {
        /// Template name
        template_name: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New location
        #[arg(long)]
        location: Option<PathBuf>,
        /// Pin to a specific commit
        #[arg(long)]
        commit: Option<String>,
        /// Pre-init hook command
        #[arg(long = "pre-init")]
        pre_init: Option<String>,
        /// Post-init hook command
        #[arg(long = "post-init")]
        post_init: Option<String>,
    },
    /// List registered templates and their paths
    List,
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?;
    match cli.command {
        Command::Init {
            template_name,
            target_path,
        } => ops::cmd_init(config, template_name, target_path),
        Command::Add { path, name, description } => ops::cmd_add(config, path, name, description),
        Command::Remove { template_name } => ops::cmd_remove(config, template_name),
        Command::Change {
            template_name,
            name,
            description,
            location,
            commit,
            pre_init,
            post_init,
        } => ops::cmd_change(config, template_name, name, description, location, commit, pre_init, post_init),
        Command::List => ops::cmd_list(config),
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{:#}", err);
        std::process::exit(1);
    }
}
