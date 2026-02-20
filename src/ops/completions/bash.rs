pub const VERSION: u32 = 3;

pub const SCRIPT: &str = r#"# templative-completions-version: 3

_templative() {
  local cur="${COMP_WORDS[$COMP_CWORD]}"
  local prev="${COMP_WORDS[$COMP_CWORD-1]}"
  local subcommands="init add change remove list completions update"

  if [[ $COMP_CWORD -eq 1 ]]; then
    COMPREPLY=($(compgen -W "$subcommands --color --no-color --version -v --help -h" -- "$cur"))
    return
  fi

  local subcommand="${COMP_WORDS[1]}"

  case "$subcommand" in
    init)
      case "$prev" in
        --git)
          COMPREPLY=($(compgen -W "fresh preserve no-git" -- "$cur")) ;;
        --write-mode)
          COMPREPLY=($(compgen -W "strict no-overwrite skip-overwrite overwrite ask" -- "$cur")) ;;
        init)
          COMPREPLY=($(compgen -W "$(templative list --names-only 2>/dev/null)" -- "$cur")) ;;
        *)
          if [[ $COMP_CWORD -eq 3 ]]; then
            COMPREPLY=($(compgen -d -- "$cur"))
          else
            COMPREPLY=($(compgen -W "--git --write-mode --help -h" -- "$cur"))
          fi ;;
      esac
      ;;
    add)
      case "$prev" in
        --git)
          COMPREPLY=($(compgen -W "fresh preserve no-git" -- "$cur")) ;;
        --write-mode)
          COMPREPLY=($(compgen -W "strict no-overwrite skip-overwrite overwrite ask" -- "$cur")) ;;
        --name|-n|--description|-d|--git-ref|--exclude)
          ;;
        add)
          COMPREPLY=($(compgen -d -- "$cur")) ;;
        *)
          COMPREPLY=($(compgen -W "--name -n --description -d --git --git-ref --exclude --write-mode --help -h" -- "$cur")) ;;
      esac
      ;;
    change)
      case "$prev" in
        --git)
          COMPREPLY=($(compgen -W "fresh preserve no-git unset" -- "$cur")) ;;
        --write-mode)
          COMPREPLY=($(compgen -W "strict no-overwrite skip-overwrite overwrite ask unset" -- "$cur")) ;;
        --no-cache)
          COMPREPLY=($(compgen -W "true false unset" -- "$cur")) ;;
        --location)
          COMPREPLY=($(compgen -d -- "$cur")) ;;
        --name|--description|--pre-init|--post-init|--git-ref|--exclude)
          ;;
        change)
          COMPREPLY=($(compgen -W "$(templative list --names-only 2>/dev/null)" -- "$cur")) ;;
        *)
          COMPREPLY=($(compgen -W "--name --description --unset-description --location --git --pre-init --unset-pre-init --post-init --unset-post-init --git-ref --unset-git-ref --no-cache --exclude --clear-exclude --write-mode --help -h" -- "$cur")) ;;
      esac
      ;;
    remove)
      if [[ $COMP_CWORD -eq 2 ]]; then
        COMPREPLY=($(compgen -W "$(templative list --names-only 2>/dev/null)" -- "$cur"))
      fi
      ;;
    list)
      COMPREPLY=($(compgen -W "--names-only --color --no-color --help -h" -- "$cur"))
      ;;
    completions)
      case "$prev" in
        --check)
          COMPREPLY=($(compgen -f -- "$cur")) ;;
        completions)
          COMPREPLY=($(compgen -W "zsh bash fish powershell" -- "$cur")) ;;
        *)
          COMPREPLY=($(compgen -W "zsh bash fish powershell --check --help -h" -- "$cur")) ;;
      esac
      ;;
    update)
      case "$prev" in
        update)
          COMPREPLY=($(compgen -W "$(templative list --names-only 2>/dev/null)" -- "$cur")) ;;
        *)
          COMPREPLY=($(compgen -W "--check --help -h" -- "$cur")) ;;
      esac
      ;;
  esac
}

complete -F _templative templative
"#;
