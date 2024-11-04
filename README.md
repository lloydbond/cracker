# cracker
A rusty ui for exploring and starting tasks. Of course, blazingly fast.

## Requirements

* [rust-lang](https://www.rust-lang.org/) >= 1.81.0

Be sure to add $HOME/.cargo/bin/ to your environment PATH variable.
## Supports:

-   Linux
-  *macOS
-  *Windows Subsystem for Linux (WSL)

\* untested, should work.

## Manual Installation

Clone the repository:

```bash
git clone git@github.com:lloydbond/cracker.git
cd cracker
cargo install --path .

```

## Usage

```bash
  cd /path/to/Makefile
  ck
```

## TODO:
- [ ] Asynchronously open Makefile
- [x] handle multi-target Makefile rules
- [ ] support commands with streaming data
  - [ ] example: tail -f /var/log/dmesg.log
- [ ] Support additional task runner type build scripts
  - [ ] npm
  - [ ] grunt
  - [ ] taskpy
  - [ ] etc.
- [x] switch makfile-lossless to PEG for rule target detecion
- [ ] seperate task runner support to library
- [ ] cracker-tui
- [ ] hx/vi compatible keymapping
- [ ] add to crates.io
- [x] CICD


## Motivation

* Quick and easy execution and monitoring of Makefile and other types of runners for your local project.
