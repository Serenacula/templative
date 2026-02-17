# templative

A tiny CLI for instantiating project templates from local directories. Templates are tracked by absolute path; there is no templating engine and no stored copies. Git is always initialized (with an initial commit) when you create a project from a template.

## Commands

-   `templative init TEMPLATE [PATH]` — Copy a template into PATH (default: current directory), then run `git init` and an initial commit.
-   `templative add [PATH] [--name NAME]` — Register an existing directory as a template (default PATH: current directory; default name: directory name).
-   `templative remove NAME` — Remove a template from the registry. Does not delete the actual template.
-   `templative list` — Show registered templates and their paths (missing paths shown as “missing”).

## Config

-   Registry location:
    -   Linux / macOS: `~/.config/templative/templates.json`
    -   Windows: `%APPDATA%\templative\templative\templates.json`

## Exclusions

When copying a template, the following are always excluded:

-   `.git/`
-   `node_modules/`
-   `.DS_Store`
