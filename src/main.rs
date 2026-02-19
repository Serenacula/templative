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

use config::{GitMode, WriteMode};
use ops::ChangeOptions;

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

/// `--write-mode strict|no-overwrite|skip-overwrite|overwrite|ask`
#[derive(clap::ValueEnum, Clone)]
enum WriteModeArg {
    Strict,
    #[value(name = "no-overwrite")]
    NoOverwrite,
    #[value(name = "skip-overwrite")]
    SkipOverwrite,
    Overwrite,
    Ask,
}

/// `--write-mode` for change (adds `unset` to clear template-level override)
#[derive(clap::ValueEnum, Clone)]
enum WriteModeChangeArg {
    Strict,
    #[value(name = "no-overwrite")]
    NoOverwrite,
    #[value(name = "skip-overwrite")]
    SkipOverwrite,
    Overwrite,
    Ask,
    Unset,
}

#[derive(clap::ValueEnum, Clone)]
enum NoCacheArg {
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
    /// Force coloured output
    #[arg(long, overrides_with = "no_color")]
    color: bool,
    /// Disable coloured output
    #[arg(long = "no-color", overrides_with = "color")]
    no_color: bool,
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
        /// Write mode: how to handle file collisions in the target directory
        #[arg(long = "write-mode")]
        write_mode: Option<WriteModeArg>,
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
        /// Additional patterns to exclude during init (e.g. dist *.log)
        #[arg(long, num_args = 0..)]
        exclude: Vec<String>,
        /// Write mode: how to handle file collisions in the target directory
        #[arg(long = "write-mode")]
        write_mode: Option<WriteModeArg>,
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
        /// Clear the description
        #[arg(long = "unset-description")]
        unset_description: bool,
        /// New location
        #[arg(long)]
        location: Option<PathBuf>,
        /// Git mode: fresh, preserve, no-git, or unset to remove override
        #[arg(long)]
        git: Option<GitModeChangeArg>,
        /// Pre-init hook command
        #[arg(long = "pre-init")]
        pre_init: Option<String>,
        /// Clear the pre-init hook
        #[arg(long = "unset-pre-init")]
        unset_pre_init: bool,
        /// Post-init hook command
        #[arg(long = "post-init")]
        post_init: Option<String>,
        /// Clear the post-init hook
        #[arg(long = "unset-post-init")]
        unset_post_init: bool,
        /// Pin to a specific git ref (branch, tag, or SHA)
        #[arg(long = "git-ref")]
        git_ref: Option<String>,
        /// Clear the pinned git ref
        #[arg(long = "unset-git-ref")]
        unset_git_ref: bool,
        /// Set no-cache behaviour (true/false/none)
        #[arg(long = "no-cache")]
        no_cache: Option<NoCacheArg>,
        /// Replace template-level exclude patterns (e.g. --exclude dist --exclude "*.log")
        #[arg(long, num_args = 1..)]
        exclude: Vec<String>,
        /// Clear all template-level exclude patterns
        #[arg(long = "clear-exclude")]
        clear_exclude: bool,
        /// Write mode override, or unset to remove template-level override
        #[arg(long = "write-mode")]
        write_mode: Option<WriteModeChangeArg>,
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

fn write_mode_arg_to_mode(arg: WriteModeArg) -> WriteMode {
    match arg {
        WriteModeArg::Strict => WriteMode::Strict,
        WriteModeArg::NoOverwrite => WriteMode::NoOverwrite,
        WriteModeArg::SkipOverwrite => WriteMode::SkipOverwrite,
        WriteModeArg::Overwrite => WriteMode::Overwrite,
        WriteModeArg::Ask => WriteMode::Ask,
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?;
    let color = if cli.no_color { false }
        else if cli.color { true }
        else if std::env::var_os("NO_COLOR").is_some() { false }
        else { config.color };
    match cli.command {
        Command::Init {
            template_name,
            target_path,
            git,
            write_mode,
        } => {
            let git_flag = git.map(git_mode_arg_to_mode);
            let write_mode_flag = write_mode.map(write_mode_arg_to_mode);
            ops::cmd_init(config, template_name, target_path, git_flag, write_mode_flag)
        }
        Command::Add {
            path,
            name,
            description,
            git,
            git_ref,
            no_cache,
            exclude,
            write_mode,
        } => {
            let git_flag = git.map(git_mode_arg_to_mode);
            let no_cache_flag = if no_cache { Some(true) } else { None };
            let write_mode_flag = write_mode.map(write_mode_arg_to_mode);
            ops::cmd_add(path, name, description, git_flag, git_ref, no_cache_flag, exclude, write_mode_flag)
        }
        Command::Remove { template_name } => ops::cmd_remove(template_name),
        Command::Change {
            template_name,
            name,
            description,
            unset_description,
            location,
            git,
            pre_init,
            unset_pre_init,
            post_init,
            unset_post_init,
            git_ref,
            unset_git_ref,
            no_cache,
            exclude,
            clear_exclude,
            write_mode,
        } => {
            let git_override = git.map(|git_arg| match git_arg {
                GitModeChangeArg::Fresh => Some(GitMode::Fresh),
                GitModeChangeArg::Preserve => Some(GitMode::Preserve),
                GitModeChangeArg::NoGit => Some(GitMode::NoGit),
                GitModeChangeArg::Unset => None,
            });
            let no_cache_override = no_cache.map(|no_cache_arg| match no_cache_arg {
                NoCacheArg::Yes => Some(true),
                NoCacheArg::No => Some(false),
                NoCacheArg::Unset => None,
            });
            let exclude_change = if clear_exclude {
                Some(None)
            } else if !exclude.is_empty() {
                Some(Some(exclude))
            } else {
                None
            };
            let write_mode_change = write_mode.map(|arg| match arg {
                WriteModeChangeArg::Unset => None,
                WriteModeChangeArg::Strict => Some(WriteMode::Strict),
                WriteModeChangeArg::NoOverwrite => Some(WriteMode::NoOverwrite),
                WriteModeChangeArg::SkipOverwrite => Some(WriteMode::SkipOverwrite),
                WriteModeChangeArg::Overwrite => Some(WriteMode::Overwrite),
                WriteModeChangeArg::Ask => Some(WriteMode::Ask),
            });
            ops::cmd_change(template_name, ChangeOptions {
                name,
                description: if unset_description { Some(None) } else { description.map(Some) },
                location,
                git: git_override,
                pre_init: if unset_pre_init { Some(None) } else { pre_init.map(Some) },
                post_init: if unset_post_init { Some(None) } else { post_init.map(Some) },
                git_ref: if unset_git_ref { Some(None) } else { git_ref.map(Some) },
                no_cache: no_cache_override,
                exclude: exclude_change,
                write_mode: write_mode_change,
            })
        }
        Command::List => ops::cmd_list(color),
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{:#}", err);
        std::process::exit(1);
    }
}
