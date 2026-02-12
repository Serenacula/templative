# templative

A tiny CLI for instantiating project templates from local directories. Templates are tracked by absolute path; there is no templating engine and no stored copies. Git is always initialized (with an initial commit) when you create a project from a template.

## Commands

-   `templative init TEMPLATE [PATH]` — Copy a template into PATH (default: current directory), then run `git init` and an initial commit.
-   `templative add [PATH] [--name NAME]` — Register an existing directory as a template (default PATH: current directory; default name: directory name).
-   `templative remove NAME` — Remove a template from the registry. Does not delete the actual template.
-   `templative list` — Show registered templates and their paths (missing paths shown as “missing”).

## Registry

-   Location: `~/.config/templative/templates.json` (or platform config equivalent via `directories`).
-   Contents: `version: 1` and a map of template names to absolute paths.

## Exclusions

When copying a template, the following are always excluded:

-   `.git/`
-   `node_modules/`
-   `.DS_Store`

## Known limitations (v1)

-   No remote/git URL templates.
-   No symlink support (symlinks in templates cause an error).
-   Initial commit requires `user.name` and `user.email` to be set in git config.

## Releases

Automated release/publish is configured with:

-   `.github/workflows/release-plz.yml` for versioned releases and crates.io publishing.
-   `.github/workflows/dist.yml` for multi-platform artifacts attached to GitHub Releases.
-   `dist-workspace.toml` for cargo-dist target/installers configuration.

Required repository secret:

-   `CARGO_REGISTRY_TOKEN` (crates.io publish token).
