use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod config;
mod errors;
mod fs_copy;
mod git;
mod ops;
mod registry;
mod resolved;
mod utilities;

#[derive(clap::ValueEnum, Clone)]
enum GitOverride {
    #[value(name = "true")]
    Yes,
    #[value(name = "false")]
    No,
    #[value(name = "none")]
    Unset,
}

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
        /// Initialise a git repository (overrides template and config)
        #[arg(long = "git", overrides_with = "no_git")]
        git: bool,
        /// Skip git initialisation (overrides template and config)
        #[arg(long = "no-git", overrides_with = "git")]
        no_git: bool,
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
        /// Set git override for this template
        #[arg(long = "git", overrides_with = "no_git")]
        git: bool,
        /// Set no-git override for this template
        #[arg(long = "no-git", overrides_with = "git")]
        no_git: bool,
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
        /// Set git behaviour for this template (true/false/none)
        #[arg(long)]
        git: Option<GitOverride>,
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
            git,
            no_git,
        } => {
            let git_flag = if git {
                Some(true)
            } else if no_git {
                Some(false)
            } else {
                None
            };
            ops::cmd_init(config, template_name, target_path, git_flag)
        }
        Command::Add {
            path,
            name,
            description,
            git,
            no_git,
        } => {
            let git_flag = if git {
                Some(true)
            } else if no_git {
                Some(false)
            } else {
                None
            };
            ops::cmd_add(path, name, description, git_flag)
        }
        Command::Remove { template_name } => ops::cmd_remove(template_name),
        Command::Change {
            template_name,
            name,
            description,
            location,
            git,
            commit,
            pre_init,
            post_init,
        } => {
            let git_override = git.map(|g| match g {
                GitOverride::Yes => Some(true),
                GitOverride::No => Some(false),
                GitOverride::Unset => None,
            });
            ops::cmd_change(
                template_name,
                name,
                description,
                location,
                git_override,
                commit,
                pre_init,
                post_init,
            )
        }
        Command::List => ops::cmd_list(),
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{:#}", err);
        std::process::exit(1);
    }
}
