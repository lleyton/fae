# Fae: The Simple Task Runner

## Why?

I personally love NPM scripts, it allows for the automation of mundane tasks in a simple way. However, NPM scripts aren't perfect. Besides being specific to node, parallel execution of tasks isn't straightforward. Fae is an attempt to build a task runner that is:

1. compatable with NPM scripts
2. language agnostic
3. fast, with ✨ parallelism ✨

## How do I use it?

Simpily run `fae` with the name of your task, like `fae build`. Fae will search your fae.toml and package.json files for your tasks. If a task cannot be found, fae will try to search for binaries in the node_modules folder if it is present.

## Status

Although Fae is still a work in progress, I am currently using it for my own projects and I would really appreciate it if you gave it a try. Fae is currently a bodge/prototype of messy code that I wrote in a night, but I'll be working to clean it up. If you have any issues or feature suggestions, feel free to file a GitHub issue.

## Installation

I currently don't have any prebuilt binaries for Fae, although this is a priority for me. For now, you can run `cargo install --git https://github.com/lleyton/fae`, which will build the package and install it to your cargo bin.

## Configuration

The Fae configuration file (fae.toml) is simpily a map between task names and an object. The object can have the following keys:

- uses: `Option<Vec<String>>`, specifies dependencies (tasks) to execute, these tasks will be executed in parallel
- run: `Option<String>`, the shell command to run
- stdout: `Option<bool>`, whether to pipe stdout, true by default
- stderr: `Option<bool>`, whether to pipe forward stderr, true by default
- cache: `Option<bool>`, "cache" runs of this task, useful when a depdencies use the same task, true by default; please note that the cache is not shared between subsequent executions of Fae

```toml
[tsc]
run = "tsc"

[lint]
run = "eslint"

[check]
uses = ["tsc", "lint"]
run = "echo LGTM!"
```

## What does "Fae" mean?

Nothing, I just like it :3

Made with <3 by a sleep deprived catgirl
