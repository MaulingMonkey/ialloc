#!/usr/bin/env bash

shopt -s globstar
wc -l {.github,.vscode,build,doc,examples,scripts,src}/**/*.* *.md Cargo.toml rust-toolchain.toml
