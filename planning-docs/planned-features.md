# Planned Features

## Major

-   Have a configuration file called config.json in .config/templative/
    -   For all future flag features, the default can be set here
    -   Create if not present, with all defaults already added
-   Pull git links as template
    -   Should still have the same overall git setup of a single commit!
        -   Probably means deleting the git folder and re-initialising
    -   Optional caching based on flag.
        -   Flagging means we need an update command
        -   We also want to be able to specify specific commits
-   Templates can optionally run a pre-init and post-init command. These are configured in the config directory
    -   If pre-init fails, abort with error
    -   Pre-init is run before folder checks and the like, so if it dirties the init folder we'll error
    -   If post-init fails, tell user
-   More complex list information:
    -   Headers of TEMPLATE and LOCATION, underlined
        -   These should be columns
    -   Yellow warning and `(no git)` if git not initialised in template
    -   Yellow warning and `(update available)` if git initialised and template is not up to date
    -   Yellow warning and `(empty)` if directory empty except from git
        -   Red warning and `(empty)` if directory is empty including git
            -   It would still show `(no git)` like normal obviously
    -   Blue colour and `(single file)` for links pointing at single files
        -   git is not relevant for this
    -   Blue colour and `(symlink)` for links pointing to a symlink
    -   Blue colour and `(at commit <COMMIT>)` if not at head
        -   This blue overrides the yellow of `(update available)`, but that warning is still shown
        -   Maybe `(in branch <BRANCH>)` if at head of a branch?
    -   Red colour and `(broken symlink)` for broken links
    -   If `template list` and no templates, tell user how to add one
-   Configurable exclusions in config
-   Optional descriptions for templates
    -   Added with --description flag during add
    -   Printed in list mode
-   Update feature
    -   Update git templates `template update <TEMPLATE>`
    -   Defaults to updating all templates
-   Change feature
    -   Change name `template change <TEMPLATE> --name <NEW_NAME>`
    -   Change description `template change <TEMPLATE> --description "<NEW_DESCRIPTION>"`
    -   Change location `template change <TEMPLATE> --location <NEW_LOCATION>`
-   Auto-complete for templates
    -   Apparently best done via a `template list --names-only` feature, which then we plug into the shell

## Minor

-   no-colour flag
    -   Removes underline too
-   Add git & no-git flags
-   Support symlinks, creating a new one
    -   If resolves inside the template, keep a relative symlink
    -   Else if resolves outside the template, make it absolute
    -   If the file cannot be found, print non-breaking warning! But still create the symlink
-   Symlink mode: literal, copies exactly
-   Add flag resolvesymlink
    -   Not resolving is default behaviour, create a new symlink
    -   resolvesymlink instead creates whatever is on the other end of the symlink
        -   Make absolutely sure there's no chance of recursion first
        -   Add a warning that this feature is experimental
-   Configure write instructions:
    -   Strict no overwrite (default): error if directory is not empty
    -   Write no overwrite: error on file overwrite, allow non-empty directory, commit to pre-existing git if available
    -   Write skip overwrite: same as above, but skips files that would overwrite not error
    -   Overwrite: Replace overwritten files with new ones
-   Prune feature, removing all the red templates
    -   For empty yellow templates, ask if user wants to delete

## Tiny

-   Check user.name and user.email are set, error if not
-   Detect recursion and error if it happens

    -   Don't allow initialising a template inside itself
    -   If resolvesymlink, check for any copy loops before starting

-   Fail with error on init for missing template
