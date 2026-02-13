# Planned Features

## Major

-   Have a configuration file called config.json in .config/templative/
    -   For all future flag features, the default can be set here
    -   Create if not present, with all defaults already added
-   Add git & no-git flags
-   Support symlinks
    -   This should create a new symlink
-   Add flags symlink & resolvesymlink
    -   Symlink is default behaviour, creating a new symlink
        -   If inside the template, keep a relative symlink
        -   Else if outside the template, make it absolute
        -   If the file cannot be found, print non-breaking warning!
    -   resolvesymlink instead creates whatever is on the other end of the symlink
        -   Add a warning that this feature is experimental
-   Pull git links as template
    -   Should still have the same overall git setup of a single commit!
        -   Probably means deleting the git folder and re-initialising
    -   Optional caching based on flag.
        -   Flagging means we need an update command
        -   We also want to be able to specify specific commits
-   Configure write instructions:
    -   Strict no overwrite (default): error if directory is not empty
    -   Write no overwrite: error on file overwrite, allow non-empty directory, commit to pre-existing git if available
    -   Write skip overwrite: same as above, but skips files that would overwrite not error
    -   Overwrite: Replace overwritten files with new ones
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
    -   Blue colour and `(symlink)` for links pointing to a symlink
    -   Red colour and `(broken symlink)` for broken links
    -   If `template list` and no templates, tell user how to add one
-   Configurable exclusions in config
-   no-colour flag
    -   Removes underline too
-   Prune feature, removing all the red templates
    -   For empty yellow templates, ask if user wants to delete
-   Optional descriptions for templates
    -   Added with --description flag during add
    -   Printed in list mode
-   Update feature
    -   Change name `template update <TEMPLATE> --name <NEW_NAME>`
    -   Change description `template update <TEMPLATE> --description "<NEW_DESCRIPTION>"`
    -   Change location `template update <TEMPLATE> --location <NEW_LOCATION>`
-   Config feature
    -   Opens up the config directory

## Minor

-   Check user.name and user.email are set, error if not
-   Detect recursion and error if it happens

    -   Don't allow initialising a template inside itself
    -   If resolvesymlink, check for any copy loops before starting

-   Fail with error on init for missing template
