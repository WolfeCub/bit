# bit (baby git)

> Just a bit of git

A toy git implementation in rust. This is a work in progress and commands are being added one by one.
For a full breakdown of implemented commands and features see the [generated cli documentation](#cli-documentation).

## Goals
- Learn more about how git works under the hood
- 100% (or as close as possible) compatible with git repositories

## Non-goals
- Replace git
- Production usage

## CLI Documentation

<details>

<summary>Expand generated CLI documatation</summary>

# Command-Line Help for `bit`

This document contains the help content for the `bit` command-line program.

**Command Overview:**

* [`bit`↴](#bit)
* [`bit init`↴](#bit-init)
* [`bit cat-file`↴](#bit-cat-file)
* [`bit hash-object`↴](#bit-hash-object)
* [`bit log`↴](#bit-log)
* [`bit ls-tree`↴](#bit-ls-tree)
* [`bit write-tree`↴](#bit-write-tree)
* [`bit show-ref`↴](#bit-show-ref)
* [`bit tag`↴](#bit-tag)
* [`bit rev-parse`↴](#bit-rev-parse)
* [`bit ls-files`↴](#bit-ls-files)
* [`bit check-ignore`↴](#bit-check-ignore)
* [`bit rm`↴](#bit-rm)
* [`bit add`↴](#bit-add)
* [`bit status`↴](#bit-status)
* [`bit commit`↴](#bit-commit)
* [`bit read-tree`↴](#bit-read-tree)
* [`bit branch`↴](#bit-branch)
* [`bit switch`↴](#bit-switch)
* [`bit restore`↴](#bit-restore)

## `bit`

**Usage:** `bit <COMMAND>`

###### **Subcommands:**

* `init` — Initializes a new bit repository
* `cat-file` — Displays the contents of a bit object
* `hash-object` — Creates a bit object from a file on disk, and optionally writes it to the object database
* `log` — Shows the commit history starting from a given commit (defaulting to HEAD)
* `ls-tree` — Displays the contents of a tree object
* `write-tree` — Writes the current index to a tree object and prints it's hash
* `show-ref` — Prints the hash and name of passed refs or all refs if none were passed
* `tag` — Creates a new tag or lists existing tags if no name is provided
* `rev-parse` — Prints the hash of the passed ref or hash
* `ls-files` — Prints the list of files in the index
* `check-ignore` — Checks if the passed path is ignored by .bitignore and prints it if it is
* `rm` — Remove a file from the index and delete it from the filesystem
* `add` — Adds a file to the index (creating a blob object for it)
* `status` — Shows the current branch, staged changes, unstaged changes and untracked files
* `commit` — Creates a new commit with the current index as the tree, and HEAD as the parent
* `read-tree` — Extract the contents of a tree object into the working directory
* `branch` — Creates, lists, deletes, or moves branches
* `switch` — Switch between branches
* `restore` — Discards either working or staged changes for the given files



## `bit init`

Initializes a new bit repository

**Usage:** `bit init [PATH]`

###### **Arguments:**

* `<PATH>`



## `bit cat-file`

Displays the contents of a bit object

**Usage:** `bit cat-file <TYPE> <OBJECT>`

###### **Arguments:**

* `<TYPE>`
* `<OBJECT>`



## `bit hash-object`

Creates a bit object from a file on disk, and optionally writes it to the object database

**Usage:** `bit hash-object [OPTIONS] <PATH>`

###### **Arguments:**

* `<PATH>`

###### **Options:**

* `-t`, `--type <TYPE>`

  Default value: `blob`
* `-w`, `--write`



## `bit log`

Shows the commit history starting from a given commit (defaulting to HEAD)

**Usage:** `bit log [COMMIT]`

###### **Arguments:**

* `<COMMIT>`



## `bit ls-tree`

Displays the contents of a tree object

**Usage:** `bit ls-tree <HASH>`

###### **Arguments:**

* `<HASH>`



## `bit write-tree`

Writes the current index to a tree object and prints it's hash

**Usage:** `bit write-tree`



## `bit show-ref`

Prints the hash and name of passed refs or all refs if none were passed

**Usage:** `bit show-ref`



## `bit tag`

Creates a new tag or lists existing tags if no name is provided

**Usage:** `bit tag [OPTIONS] [NAME] [OBJECT]`

###### **Arguments:**

* `<NAME>`
* `<OBJECT>`

###### **Options:**

* `-a` — Create a tag object instead of a lightweight tag. This now also requires a message to be provided
* `-m`, `--message <MESSAGE>`



## `bit rev-parse`

Prints the hash of the passed ref or hash

**Usage:** `bit rev-parse [HASH_OR_REF]`

###### **Arguments:**

* `<HASH_OR_REF>`



## `bit ls-files`

Prints the list of files in the index

**Usage:** `bit ls-files [OPTIONS]`

###### **Options:**

* `-v`, `--verbose` — This doesn't exist in actual git but it's useful for inspecting our index



## `bit check-ignore`

Checks if the passed path is ignored by .bitignore and prints it if it is

**Usage:** `bit check-ignore <PATH>`

###### **Arguments:**

* `<PATH>`



## `bit rm`

Remove a file from the index and delete it from the filesystem

**Usage:** `bit rm [OPTIONS] [PATHS]...`

###### **Arguments:**

* `<PATHS>`

###### **Options:**

* `-r`



## `bit add`

Adds a file to the index (creating a blob object for it)

**Usage:** `bit add [PATHS]...`

###### **Arguments:**

* `<PATHS>`



## `bit status`

Shows the current branch, staged changes, unstaged changes and untracked files

**Usage:** `bit status`



## `bit commit`

Creates a new commit with the current index as the tree, and HEAD as the parent

**Usage:** `bit commit [OPTIONS]`

###### **Options:**

* `-m`, `--message <MESSAGE>`



## `bit read-tree`

Extract the contents of a tree object into the working directory

**Usage:** `bit read-tree [OPTIONS] <TREE>`

###### **Arguments:**

* `<TREE>`

###### **Options:**

* `-u`
* `-m`



## `bit branch`

Creates, lists, deletes, or moves branches

**Usage:** `bit branch [OPTIONS] [BRANCH] [SECOND_ARG]`

###### **Arguments:**

* `<BRANCH>`
* `<SECOND_ARG>`

###### **Options:**

* `--force`
* `-d`, `--delete`
* `-m`, `--move`



## `bit switch`

Switch between branches

**Usage:** `bit switch [OPTIONS] <BRANCH>`

###### **Arguments:**

* `<BRANCH>`

###### **Options:**

* `-c`, `--create`



## `bit restore`

Discards either working or staged changes for the given files

**Usage:** `bit restore [OPTIONS] [FILES]...`

###### **Arguments:**

* `<FILES>`

###### **Options:**

* `-W`, `--worktree`
* `-C`, `--staged`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>


</details>
