# `cargo-restore`

A cargo subcommand to automatically install all packages listed in a `.crates2.json` file (normally located in `~/.cargo`)

## Usage

```
Usage: cargo-restore [OPTIONS] [COMMAND]

Commands:
  restore
  help     Print this message or the help of the given subcommand(s)

Options:
  -c, --crate-file <CRATES_FILE>  The file to parse as the .crates2.json if you don't want to use the default one
  -t, --fix-target                Change the target of all to-be-installed packges to match this device's target if it's
                                  different
  -i, --install-latest            Whether to install the latest version available (true) or install the version in the
                                  lockfile (false, default)
  -f, --force-all                 Whether to force (re)installation of al packages listed in .crates2.json, even if we can
                                  detect that all their binaries are already correctly installed
  -v, --verbose                   Whether to enable verbose output
  -q, --quick-fail                If true, `cargo-restore` will exit with an error code as soon as any package fails to
                                  install
  -h, --help                      Print help
  -V, --version                   Print version
```

When invoked without arguments, `cargo-restore` will automatically read all packages from `~/.cargo/.crates2.json` (or wherever else your cargo config defines to be root) and attempt to install them all. If any fail to install, it will print an error message and continue with the next on the list.

## Installation
```bash
cargo install --git https://github.com/itsjunetime/cargo-restore.git
```

## Caveats
This is designed to be used with another dotfile-managing system, such as yadm, so that you could simply run `cargo install cargo-restore && cargo restore` to install all packages you want from cargo on your system. This requires reading from `~/.cargo/.crates2.json` by default, but that can lead to some confusion due to the fact that `cargo-restore` will, as a side-effect of using cargo to invoke installs, overwrite this file after each install.

This means that if you invoke `cargo restore`, then a crate fails to build, invoking `cargo restore` again may inform you that all crates are installed correctly, since `~/.cargo/.crates2.json` was overwritten by the installation failure to not include that crate.

Due to this issue, `cargo-restore` supports reading from a different file from `~/.cargo/.crates2.json` (but which must still be in the same format) to generate the package list to install. One can then use other tools at their disposal to keep this other file in-sync with cargo's `.crates2.json` upon successful installations so that dotfile syncing of cargo packages doesn't need to incur extra overhead.
