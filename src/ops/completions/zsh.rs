pub const VERSION: u32 = 3;

pub const SCRIPT: &str = r#"#compdef templative
# templative-completions-version: 3

_templative_template_names() {
  local -a names
  names=(${(f)"$(templative list --names-only 2>/dev/null)"})
  _describe 'template' names
}

_templative() {
  local state line

  _arguments -C \
    '(-v --version)'{-v,--version}'[Print version]' \
    '--color[Force coloured output]' \
    '--no-color[Disable coloured output]' \
    '1:command:->command' \
    '*::args:->args'

  case $state in
    command)
      local -a commands
      commands=(
        'init:Copy a template into a directory'
        'add:Register a directory or git URL as a template'
        'change:Update fields on a registered template'
        'remove:Remove a template from the registry'
        'list:List registered templates'
        'completions:Generate shell completion scripts'
        'update:Update cached git templates'
      )
      _describe 'command' commands
      ;;
    args)
      case $line[1] in
        init)
          _arguments \
            '--git[Git mode]:mode:(fresh preserve no-git)' \
            '--write-mode[Write mode]:mode:(strict no-overwrite skip-overwrite overwrite ask)' \
            '1:template:_templative_template_names' \
            '2:path:_files -/'
          ;;
        add)
          _arguments \
            '(-n --name)'{-n,--name}'[Template name]:name:' \
            '(-d --description)'{-d,--description}'[Description]:desc:' \
            '--git[Git mode]:mode:(fresh preserve no-git)' \
            '--git-ref[Pin to git ref]:ref:' \
            '--exclude[Exclude patterns]:pattern:' \
            '--write-mode[Write mode]:mode:(strict no-overwrite skip-overwrite overwrite ask)' \
            '1:path:_files -/'
          ;;
        change)
          _arguments \
            '--name[New name]:name:' \
            '--description[New description]:desc:' \
            '--unset-description[Clear description]' \
            '--location[New location]:path:_files -/' \
            '--git[Git mode]:mode:(fresh preserve no-git unset)' \
            '--pre-init[Pre-init hook]:cmd:' \
            '--unset-pre-init[Clear pre-init hook]' \
            '--post-init[Post-init hook]:cmd:' \
            '--unset-post-init[Clear post-init hook]' \
            '--git-ref[Pin to git ref]:ref:' \
            '--unset-git-ref[Clear git ref]' \
            '--exclude[Exclude patterns]:pattern:' \
            '--clear-exclude[Clear all exclude patterns]' \
            '--write-mode[Write mode]:mode:(strict no-overwrite skip-overwrite overwrite ask unset)' \
            '1:template:_templative_template_names'
          ;;
        remove)
          _arguments \
            '1:template:_templative_template_names'
          ;;
        list)
          _arguments \
            '--names-only[Print only template names]'
          ;;
        completions)
          _arguments \
            '--check[Check if installed script is up to date]:path:_files' \
            '1:shell:(zsh bash fish powershell)'
          ;;
        update)
          _arguments \
            '--check[Check for updates without applying]' \
            '1:template:_templative_template_names'
          ;;
      esac
      ;;
  esac
}

_templative
"#;
