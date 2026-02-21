pub const VERSION: u32 = 4;

pub const SCRIPT: &str = r#"# templative-completions-version: 4

# Disable file completion globally
complete -c templative -f

# Global flags
complete -c templative -n 'not __fish_seen_subcommand_from init add change remove list completions update' -s v -l version -d 'Print version'

# Subcommands
complete -c templative -n 'not __fish_seen_subcommand_from init add change remove list completions update' -a init -d 'Copy a template into a directory'
complete -c templative -n 'not __fish_seen_subcommand_from init add change remove list completions update' -a add -d 'Register a directory or git URL as a template'
complete -c templative -n 'not __fish_seen_subcommand_from init add change remove list completions update' -a change -d 'Update fields on a registered template'
complete -c templative -n 'not __fish_seen_subcommand_from init add change remove list completions update' -a remove -d 'Remove a template from the registry'
complete -c templative -n 'not __fish_seen_subcommand_from init add change remove list completions update' -a list -d 'List registered templates'
complete -c templative -n 'not __fish_seen_subcommand_from init add change remove list completions update' -a completions -d 'Generate shell completion scripts'
complete -c templative -n 'not __fish_seen_subcommand_from init add change remove list completions update' -a update -d 'Update cached git templates'

# Returns true when 'init' has been given and at least one non-flag argument follows it
function __templative_init_has_template
    set -l tokens (commandline -poc)
    set -l past_init 0
    set -l count 0
    for token in $tokens
        if test $past_init -eq 1; and not string match -qr -- '^-' $token
            set count (math $count + 1)
        end
        if test $token = init
            set past_init 1
        end
    end
    test $count -ge 1
end

# init
complete -c templative -n '__fish_seen_subcommand_from init' -a '(templative list --names-only 2>/dev/null)'
complete -c templative -n '__fish_seen_subcommand_from init; and __templative_init_has_template' -F -d 'Target directory'
complete -c templative -n '__fish_seen_subcommand_from init' -l git -d 'Git mode' -a 'fresh preserve no-git' -r
complete -c templative -n '__fish_seen_subcommand_from init' -l write-mode -d 'Write mode' -a 'strict no-overwrite skip-overwrite overwrite ask' -r

# add
complete -c templative -n '__fish_seen_subcommand_from add' -F -d 'Template directory'
complete -c templative -n '__fish_seen_subcommand_from add' -s n -l name -d 'Template name' -r
complete -c templative -n '__fish_seen_subcommand_from add' -s d -l description -d 'Description' -r
complete -c templative -n '__fish_seen_subcommand_from add' -l git -d 'Git mode' -a 'fresh preserve no-git' -r
complete -c templative -n '__fish_seen_subcommand_from add' -l git-ref -d 'Pin to git ref' -r
complete -c templative -n '__fish_seen_subcommand_from add' -l exclude -d 'Exclude patterns' -r
complete -c templative -n '__fish_seen_subcommand_from add' -l write-mode -d 'Write mode' -a 'strict no-overwrite skip-overwrite overwrite ask' -r

# change
complete -c templative -n '__fish_seen_subcommand_from change' -a '(templative list --names-only 2>/dev/null)'
complete -c templative -n '__fish_seen_subcommand_from change' -l name -d 'New name' -r
complete -c templative -n '__fish_seen_subcommand_from change' -l description -d 'New description' -r
complete -c templative -n '__fish_seen_subcommand_from change' -l unset-description -d 'Clear description'
complete -c templative -n '__fish_seen_subcommand_from change' -l location -d 'New location' -r -F
complete -c templative -n '__fish_seen_subcommand_from change' -l git -d 'Git mode' -a 'fresh preserve no-git unset' -r
complete -c templative -n '__fish_seen_subcommand_from change' -l pre-init -d 'Pre-init hook' -r
complete -c templative -n '__fish_seen_subcommand_from change' -l unset-pre-init -d 'Clear pre-init hook'
complete -c templative -n '__fish_seen_subcommand_from change' -l post-init -d 'Post-init hook' -r
complete -c templative -n '__fish_seen_subcommand_from change' -l unset-post-init -d 'Clear post-init hook'
complete -c templative -n '__fish_seen_subcommand_from change' -l git-ref -d 'Pin to git ref' -r
complete -c templative -n '__fish_seen_subcommand_from change' -l unset-git-ref -d 'Clear git ref'
complete -c templative -n '__fish_seen_subcommand_from change' -l exclude -d 'Exclude patterns' -r
complete -c templative -n '__fish_seen_subcommand_from change' -l clear-exclude -d 'Clear all exclude patterns'
complete -c templative -n '__fish_seen_subcommand_from change' -l write-mode -d 'Write mode' -a 'strict no-overwrite skip-overwrite overwrite ask unset' -r

# remove
complete -c templative -n '__fish_seen_subcommand_from remove' -a '(templative list --names-only 2>/dev/null)'

# list
complete -c templative -n '__fish_seen_subcommand_from list' -l names-only -d 'Print only template names'
complete -c templative -n '__fish_seen_subcommand_from list' -l color -d 'Force coloured output'
complete -c templative -n '__fish_seen_subcommand_from list' -l no-color -d 'Disable coloured output'

# completions
complete -c templative -n '__fish_seen_subcommand_from completions' -a 'zsh bash fish powershell'
complete -c templative -n '__fish_seen_subcommand_from completions' -l check -d 'Check if installed script is up to date' -r -F

# update
complete -c templative -n '__fish_seen_subcommand_from update' -a '(templative list --names-only 2>/dev/null)'
complete -c templative -n '__fish_seen_subcommand_from update' -l check -d 'Check for updates without applying'
"#;
