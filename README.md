# reaper-rs

[REAPER](https://www.reaper.fm/) bindings for the [Rust](https://www.rust-lang.org/) programming language

[![Latest Version](https://img.shields.io/crates/v/reaper-rs.svg)](https://crates.io/crates/reaper-rs)
[![documentation](https://docs.rs/reaper-rs/badge.svg)](https://docs.rs/reaper-rs)
[![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/helgoboss/reaper-rs/master/LICENSE)

## Introduction

*reaper-rs* allows programmers to write plug-ins for the [REAPER](https://www.reaper.fm/) DAW 
(digital audio workstation) in the  [Rust](https://www.rust-lang.org/) programming 
language. It does so by providing raw Rust bindings to the 
[REAPER C++ SDK](https://www.reaper.fm/sdk/plugin/plugin.php) and more convenient APIs on top of that.

## Basics

*reaper-rs* consists of 4 production crates:

- [reaper_rs_macros](https://crates.io/crates/reaper_rs_macros)
- [reaper_rs_low](https://crates.io/crates/reaper_rs_low)
- [reaper_rs_medium](https://crates.io/crates/reaper_rs_medium)
- [reaper_rs_high](https://crates.io/crates/reaper_rs_high)

`reaper_rs_macros` provides a simple attribute macro to simplify bootstrapping REAPER extension plug-ins.

The remaining 3 crates represent the 3 different APIs of *reaper-rs*.

### 1. Low-level API

This API contains the raw bindings, nothing more. It's unsafe to a large extent and not intended to be used 
directly. However, it serves as foundation for all the other APIs and is easy to keep up-to-date because it's 
mostly auto-generated from `reaper_plugin_functions.h`. It also can serve as last resort if a function has not
yet been implemented in the medium-level API (although I rather want encourage to contribute to the medium-level API
in such a case). 

Status:

- ![](https://via.placeholder.com/30/aed581/000000?text=+) **crates.io**: published
- ![](https://via.placeholder.com/30/ffd54f/000000?text=+) **API stability**: nearly stable (quite polished already, breaking changes possible but not planned)
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

This API builds on top of the low-level API. It exposes the original REAPER SDK functions almost
one to one, but in an idiomatic and type-safe way. It's a big step forward from the raw bindings
and far more convenient to use. Its focus is on stability rather than exploring new paradigms.
Since the high-level API is still very unstable, *this is the recommended API*.

Status:

- ![](https://via.placeholder.com/30/aed581/000000?text=+) **crates.io**: published
- ![](https://via.placeholder.com/30/ffd54f/000000?text=+) **API stability**: nearly stable (quite polished already, breaking changes possible but not planned)
- ![](https://via.placeholder.com/30/ff8a65/000000?text=+) **Completion**: ~13% (solid foundation, roughly 100 of 800 functions implemented)

Example:

```rust,ignore
let functions = reaper.functions();
functions.show_console_msg("Hello world from reaper-rs medium-level API!");
let track = functions.get_track(CurrentProject, 0).ok_or("no tracks")?;
unsafe { functions.delete_track(track); }
```
   
### 3. High-level API

This API builds on top of the medium-level API. It makes a break with the "flat functions" nature of the original 
REAPER SDK and replaces it with an API that uses reactive and object-oriented paradigms. This break makes it
possible to provide a very intuitive API which can be used completely without `unsafe`. 

Status:

- ![](https://via.placeholder.com/30/ff8a65/000000?text=+) **crates.io**: not published
- ![](https://via.placeholder.com/30/ff8a65/000000?text=+) **API stability**: unstable (in a state of flux, but working)
- ![](https://via.placeholder.com/30/ff8a65/000000?text=+) **Completion**: ~13% (roughly on par with the medium-level API)

Example:

```rust,ignore
reaper.show_console_msg("Hello world from reaper-rs high-level API!");
reaper.track_removed().subscribe(|t| println!("Track {:?} removed", t));
let project = reaper.get_current_project();
let track = project.get_track_by_index(0).ok_or("no tracks")?;
project.remove_track(&track);
```

## Usage

The procedure depends on the desired *type* of plug-in. 
In addition to writing REAPER extension plug-ins, *reaper-rs* can be used for developing VST plug-ins 
that use REAPER functions. No matter what you choose, the possibilities of interacting with REAPER are 
essentially the same. The difference between the two is the context in which your plug-in will run.

An extension plug-in is loaded when REAPER starts and remains active until REAPER quits, so it's 
perfectly suited to add some functions to REAPER which should be available globally. Popular examples are 
[SWS](https://www.sws-extension.org/) and [ReaPack](https://reapack.com/) (both written in C++).

A REAPER VST plug-in is loaded as track, take or monitoring FX as part of a particular REAPER project, 
just like any instrument or effect plug-in out there. That also means it can be instantiated multiple 
times. Examples are [Playtime](https://www.helgoboss.org/projects/playtime/) (written in C++) and 
[ReaLearn](https://www.helgoboss.org/projects/realearn/) (written in C++ but being ported to Rust).

In both cases you need to make a library crate of type `cdylib`.

### REAPER extension plug-in

Using the `reaper_extension_plugin` macro is the fastest way to get going.

Add this to your `Cargo.toml`:

```toml
[dependencies]
reaper-rs-low = "0.1.0"
reaper-rs-medium = "0.1.0"
reaper-rs-macros = "0.1.0"

[lib]
name = "my_reaper_extension_plugin"
crate-type = ["cdylib"]
```

Then in your `lib.rs`:
```rust
use std::error::Error;
use reaper_rs_macros::reaper_extension_plugin;
use reaper_rs_low::ReaperPluginContext;
use reaper_rs_medium::Reaper;

#[reaper_extension_plugin]
fn plugin_main(context: &ReaperPluginContext) -> Result<(), Box<dyn Error>> {
    let reaper = Reaper::load(context);
    reaper.functions().show_console_msg("Hello world from reaper-rs medium-level API!");
    Ok(())
}
```

The macro doesn't do much more than exposing an `extern "C" ReaperPluginEntry()` function which calls
`reaper_rs_low::bootstrap_extension_plugin()`. So if for some reason you don't want to use
macros, have a look into the macro implementation. No magic there.

### REAPER VST plug-in

A REAPER VST plug-in is nothing else than a normal VST plug-in which gets access to functions from the REAPER SDK. 
Luckily, there is a Rust crate for creating VST plug-ins already: [vst-rs](https://crates.io/crates/vst).
So all you need to do is write a VST plug-in via *vst-rs* and gain access to the REAPER functions by letting
*reaper-rs* access the `HostCallback` function.

Add this to your `Cargo.toml`:

```toml
[dependencies]
reaper-rs-low = "0.1.0"
reaper-rs-medium = "0.1.0"
vst = "0.2.0"

[lib]
name = "my_reaper_vst_plugin"
crate-type = ["cdylib"]
```

Then in your `lib.rs`:
```rust
use vst::plugin::{Info, Plugin, HostCallback};
use reaper_rs_low::ReaperPluginContext;
use reaper_rs_medium::Reaper;

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
        if let Ok(context) = ReaperPluginContext::from_vst_plugin(self.host) {
            let reaper = Reaper::load(&context);
            reaper
                .functions()
                .show_console_msg("Hello world from reaper-rs medium-level API!");
        }
    }
}

vst::plugin_main!(MyReaperVstPlugin);
```
    
## Contribute

Contributions are very welcome! Especially to the medium-level API.

### Build

Thanks to Cargo, building *reaper-rs* is not a big deal.  

#### Windows

In the following you will find the instructions for Windows 10. Points where you have to consider the target 
architecture (REAPER 32-bit vs. 64-bit) are marked with :star: (the instructions assume 64-bit).

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
    - [Download](https://www.rust-lang.org/tools/install)  and execute `rustup-init.exe` 
    - Accept the defaults
    - Set the correct toolchain default (*nightly* toolchain is not necessary if you only want to build 
      `reaper_rs_low` and `reaper_rs_medium`) :star:
        ```batch
        rustup default nightly-x86_64-pc-windows-msvc
        ```
3. Download and install [Git for Windows](https://git-scm.com/download/win)
4. Clone the *reaper-rs* Git repository
    ```batch
    git clone --recurse-submodules https://github.com/helgoboss/reaper-rs.git`
    ```
5. Build *reaper-rs*
    ```batch
    cd reaper-rs
    cargo build
    ```

This is how you regenerate the low-level API:

1. [Download](https://releases.llvm.org/download.html) and install LLVM for Windows 64-bit :star:
2. Build with the `generate` feature enabled
    ```batch
    cd main\low
    cargo build --features generate
    ``` 



#### Linux

Complete instructions to build *reaper-rs* from a *fresh* Ubuntu 18.04.3 LTS installation and make the test plug-ins
available in REAPER:

```sh
# Install basic stuff
sudo apt update
sudo apt install curl git build-essential -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # choose 1 (default)
source $HOME/.cargo/env
# Using nightly is not necessary if you want to build just the low-level or medium-level API!
rustup default nightly

# Clone reaper-rs
cd Downloads
git clone --recurse-submodules https://github.com/helgoboss/reaper-rs.git
cd reaper-rs
cargo build

# At this point, download REAPER for Linux and start it at least one time!
# ...

# Then continue
ln -s $HOME/Downloads/reaper-rs/target/debug/libreaper_rs_test_extension_plugin.so $HOME/.config/REAPER/UserPlugins/reaper_rs_test_extension_plugin.so
mkdir -p $HOME/.config/REAPER/UserPlugins/FX
ln -s $HOME/Downloads/reaper-rs/target/debug/libreaper_rs_test_vst_plugin.so $HOME/.config/REAPER/UserPlugins/FX/reaper_rs_test_extension_plugin.so
```

That's it!

#### Mac OS X

*To be done*

### Test

When building the complete *reaper-rs* workspace, 3 test crates are produced:

- `reaper_rs_test`
- `reaper_rs_test_extension_plugin`
- `reaper_rs_test_vst_plugin`

`reaper_rs_test` provides an integration test that is supposed to be run in REAPER itself. This is the main testing
mechanism for *reaper-rs*. `reaper_rs_test_extension_plugin` and `reaper_rs_test_vst_plugin` are both test plug-ins 
which register the integration test as REAPER action.

Running the integration test is not only a good way to find *reaper-rs* regression bugs, but can also help to expose
subtle changes in the REAPER SDK itself. Currently, the test assertions are very strict in order to reveal even
the slightest deviations.

## Project background

*reaper-rs* has been born as part of an effort to port the REAPER VST plug-in 
[ReaLearn](https://www.helgoboss.org/projects/realearn/) to Rust and publish it as open-source project. The high-level
API is heavily inspired by ReaPlus, a C++ facade for the REAPER SDK, which is a basic building block of the original 
ReaLearn.