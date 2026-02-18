use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod config;
mod errors;
mod fs_copy;
mod git;
mod git_cache;
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

#[derive(clap::ValueEnum, Clone)]
enum FreshOverride {
    #[value(name = "true")]
    Yes,
    #[value(name = "false")]
    No,
    #[value(name = "none")]
    Unset,
}

#[derive(clap::ValueEnum, Clone)]
enum NoCacheOverride {
    #[value(name = "true")]
    Yes,
    #[value(name = "false")]
    No,
    #[value(name = "none")]
    Unset,
}

#[derive(Parser)]
#[command(name = "templative")]
#[command(about = "Instantiate project templates from local directories or git URLs")]
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
        /// Strip git history on copy (overrides template and config)
        #[arg(long = "fresh", overrides_with = "no_fresh")]
        fresh: bool,
        /// Preserve git history via local clone (overrides template and config)
        #[arg(long = "no-fresh", overrides_with = "fresh")]
        no_fresh: bool,
    },
    /// Register a directory or git URL as a template
    Add {
        /// Path or git URL to template (default: current directory)
        #[arg(default_value = ".")]
        path: String,
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
        /// Pin to a specific git ref (branch, tag, or SHA)
        #[arg(long = "git-ref")]
        git_ref: Option<String>,
        /// Skip cache; clone fresh on each init
        #[arg(long = "no-cache")]
        no_cache: bool,
        /// Strip git history on init (overrides config)
        #[arg(long = "fresh", overrides_with = "no_fresh")]
        fresh: bool,
        /// Preserve git history on init (overrides config)
        #[arg(long = "no-fresh", overrides_with = "fresh")]
        no_fresh: bool,
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
        /// Pin to a specific git ref (branch, tag, or SHA)
        #[arg(long = "git-ref")]
        git_ref: Option<String>,
        /// Set no-cache behaviour (true/false/none)
        #[arg(long = "no-cache")]
        no_cache: Option<NoCacheOverride>,
        /// Set fresh behaviour (true/false/none)
        #[arg(long = "fresh")]
        fresh: Option<FreshOverride>,
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
            fresh,
            no_fresh,
        } => {
            let git_flag = if git {
                Some(true)
            } else if no_git {
                Some(false)
            } else {
                None
            };
            let fresh_flag = if fresh {
                Some(true)
            } else if no_fresh {
                Some(false)
            } else {
                None
            };
            ops::cmd_init(config, template_name, target_path, git_flag, fresh_flag)
        }
        Command::Add {
            path,
            name,
            description,
            git,
            no_git,
            git_ref,
            no_cache,
            fresh,
            no_fresh,
        } => {
            let git_flag = if git {
                Some(true)
            } else if no_git {
                Some(false)
            } else {
                None
            };
            let fresh_flag = if fresh {
                Some(true)
            } else if no_fresh {
                Some(false)
            } else {
                None
            };
            let no_cache_flag = if no_cache { Some(true) } else { None };
            ops::cmd_add(path, name, description, git_flag, git_ref, no_cache_flag, fresh_flag)
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
            git_ref,
            no_cache,
            fresh,
        } => {
            let git_override = git.map(|g| match g {
                GitOverride::Yes => Some(true),
                GitOverride::No => Some(false),
                GitOverride::Unset => None,
            });
            let no_cache_override = no_cache.map(|v| match v {
                NoCacheOverride::Yes => Some(true),
                NoCacheOverride::No => Some(false),
                NoCacheOverride::Unset => None,
            });
            let fresh_override = fresh.map(|v| match v {
                FreshOverride::Yes => Some(true),
                FreshOverride::No => Some(false),
                FreshOverride::Unset => None,
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
                git_ref,
                no_cache_override,
                fresh_override,
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
