# Roadmap

Looking at the dependency graph, here's what I'd suggest:

**1. Config file** ✅
Almost everything else wants to read defaults from config. Do this first so no feature ever hardcodes a value that later needs to be made configurable.

**2. Tiny safety fixes**
Low-effort, no architectural impact: `user.name`/`user.email` check, recursion detection, missing template error. Clean these up while the codebase is still small.

**3. Registry schema expansion — all at once**
Descriptions, git URL fields (`url`, `cached`, `commit`), and hook fields (`pre_init`, `post_init`) all live on the template entry. Plan and implement the final schema shape in one go to avoid multiple migrations. Paired with the **change command** since it directly manipulates these fields.

**4. Git URL templates + update command**
These go together naturally — update only makes sense once remote templates exist. This significantly expands `git.rs` (clone, fetch, pull), so doing it as one chunk keeps that module coherent.

**5. Enhanced list output**
By this point, all the data it needs exists: git status, git URL info, descriptions, symlink/file type detection. Probably warrants splitting display logic into its own module rather than keeping it in `ops.rs`.

**6. Hooks**
Pre/post-init. The registry fields are already defined (step 3), config defaults already exist (step 1), so this is just wiring execution into `cmd_init`.

**7. `fs_copy.rs` overhaul**
Configurable exclusions, symlink support, and overwrite behavior all touch the same module. Doing them together avoids three separate refactors of the same code.

**8. No-colour, `--git`/`--no-git`, `--names-only`, prune, autocomplete**
Polish and convenience. These are all self-contained and safe to do in any order at the end.

---

The key architectural principle: **schema changes first, features that consume that schema second**. The other risk to avoid is touching `fs_copy.rs` multiple times — grouping all its complexity into one refactor is cleaner.
