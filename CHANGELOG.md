# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- [`#6`](https://github.com/Aloso/to-html/pull/6): Support different shells.

  All shells can be used that have a `-c <command>` argument to execute a command, and support multiple commands separated with `;`. This includes `bash`, `fish`, `ksh` and `zsh`.

- [`#5`](https://github.com/Aloso/to-html/pull/5): Add `--doc`/`-d` flag to emit a full HTML document. It can then be redirected to a file like this:

    ```shell
    $ to-html 'cargo test --workspace' -d > output.html
    $ firefox output.html
    ```

## [0.1.1]

- [`#4`](https://github.com/Aloso/to-html/pull/4): FreeBSD support added (this also fixed a few bugs on macOS)

## [0.1.0] - 2020-11-15

Initial release
