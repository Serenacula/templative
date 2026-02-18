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

use config::GitMode;

/// `--git fresh|preserve|no-git` for init and add
#[derive(clap::ValueEnum, Clone)]
enum GitModeArg {
    Fresh,
    Preserve,
    #[value(name = "no-git")]
    NoGit,
}

/// `--git fresh|preserve|no-git|unset` for change
#[derive(clap::ValueEnum, Clone)]
enum GitModeChangeArg {
    Fresh,
    Preserve,
    #[value(name = "no-git")]
    NoGit,
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
    /// Copy template into PATH
    Init {
        /// Template name (as registered with add)
        template_name: String,
        /// Target path (default: current directory)
        #[arg(default_value = ".")]
        target_path: PathBuf,
        /// Git mode: fresh (copy + new history), preserve (clone), no-git (copy only)
        #[arg(long)]
        git: Option<GitModeArg>,
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
        /// Git mode: fresh (copy + new history), preserve (clone), no-git (copy only)
        #[arg(long)]
        git: Option<GitModeArg>,
        /// Pin to a specific git ref (branch, tag, or SHA)
        #[arg(long = "git-ref")]
        git_ref: Option<String>,
        /// Skip cache; clone fresh on each init
        #[arg(long = "no-cache")]
        no_cache: bool,
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
        /// Git mode: fresh, preserve, no-git, or unset to remove override
        #[arg(long)]
        git: Option<GitModeChangeArg>,
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
    },
    /// List registered templates and their paths
    List,
}

fn git_mode_arg_to_mode(arg: GitModeArg) -> GitMode {
    match arg {
        GitModeArg::Fresh => GitMode::Fresh,
        GitModeArg::Preserve => GitMode::Preserve,
        GitModeArg::NoGit => GitMode::NoGit,
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?;
    match cli.command {
        Command::Init {
            template_name,
            target_path,
            git,
        } => {
            let git_flag = git.map(git_mode_arg_to_mode);
            ops::cmd_init(config, template_name, target_path, git_flag)
        }
        Command::Add {
            path,
            name,
            description,
            git,
            git_ref,
            no_cache,
        } => {
            let git_flag = git.map(git_mode_arg_to_mode);
            let no_cache_flag = if no_cache { Some(true) } else { None };
            ops::cmd_add(path, name, description, git_flag, git_ref, no_cache_flag)
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
        } => {
            let git_override = git.map(|g| match g {
                GitModeChangeArg::Fresh => Some(GitMode::Fresh),
                GitModeChangeArg::Preserve => Some(GitMode::Preserve),
                GitModeChangeArg::NoGit => Some(GitMode::NoGit),
                GitModeChangeArg::Unset => None,
            });
            let no_cache_override = no_cache.map(|v| match v {
                NoCacheOverride::Yes => Some(true),
                NoCacheOverride::No => Some(false),
                NoCacheOverride::Unset => None,
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
