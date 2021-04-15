# reaper-rs

[![Continuous integration](https://github.com/helgoboss/reaper-rs/workflows/Windows/badge.svg)](https://github.com/helgoboss/reaper-rs/actions)
[![Continuous integration](https://github.com/helgoboss/reaper-rs/workflows/macOS/badge.svg)](https://github.com/helgoboss/reaper-rs/actions)
[![Continuous integration](https://github.com/helgoboss/reaper-rs/workflows/Linux/badge.svg)](https://github.com/helgoboss/reaper-rs/actions)
[![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/helgoboss/reaper-rs/master/LICENSE)

[Rust](https://www.rust-lang.org/) bindings for the [REAPER](https://www.reaper.fm/) C++ API.

**Important note:** If you want to use _reaper-rs_ for your own project, please use the master branch for the time
being, not the crates on [crates.io](https://crates.io/)! I push changes here pretty often but I don't publish to
crates.io at the moment, so my crates there are a bit outdated. Rationale: As long as I'm the only consumer of this
library, this process is easier for me. I tend to keep `reaper-low` and `reaper-medium` mostly stable, so no worries
about that :)

Here's the snippet:

```ignore
reaper-medium = { git = "https://github.com/helgoboss/reaper-rs.git", branch = "master" }
reaper-low = { git = "https://github.com/helgoboss/reaper-rs.git", branch = "master" }
reaper-macros = { git = "https://github.com/helgoboss/reaper-rs.git", branch = "master" }
```

## Table of Contents

- [Introduction](#introduction)
- [Basics](#basics)
- [Usage](#usage)
- [Contribution](#contribution)
- [Background](#background)

## Introduction

_reaper-rs_ allows programmers to write plug-ins for the [REAPER](https://www.reaper.fm/) DAW
(digital audio workstation) in the [Rust](https://www.rust-lang.org/) programming
language. It does so by providing raw Rust bindings for the
[REAPER C++ API](https://www.reaper.fm/sdk/plugin/plugin.php) and more convenient APIs on top of that.
It also exposes the [SWELL C++ API](https://www.cockos.com/wdl/), which is provided by
REAPER on Linux and macOS in order to enable developers to create cross-platform user interfaces with a subset of the
Win32 API.

## Basics

_reaper-rs_ consists of the following production crates:

- [reaper-macros](https://docs.rs/reaper-macros)
- [reaper-low](https://docs.rs/reaper-low)
- [reaper-medium](https://docs.rs/reaper-medium)
- `reaper-high` (not yet published)
- `reaper-rx` (not yet published)

`reaper-macros` provides a simple attribute macro to simplify bootstrapping REAPER extension plug-ins.

`reaper-low`, `reaper-medium` and `reaper-high` represent the 3 different APIs of _reaper-rs_

The remaining crates are add-ons for the high-level API.

### 1. Low-level API

[![Latest Version](https://img.shields.io/crates/v/reaper-low.svg)](https://docs.rs/reaper-low)
[![documentation](https://docs.rs/reaper-low/badge.svg)](https://docs.rs/reaper-low)

This API contains the raw bindings, nothing more. It's unsafe to a large extent and not intended to be used
directly. However, it serves as foundation for all the other APIs and is easy to keep up-to-date because it's
mostly auto-generated from `reaper_plugin_functions.h`. It also can serve as last resort if a function has not
yet been implemented in the medium-level API (although I rather want encourage to contribute to the medium-level API
in such a case).

Status:

- ![](https://via.placeholder.com/30/aed581/000000?text=+) **crates.io**: published
- ![](https://via.placeholder.com/30/ffd54f/000000?text=+) **API stability**: approaching stable (quite polished already, breaking changes still possible)
- ![](https://via.placeholder.com/30/aed581/000000?text=+) **Completion**: ~95% (some virtual function calls still missing)

Example:

```rust,ignore
unsafe {
    reaper.ShowConsoleMsg(c_str!("Hello world from reaper-rs low-level API!").as_ptr());
    let track = reaper.GetTrack(null_mut(), 0);
    reaper.DeleteTrack(track);
}
```

### 2. Medium-level API

[![Latest Version](https://img.shields.io/crates/v/reaper-medium.svg)](https://docs.rs/reaper-medium)
[![documentation](https://docs.rs/reaper-medium/badge.svg)](https://docs.rs/reaper-medium)

This API builds on top of the low-level API. It exposes the original REAPER C++ API functions almost
one to one, but in an idiomatic and type-safe way. It's a big step forward from the raw bindings
and far more convenient to use. Its focus is on stability rather than exploring new paradigms.
Since the high-level API is still very unstable, _this is the recommended API_.

Status:

- ![](https://via.placeholder.com/30/aed581/000000?text=+) **crates.io**: published
- ![](https://via.placeholder.com/30/ffd54f/000000?text=+) **API stability**: approaching stable (quite polished already, breaking changes still possible)
- ![](https://via.placeholder.com/30/ff8a65/000000?text=+) **Completion**: ~13% (solid foundation, roughly 100 of 800 functions implemented)

#### Examples

Basics:
```rust,ignore
reaper.show_console_msg("Hello world from reaper-rs medium-level API!");
let track = reaper.get_track(CurrentProject, 0).ok_or("no tracks")?;
unsafe { reaper.delete_track(track); }
```

Control surface:
```rust,ignore
#[derive(Debug)]
struct MyControlSurface;

impl ControlSurface for MyControlSurface {
    fn set_track_list_change(&self) {
        println!("Tracks changed");
    }
}

session.plugin_register_add_csurf_inst(MyControlSurface);
```

Audio hook:

```rust,ignore
struct MyOnAudioBuffer {
    counter: u64
}

impl OnAudioBuffer for MyOnAudioBuffer {
    fn call(&mut self, args: OnAudioBufferArgs) {
        if self.counter % 100 == 0 {
            println!("Audio hook callback counter: {}\n", self.counter);
        }
        self.counter += 1;
    }
}

session.audio_reg_hardware_hook_add(MyOnAudioBuffer { counter: 0 });
```

### 3. High-level API

This API builds on top of the medium-level API. It makes a break with the "flat functions" nature of the original
REAPER C++ API and replaces it with an API that uses object-oriented paradigms. This break makes it possible to provide
an intuitive API which can be used completely without `unsafe`.

Status:

- ![](https://via.placeholder.com/30/ff8a65/000000?text=+) **crates.io**: not published
- ![](https://via.placeholder.com/30/ff8a65/000000?text=+) **API stability**: unstable (in a state of flux, but working)
- ![](https://via.placeholder.com/30/ff8a65/000000?text=+) **Completion**: ~13% (roughly on par with the medium-level API)

Example:

```rust,ignore
reaper.show_console_msg("Hello world from reaper-rs high-level API!");
let project = reaper.current_project();
let track = project.track_by_index(0).ok_or("no tracks")?;
project.remove_track(&track);
```

#### Reactive extensions

`reaper-rx` adds reactive programming via [rxRust](https://github.com/rxRust/rxRust) to the mix.

Example:

```rust,ignore
rx.track_removed().subscribe(|t| println!("Track {:?} removed", t));
```

## Usage

The procedure depends on the desired _type_ of plug-in.
In addition to writing REAPER extension plug-ins, _reaper-rs_ can be used for developing VST plug-ins
that use REAPER functions. No matter what you choose, the possibilities of interacting with REAPER are
essentially the same. The difference between the two is the context in which your plug-in will run.

An extension plug-in is loaded when REAPER starts and remains active until REAPER quits, so it's
perfectly suited to add some functions to REAPER which should be available globally. Popular examples are
[SWS](https://www.sws-extension.org/) and [ReaPack](https://reapack.com/) (both written in C++).

A REAPER VST plug-in is loaded as track, take or monitoring FX as part of a particular REAPER project,
just like any instrument or effect plug-in out there. That also means it can be instantiated multiple
times. Examples are [Playtime](https://www.helgoboss.org/projects/playtime/) (written in C++) and
[ReaLearn](https://www.helgoboss.org/projects/realearn/) (successfully ported to Rust).

In both cases you need to make a library crate of type `cdylib`.

### REAPER extension plug-in

Using the `reaper_extension_plugin` macro is the fastest way to get going.

Add this to your `Cargo.toml`:

```toml
[dependencies]
reaper-low = "0.1.0"
reaper-medium = "0.1.0"
reaper-macros = "0.1.0"

[lib]
name = "reaper_my_extension"
crate-type = ["cdylib"]
```

Then in your `lib.rs`:

```rust
use std::error::Error;
use reaper_macros::reaper_extension_plugin;
use reaper_low::PluginContext;
use reaper_medium::ReaperSession;

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    let session = ReaperSession::load(context);
    session.reaper().show_console_msg("Hello world from reaper-rs medium-level API!");
    Ok(())
}
```

> **Important:** Compiled REAPER extension plug-ins (i.e. `.dll` files) must be prefixed with `reaper_` in order for REAPER to load them during startup - even on Linux and macOS, where library file names usually start with `lib`. On Windows, it's enough to name the library `reaper_my_extension` in `Cargo.toml` and it will result in the compiled file being named `reaper_my_extension`, thus obeying this rule. On Linux and macOS, you still need to remove the `lib` prefix. In any case, make sure that the compiled file placed in `REAPER_RESOURCE_PATH/UserPlugins` is prefixed with `reaper_` before attempting to test it!

The macro primarily exposes an `extern "C" ReaperPluginEntry()` function which calls
`reaper_low::bootstrap_extension_plugin()`. So if for some reason you don't want to use that
macro, have a look at the macro implementation. No magic there.

#### Step-by-step instructions

The following instructions should result in a functional extension, loaded into REAPER on start:

1. Run `cargo new reaper-my-extension --lib` to initialize the project
2. Run `cargo build` from within `reaper-my-extension` to generate the compiled plugin extension inside of the `target/debug` directory
3. Copy the extension plug-in to the `REAPER/UserPlugins` directory
    - You could do this manually, and overwrite the file after each build
    - Or, you could create a symbolic link from the `target/debug` file, to `REAPER/UserPlugins` so that they were synced
        - > Note: Here it's explicitly necessary to give the link a name that starts with `reaper_` (by default it will start with `lib`)
        - To do this, on unix-based systems, run `ln -s ./target/debug/<name-of-the-compiled-extension-file> <path to REAPER/UserPlugins>`
        - On Windows, you can use the same command if running Git Bash, else you can use `mklink \D target\debug\<name-of-the-compiled-extension-file> %AppData%\REAPER\UserPlugins`
4. Now start REAPER, and you should see the console message from the code appear!


### REAPER VST plug-in

A REAPER VST plug-in is nothing else than a normal VST plug-in which gets access to functions from the REAPER C++ API.
Luckily, there is a Rust crate for creating VST plug-ins already: [vst-rs](https://docs.rs/vst).
So all you need to do is write a VST plug-in via _vst-rs_ and gain access to the REAPER functions by letting
_reaper-rs_ access the `HostCallback` function.

Add this to your `Cargo.toml`:

```toml
[dependencies]
reaper-low = "0.1.0"
reaper-medium = "0.1.0"
vst = "0.2.0"

[lib]
name = "my_reaper_vst_plugin"
crate-type = ["cdylib"]
```

Then in your `lib.rs`:

```rust
use vst::plugin::{Info, Plugin, HostCallback};
use reaper_low::{PluginContext, reaper_vst_plugin, static_vst_plugin_context};
use reaper_medium::ReaperSession;

reaper_vst_plugin!();

#[derive(Default)]
struct MyReaperVstPlugin {
    host: HostCallback,
};

impl Plugin for MyReaperVstPlugin {
    fn new(host: HostCallback) -> Self {
        Self { host }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "My REAPER VST plug-in".to_string(),
            unique_id: 6830,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        if let Ok(context) = PluginContext::from_vst_plugin(&self.host, static_vst_plugin_context()) {
            let session = ReaperSession::load(context);
            session
                .reaper()
                .show_console_msg("Hello world from reaper-rs medium-level API!");
        }
    }
}

vst::plugin_main!(MyReaperVstPlugin);
```

## Contribution

Contributions are very welcome! Especially to the medium-level API.

### Directory structure

| Directory entry               | Content                                                 |
| ----------------------------- | ------------------------------------------------------- |
| `/`                           | Workspace root                                          |
| `/main`                       | Production code                                         |
| `/main/high`                  | High-level API (`reaper-high`)                          |
| `/main/low`                   | Low-level API (`reaper-low`)                            |
| `/main/macros`                | Macros (`reaper-macros`)                                |
| `/main/medium`                | Medium-level API (`reaper-medium`)                      |
| `/main/rx`                    | rxRust integration for high-level API (`reaper-rx`)     |
| `/test`                       | Integration test code                                   |
| `/test/test`                  | Integration test logic (`reaper-test`)                  |
| `/test/test-extension-plugin` | Test extension plug-in (`reaper-test-extension-plugin`) |
| `/test/test-vst-plugin`       | Test VST plug-in (`reaper-test-vst-plugin`)             |

### Low-level API code generation

`reaper-low` has several generated files, namely `bindings.rs`, `reaper.rs` and `swell.rs`.
These files are not generated with each build though. In order to decrease build time and improve
IDE/debugging support, they are included in the Git repository like any other Rust source.

You can generate these files on demand (see build section), e.g. after you have adjusted
`reaper_plugin_functions.h`. Right now this is enabled for Linux and macOS only. If we would generate the files on
Windows, `bindings.rs` would look quite differently (whereas `reaper.rs` should end up the
same). The reason is that `reaper_plugin.h` includes `windows.h` on Windows, whereas on Linux and macOS, it uses
`swell.h` ([Simple Windows Emulation Layer](https://www.cockos.com/wdl/)) as a replacement.

Most parts of `bindings.rs` are used to generate `reaper.rs` and otherwise ignored, but a few
structs, types and constants are published as part of the `raw` module. In order to have
deterministic builds, for now the convention is to only commit files generated on Linux.
Rationale: `swell.h` is a sort of subset of `windows.h`, so if things work
with the subset, they also should work for the superset. The inverse isn't true.

### Build

Thanks to Cargo, building _reaper-rs_ is not a big deal.

#### Windows

In the following you will find the complete instructions for Windows 10, including Rust setup. Points where you have to consider the target
architecture (REAPER 32-bit vs. 64-bit) are marked with :star:.

1. Setup "Build tools for Visual Studio 2019"
    - Rust uses native build toolchains. On Windows, it's necessary to use the MSVC (Microsoft Visual Studio
      C++) toolchain because REAPER plug-ins only work with that.
    - [Visual Studio downloads](https://visualstudio.microsoft.com/downloads/) → All downloads → Tools for Visual Studio 2019
      → Build Tools for Visual Studio 2019
    - Start it and follow the installer instructions
    - Required components
        - Workloads tab
            - "C++ build tools" (large box on the left)
            - Make sure "Windows 10 SDK" is checked on the right side (usually it is)
        - Language packs
            - English
2. Setup Rust
    - [Download](https://www.rust-lang.org/tools/install) and execute `rustup-init.exe`
    - Accept the defaults
    - Set the correct toolchain default (_nightly_ toolchain is not necessary if you only want to build
      `reaper-low`, `reaper-medium` and `reaper-high`) :star:
      ```batch
      rustup default nightly-2020-12-10-x86_64-pc-windows-msvc
      ```
3. Download and install [Git for Windows](https://git-scm.com/download/win)
4. Clone the _reaper-rs_ Git repository
   ```batch
   git clone --recurse-submodules https://github.com/helgoboss/reaper-rs.git`
   cd reaper-rs
   ```
5. Build _reaper-rs_
   ```batch
   cargo build
   ```

Regenerating the low-level API from Windows is disabled for now.

#### Linux

Complete instructions to build _reaper-rs_ from a _fresh_ Ubuntu 18.04.3 LTS installation,
including Rust setup:

```sh
# Install basic stuff
sudo apt update
sudo apt install curl git build-essential pkg-config libssl-dev liblzma-dev llvm-dev libclang-dev clang -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # choose 1 (default)
source $HOME/.cargo/env
# Using nightly is not necessary if you want to build just the low-level, medium-level or high-level API!
rustup default nightly-2020-12-10-x86_64-unknown-linux-gnu

# Clone reaper-rs
cd Downloads
git clone --recurse-submodules https://github.com/helgoboss/reaper-rs.git
cd reaper-rs

# Build reaper-rs
cargo build
```

Make the test plug-ins available in REAPER:

1. Download REAPER for Linux and start it at least one time.
2. Create symbolic links
   ```sh
   mkdir -p $HOME/.config/REAPER/UserPlugins/FX
   ln -s $HOME/Downloads/reaper-rs/target/debug/libreaper_test_extension_plugin.so $HOME/.config/REAPER/UserPlugins/reaper_test_extension_plugin.so
   ln -s $HOME/Downloads/reaper-rs/target/debug/libreaper_test_vst_plugin.so $HOME/.config/REAPER/UserPlugins/FX/reaper_test_vst_plugin.so
   ```

Regenerate the low-level API:

```sh
cd main/low
cargo build --features generate
cargo fmt
```

#### macOS

The following instructions include Rust setup. However, it's very well possible that some native toolchain setup
instructions are missing, because I don't have a bare macOS installation at my disposal. The Rust installation script
should provide you with the necessary instructions if something is missing.

```sh
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # choose 1 (default)
source $HOME/.cargo/env
# Using nightly is not necessary if you want to build just the low-level, medium-level or high-level API!
rustup default nightly-2020-12-10-x86_64-apple-darwin

# Clone reaper-rs
cd Downloads
git clone --recurse-submodules https://github.com/helgoboss/reaper-rs.git
cd reaper-rs

# Build reaper-rs
cargo build
```

### Test

When building the complete _reaper-rs_ workspace, 3 test crates are produced:

- `reaper-test`
- `reaper-test-extension-plugin`
- `reaper-test-vst-plugin`

`reaper-test` provides an integration test that is supposed to be run in REAPER itself. This is the main testing
mechanism for _reaper-rs_. `reaper-test-extension-plugin` and `reaper-test-vst-plugin` are both test plug-ins
which register the integration test as REAPER action.

Running the integration test is not only a good way to find _reaper-rs_ regression bugs, but can also help to expose
subtle changes in the REAPER C++ API itself. Currently, the test assertions are very strict in order to reveal even
the slightest deviations.

**Attention:** The test should be executed using a fresh `reaper.ini`. Some assertions assume that REAPER
preferences are set to their defaults. Executing the test with modified preferences can lead to wrong test results!

On Linux and macOS, the REAPER integration test will be run automatically as Cargo integration test
`run_reaper_integration_test` when invoking `cargo test` (downloads, unpacks and executes REAPER). This test is part of
`reaper-test-extension-plugin`. It can be disabled by building that crate with `--no-default-features`.

`reaper-test` activates the performance measurement features of `reaper-medium` and `reaper-high`. At the end of an
integration test run, it prints detailed response time statistics to standard output.

## Background

_reaper-rs_ has been born as part of the effort of porting the REAPER VST plug-in
[ReaLearn](https://www.helgoboss.org/projects/realearn/) to Rust and publish it as open-source project. The high-level
API is heavily inspired by ReaPlus, a C++ facade for the native REAPER C++ API, which is a basic
building block of the original ReaLearn.
