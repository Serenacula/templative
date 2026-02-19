# templative

A tiny CLI for instantiating project templates from local directories. Templates are tracked by absolute path; there is no templating engine.

Git is initialized by default (with an initial commit) when you create a project from a template.

## Install

Download a pre-built binary from the [releases page](https://github.com/serenacula/templative/releases).

### Completions

Generate a completion script for your shell and place it where your shell expects it.

**zsh** — add to a directory in `$fpath`, e.g.:

```sh
templative completions zsh > ~/.zsh/completions/_templative
```

**bash** — source from your `.bashrc`:

```sh
templative completions bash > ~/.bash_completions/templative
echo 'source ~/.bash_completions/templative' >> ~/.bashrc
```

**fish**:

```sh
templative completions fish > ~/.config/fish/completions/templative.fish
```

**powershell** — add to your profile:

```sh
templative completions powershell >> $PROFILE
```

To check whether an installed script is up to date after upgrading templative:

```sh
templative completions zsh --check ~/.zsh/completions/_templative
```

## Commands

-   `templative init TEMPLATE [PATH]` — Copy a template into PATH (default: current directory), then run `git init` and an initial commit.
-   `templative add [PATH] [--name NAME]` — Register an existing directory as a template (default PATH: current directory; default name: directory name).
-   `templative change TEMPLATE [FLAGS]` — Change and set custom features for individual templates.
-   `templative remove TEMPLATE` — Remove a template from the registry. Does not delete the actual template.
-   `templative list` — Show registered templates and their paths.

Optional flags are available to view with `--help`. This also applies to subcommands, e.g. `templative change --help`.

## Config

The config can be used to set values that apply across all templates, or which affect tool functionality. Values here can be overridden by settings in `templates.json` or with flags.

-   Config location:
    -   Linux / macOS: `~/.config/templative/config.json`
    -   Windows: `%APPDATA%\templative\templative\config.json`

A default config is created automatically if there isn't one:

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

            // a description for `template list`
            "description": "an example template",
            // pin a git template to a specific commit/branch/tag
            "git-ref": "v2.0.0",
            // hook that runs before init
            "pre-init": "pwd",
            // hook that runs after init
            "post-init": "ls -l",

            // below are options that override the config behaviours

            "git": "fresh",
            "exclude": ["target"],
            "write-mode": "ask"
        }
    ]
}
```
