# cracker
A rusty ui for exploring and starting tasks. Of course, blazingly fast.

- [Requirements](#requirements)
- [Supports](#supports)
- [Installation Methods](#installation-methods)
  - [Install with cargo](#install-with-cargo)
  - [Manual Installation](#manual-installation)
- [Usage](#usage)
  - [Enable Log Messages](#enable-log-messages)
- [TODO](#todo)
- [Motivation](#motivation)
- [Who is this For](#who-is-this-for)

## Requirements

* [rust-lang](https://www.rust-lang.org/) >= 1.81.0

Be sure to add $HOME/.cargo/bin/ to your environment PATH variable.
## Supports

-   Linux
-  *macOS
-  *Windows Subsystem for Linux (WSL)

\* untested, should work.
## Installation Methods
### Install with cargo
If you haven't isntalled `cargo` yet, now's a good time.
Follow the cargo install instructions [here](https://doc.rust-lang.org/cargo/getting-started/installation.html)
Then run from your terminal.
```bash
cargo install ck-cracker
```

### Manual Installation

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

### Enable Log messages
Log messages are limited for as the tool reaches a 1.0 release.
```bash
  RUST_LOG=ck=[warn|info|error|debug] ck

  RUST_LOG=ck=debug ck
```
## TODO:
- [x] Asynchronously open Makefile
- [x] handle multi-target Makefile rules
- [x] support commands with streaming data
  - [x] example: tail -f /var/log/dmesg.log
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

### Who is this For
Quick and easy execution and monitoring of Makefile and other types of runners for your local project.
When you need to give a laymon commands to run and they are averse to typing in a terminal.

### ***Note from the maintainer***

I've been using this project as a means to learn Rust. Expect improvments and additions as I have time. Any improvements, code style, or feed back will go along way for
this project. Any contributions or additions to the supported Task runners as well as UI beautifcations or simplifications
are greatly welcomed and appreciated. Happy Coding!

