# templative

A tiny CLI for instantiating project templates from local directories. Templates are tracked by absolute path; there is no templating engine.

Git is initialized by default (with an initial commit) when you create a project from a template.

## Commands

| Command | Description |
|---|---|
| `templative init TEMPLATE [PATH]` | Copy a template into PATH (default: current directory), then run `git init` and an initial commit. |
| `templative add [PATH] [--name NAME]` | Register an existing directory as a template (default PATH: current directory; default name: directory name). |
| `templative change TEMPLATE [FLAGS]` | Change and set custom features for individual templates. |
| `templative remove TEMPLATE...` | Remove one or more templates from the registry. Does not delete the actual files. |
| `templative update [TEMPLATE] [--check]` | Update cached git templates. Omit name to update all. `--check` reports what's out of date without applying changes. |
| `templative list` | Show registered templates and their paths. |

Optional flags are available to view with `--help`. This also applies to subcommands, e.g. `templative change --help`.

## Install

Download a pre-built binary from the [releases page](https://github.com/serenacula/templative/releases).

### Completions

**zsh:**

```sh
mkdir -p ~/.zsh/completions
templative completions zsh > ~/.zsh/completions/_templative
```

Then add these two lines to your `~/.zshrc` **before** any existing `compinit` call (or at the end if you don't have one):

```sh
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit
```

Then: `source ~/.zshrc`

**bash:**

```sh
mkdir -p ~/.local/share/bash-completion/completions
templative completions bash > ~/.local/share/bash-completion/completions/templative
```

**fish:**

```sh
templative completions fish > ~/.config/fish/completions/templative.fish
```

**PowerShell:**

```sh
templative completions powershell >> $PROFILE
. $PROFILE
```

---

To check whether an installed script is up to date after upgrading templative:

```sh
templative completions zsh --check ~/.zsh/completions/_templative
```

## Config

The config can be used to set values that apply across all templates, or which affect tool functionality. Values here can be overridden by settings in `templates.json` or with flags.

-   Config location:
    -   Linux / macOS: `~/.config/templative/config.json`
    -   Windows: `%APPDATA%\templative\templative\config.json`

A default config is created automatically if there isn't one. This is the default config created, with comments added:

```json
{
    // config.json version
    "version": 1,

    // whether to display colors in `templative list`
    "color": true,

    // git init behaviour
    // fresh: start a new git
    // preserve: clone the original git
    // no-git: do not setup git
    "git": "fresh",

    // files excluded when creating a new template - glob patterns are accepted
    "exclude": ["node_modules", ".DS_Store"],

    // overwrite behaviour during init
    // strict: fail if target directory isn't empty
    // no-overwrite: fail if a file would be overwritten
    // skip-overwrite: skip overwriting files
    // overwrite: overwriting existing files
    // ask: ask the user when collision detected
    "write_mode": "strict"
}
```

## Template Registry

The registry of available templates is the backbone of templative. You can customise individual templates with specific behaviours using `templative change`.

By default optional fields are not set. In this case they fallback to the config setting.

-   Registry location:
    -   Linux / macOS: `~/.config/templative/templates.json`
    -   Windows: `%APPDATA%\templative\templative\templates.json`

```json
{
    // templates.json version
    "version": 2,
    "templates": [
        {
            "name": "example-template",
            "location": "/templates/example-template",

            // below are optional features specific to templates
            // these are not defaults, they're just examples

            // a description for `template list`
            "description": "an example template",
            // pin a git template to a specific commit/branch/tag
            "git-ref": "v2.0.0",
            // hook that runs before init
            "pre-init": "pwd",
            // hook that runs after init
            "post-init": "ls -l",

            // below are optional features that override the config behaviours

            "git": "fresh",
            "exclude": ["target"],
            "write-mode": "ask"
        }
    ]
}
```
