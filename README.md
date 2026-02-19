# templative

A tiny CLI for instantiating project templates from local directories. Templates are tracked by absolute path; there is no templating engine and no stored copies. Git is always initialized (with an initial commit) when you create a project from a template.

## Commands

-   `templative init TEMPLATE [PATH]` — Copy a template into PATH (default: current directory), then run `git init` and an initial commit.
-   `templative add [PATH] [--name NAME]` — Register an existing directory as a template (default PATH: current directory; default name: directory name).
-   `templative remove NAME` — Remove a template from the registry. Does not delete the actual template.
-   `templative list` — Show registered templates and their paths (missing paths shown as “missing”).

Optional flags are available to view with `--help`

## Config

Config can be used to set values that apply across all templates, or which affect tool functionality. Values here can be be overridden by settings in `templates.json` or with flags.

-   Config location:
    -   Linux / macOS: `~/.config/templative/config.json`
    -   Windows: `%APPDATA%\templative\templative\config.json`

A default config is created automatically if there isn't one:

```json
{
    // config version
    "version": 1,
    // whether to display colors in `templative list`
    "color": true,
    // git init behaviour
    // fresh: start a new git
    // preserve: clone the original git
    // no-git: do not setup git
    "git": "fresh",
    // whether to update a template to the latest version when initialising
    // always: always update to the latest version
    // only-url: only update for cached git repos, not local templates
    // never: do not update git repos
    "update_on_init": "only-url",
    // whether to skip using the cache for git repos
    // true: always pulls directly from the repo
    // false: caches repos for quicker init
    "no_cache": false,
    // files excluded when creating a new template
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

-   Registry location:
    -   Linux / macOS: `~/.config/templative/templates.json`
    -   Windows: `%APPDATA%\templative\templative\templates.json`

```json
{
    "version": 2,
    "templates": [
        {
            "name": "node-writing",
            "location": "/Users/serenacula/Sync/Writing/Other/Templates/node-writing"
        }
    ]
}
```
