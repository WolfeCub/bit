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

## `bit`

**Usage:** `bit <COMMAND>`

###### **Subcommands:**

* `init` — 
* `cat-file` — 
* `hash-object` — 
* `log` — 
* `ls-tree` — 
* `write-tree` — 
* `show-ref` — 
* `tag` — 
* `rev-parse` — 
* `ls-files` — 
* `check-ignore` — 
* `rm` — 
* `add` — 
* `status` — 



## `bit init`

**Usage:** `bit init [PATH]`

###### **Arguments:**

* `<PATH>`



## `bit cat-file`

**Usage:** `bit cat-file <TYPE> <OBJECT>`

###### **Arguments:**

* `<TYPE>`
* `<OBJECT>`



## `bit hash-object`

**Usage:** `bit hash-object [OPTIONS] <PATH>`

###### **Arguments:**

* `<PATH>`

###### **Options:**

* `-t`, `--type <TYPE>`

  Default value: `blob`
* `-w`, `--write`

  Default value: `false`



## `bit log`

**Usage:** `bit log [COMMIT]`

###### **Arguments:**

* `<COMMIT>`



## `bit ls-tree`

**Usage:** `bit ls-tree <HASH>`

###### **Arguments:**

* `<HASH>`



## `bit write-tree`

**Usage:** `bit write-tree`



## `bit show-ref`

**Usage:** `bit show-ref`



## `bit tag`

**Usage:** `bit tag [OPTIONS] [NAME] [OBJECT]`

###### **Arguments:**

* `<NAME>`
* `<OBJECT>`

###### **Options:**

* `-a`

  Default value: `false`
* `-m`, `--message <MESSAGE>`



## `bit rev-parse`

**Usage:** `bit rev-parse [HASH_OR_REF]`

###### **Arguments:**

* `<HASH_OR_REF>`



## `bit ls-files`

**Usage:** `bit ls-files [OPTIONS]`

###### **Options:**

* `-v`, `--verbose` — This doesn't exist in actual git but it's useful for inspecting our index

  Default value: `false`



## `bit check-ignore`

**Usage:** `bit check-ignore <PATH>`

###### **Arguments:**

* `<PATH>`



## `bit rm`

**Usage:** `bit rm [PATHS]...`

###### **Arguments:**

* `<PATHS>`



## `bit add`

**Usage:** `bit add [PATHS]...`

###### **Arguments:**

* `<PATHS>`



## `bit status`

**Usage:** `bit status`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>


</details>
