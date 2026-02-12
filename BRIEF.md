# Templative (Rust) — build plan

Goal: a tiny, opinionated CLI for instantiating project templates from local directories, tracked by absolute path. No templating engine. No stored copies. Git is always initialized on `init`.

Non-goals (v1): remote git templates, version pinning, hooks, variable substitution, interactive prompts.

---

## 0) UX summary

### Commands

-   `templative init TEMPLATE [PATH=.]`

    -   Copy files from template directory into PATH
    -   Refuse dangerous targets (`/`, `$HOME`) unless forced (internal gate is OK even if no flags exposed yet)
    -   Require PATH empty (strict)
    -   Always: `git init` + initial commit

-   `templative add [PATH=. ] [--name NAME]`

    -   Register an existing directory as a template by absolute realpath
    -   Default name = basename(PATH)
    -   Fail if name already exists

-   `templative remove NAME`

    -   Remove from registry only

-   `templative list`
    -   Show templates and their paths
    -   If path missing → show as “missing” (red + strikethrough)

### Template registry

-   File: `~/.config/templative/templates.json`
-   Structure:
    -   `version: 1`
    -   `templates: { "name": "/absolute/path" }`

---

## 1) CLI argument design (clap)

Use `clap` derive.

Subcommands:

-   `Init { template_name: String, target_path: PathBuf }`
-   `Add { path: PathBuf, name: Option<String> }`
-   `Remove { template_name: String }`
-   `List`

Defaults:

-   `Init.target_path` default `"."`
-   `Add.path` default `"."`

Output conventions:

-   Normal success: minimal single-line confirmation per command (unless `list`).
-   Errors: message to stderr + non-zero exit.

---

## 2) Safety / invariants (v1)

### Paths

-   On `add`, store `canonicalize()`d path (realpath).
-   On `init`, resolve `target_path` similarly (create dir if not exists? decide; recommended: create if missing).
-   Explicitly block:
    -   target == `/`
    -   target == `$HOME`
    -   Possibly also block parent of home? (optional)
    -   If blocked: error with “refusing to operate on …”.
    -   You can keep this internal; flags can be added later.

### Empty target requirement

-   `init` requires target directory exists and is empty.
    -   Empty means: no entries at all. (Do not allow `.git` to already exist.)
    -   If non-empty: error.

### Copy rules

-   Always exclude:
    -   `.git/`
    -   `node_modules/`
    -   `.DS_Store`
-   Everything else copied recursively.
-   Preserve permissions and timestamps if easy (permissions is the main one).

### Git requirement

-   After copy:
    -   run `git init` in target
    -   run `git add -A`
    -   run `git commit -m "Initial commit from template: <name>"`
-   If git commands fail:
    -   print stderr and exit non-zero
    -   do not attempt rollback (v1)

### Template existence

-   `init` errors if template name not found, or template path missing, or template directory unreadable.

---

## 3) Terminal output / styling

Use `owo-colors` for minimal color:

-   `list`:
    -   normal template: `name  path`
    -   missing template:
        -   name struck-through and red (if supported) OR fallback: red + prefix `MISSING`
        -   include reason: “(missing)”
-   Keep output stable for grepping (avoid fancy tables).

Note: true strikethrough depends on terminal support; implement as ANSI `\x1b[9m` and fall back gracefully if desired.

---

## 4) File layout

Crate layout (simple):

-   `src/main.rs` — clap + dispatch
-   `src/registry.rs` — load/save templates.json
-   `src/ops.rs` — add/remove/list/init logic
-   `src/fs_copy.rs` — copy implementation with exclusions
-   `src/git.rs` — run git subprocess helpers
-   `src/errors.rs` — error enum + display (optional; `anyhow` is fine)

Dependencies (suggested minimal):

-   `clap = { version = "4", features = ["derive"] }`
-   `serde = { version = "1", features = ["derive"] }`
-   `serde_json = "1"`
-   `directories = "5"` (config dir resolution)
-   `walkdir = "2"` (recursive copy)
-   `owo-colors = "4"`
-   `anyhow = "1"` (or `thiserror` + custom types)

---

## 5) Registry implementation details

### Config path resolution

Use `directories::ProjectDirs`:

