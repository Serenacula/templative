# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/Serenacula/templative/compare/v0.6.5...v1.0.0) - 2026-02-21

### Added

-   add manual trigger to homebrew workflow

### Fixed

-   remove stale --no-cache from bash completions

## [0.6.5](https://github.com/Serenacula/templative/compare/v0.6.4...v0.6.5) - 2026-02-20

### Added

-   allow removing multiple templates at once
-   add templative update command

### Fixed

-   move --color/--no-color flags to list subcommand

## [0.6.4](https://github.com/Serenacula/templative/compare/v0.6.3...v0.6.4) - 2026-02-20

### Added

-   add completions subcommand for zsh, bash, fish, and powershell
-   add --names-only flag to list for autocomplete support
-   add --color/--no-color flags; no-overwrite pre-flights collisions

## Removed

-   remove no_cache/update_on_init, always-fetch cache, fix git_ref temp clone, fix pre-init ordering

### Fixed

-   add directory completion for init path and add path in bash and fish
-   rename no-cache unset value from none to unset
-   remove global flag from --color/--no-color

### Other

-   remove cargo install from readme
-   add install section with completions setup instructions

## [0.6.3](https://github.com/Serenacula/templative/compare/v0.6.2...v0.6.3) - 2026-02-19

### Added

-   support symlinks when copying templates

### Fixed

-   address second review pass
-   address code review findings

### Other

-   remove commit field, git_ref handles all ref pinning

## [0.6.2](https://github.com/Serenacula/templative/compare/v0.6.1...v0.6.2) - 2026-02-19

### Added

-   write mode for controlling file collision behaviour on init
-   configurable exclude patterns via glob matching
-   use #fcdd2a yellow with fallback to basic yellow for non-truecolor terminals

### Fixed

-   apply row style to whole line; remove symlink states from list
-   apply row style to name column only
-   don't apply strikethrough style to name/status column padding
-   apply row style only to name and status columns

### Other

-   clarify symlink planning notes
-   code quality pass
-   rename single-letter variables and clean up list command

## [0.6.1](https://github.com/Serenacula/templative/compare/v0.6.0...v0.6.1) - 2026-02-19

### Added

-   add STATUS column to list with per-state notes
-   colour-code list rows by template state
-   hide status and description columns when unused

### Other

-   split ops.rs into per-command files under src/ops/; list column order NAME STATUS DESCRIPTION LOCATION

## [0.6.0](https://github.com/Serenacula/templative/compare/v0.5.1...v0.6.0) - 2026-02-18

### Added

-   underline list headers
-   show description in list output with aligned columns
-   [**breaking**] merge git and fresh flags into single GitMode enum

### Fixed

-   underline header text only, not trailing padding

### Other

-   updating lock

## [0.5.1](https://github.com/Serenacula/templative/compare/v0.5.0...v0.5.1) - 2026-02-18

### Added

-   add git URL template support
-   implement pre_init and post_init hooks

### Other

-   add tests for git URL features
-   updating readme

## [0.5.0](https://github.com/Serenacula/templative/compare/v0.1.0...v0.5.0) - 2026-02-18

### Added

-   add --version and -v flags
-   add config.json foundation
-   add change command
-   add git setting with flag/template/config resolution
-   add tests for git resolution
-   add ResolvedOptions infrastructure
-   adding message when listing no templates

### Changed

-   adding pr job to release-plz

### Other

-   remove unused config params
-   bump to 0.4.0
-   version bump
-   registry schema v2: template objects with optional fields
-   bumping version
-   bumping version
-   bumping version
-   .
-   tiny safety fixes
-   organising planning docs
-   gitignore
-   minor refactor
-   initial version release
-   readme adjustment