-   `ProjectDirs::from("dev", "templative", "templative")`
-   Config dir: `project_dirs.config_dir()`
-   Ensure directory exists.

Registry file:

-   `<config_dir>/templates.json`

### Load / Save behavior

-   If file missing:
    -   treat as empty registry
    -   create on first write
-   Load:
    -   parse JSON
    -   if `version != 1`, error “unsupported registry version”
-   Save:
    -   write atomically:
        -   write to temp file in same dir
        -   rename over original

### Canonical path storage

-   On add: `canonicalize(path)` and store string.
-   On list/init: use stored path string as PathBuf, check `exists()`.

---

## 6) Copy implementation (walkdir)

Algorithm:

1. Validate source dir exists.
2. Iterate over entries using `WalkDir::new(source_dir)`:
    - Skip root itself.
    - Determine relative path: `entry.path().strip_prefix(source_dir)`
    - Exclusion filter:
        - if any component matches `.git` or `node_modules` → skip subtree (`filter_entry`)
        - if file name == `.DS_Store` → skip
3. For directories: create destination dir.
4. For files:
    - create parent dirs
    - copy file bytes (`std::fs::copy`)
    - optionally set permissions from source:
        - `metadata.permissions()`, `set_permissions`

Corner cases:

-   Symlinks:
    -   Decide now:
        -   simplest: copy symlink as symlink on unix? (harder)
        -   or dereference and copy file contents? (walkdir provides info)
    -   v1 recommendation: error on symlinks with a clear message (“symlinks not supported yet”)
    -   This avoids surprising behavior.

---

## 7) Git subprocess helper

Use `std::process::Command`:

-   Always set `.current_dir(target_path)`
-   Capture stderr/stdout.
-   On failure: include command + stderr in error.

Commands:

-   `git init`
-   `git add -A`
-   `git commit -m <message>`

Potential failure notes (document in README):

-   if user.name/email not set, commit fails.

---

## 8) Command behaviors (exact)

### `templative add [PATH] [--name NAME]`

-   Resolve PATH default `"."`
-   Canonicalize PATH
-   Determine template name:
    -   if `--name` provided, use it
    -   else basename of canonicalized path
-   Load registry
-   If name exists → error
-   Insert name -> canonical path string
-   Save registry
-   Print: `added <name> -> <path>`

### `templative remove NAME`

-   Load registry
-   If not found → error
-   Remove entry
-   Save
-   Print: `removed <name>`

### `templative list`

-   Load registry (empty OK)
-   For each template name sorted:
    -   if path exists: print normal
    -   else: print missing highlighted
-   Exit 0 always (even if missing templates), unless registry unreadable.

### `templative init TEMPLATE [PATH]`

-   Load registry
-   Resolve TEMPLATE -> template_path; if missing error
-   Resolve target path:
    -   default `"."`
    -   if path does not exist: create dir
    -   canonicalize target (after create)
-   Safety checks: block `/` and `$HOME`
-   Empty check
-   Copy template_path -> target (with exclusions)
-   Run git init/add/commit
-   Print: `created <target> from <template>`

---

## 9) Tests (keep light)

Unit-ish tests:

-   Registry:
    -   load missing file -> empty registry
    -   save then reload roundtrip
    -   rejects version mismatch
-   Copy:
    -   copies nested structure
    -   excludes `.git`, `node_modules`, `.DS_Store`
    -   errors on symlink (if chosen)

Integration tests (optional):

-   Use tempdir to:
    -   add template, init into new dir, verify files exist + `.git` exists.

---

## 10) v1 README bullets (keep tiny)

-   What it does (path-tracked templates, copy + git init)
-   How to add / init / list / remove
-   Where registry lives
-   Exclusions list
-   Known limitations:
    -   no git/remote templates
    -   no symlink support (if chosen)
    -   git commit requires git user config

---

## 11) Next features (parking lot)

-   Git URL templates (`add --git URL`)
-   Pin commit (`--ref` / store resolved commit)
-   `templative update NAME` (refresh git template)
-   Safety flags (`--force`, `--dry-run`)
-   Allow init into non-empty dir with explicit overwrite rules
-   Optional post-init hooks (danger of cookiecutter creep)
