#![feature(fn_traits, clamp, backtrace, test)]
//! This crate contains the high-level API of [reaper-rs](https://github.com/helgoboss/reaper-rs).
//!
//! **This API is not polished yet and will still undergo many changes!**
//!
//! # Example
//!
//! ```no_run
//! # let reaper = reaper_high::Reaper::get();
//! use rxrust::prelude::*;
//!
//! reaper.show_console_msg("Hello world from reaper-rs high-level API!");
//! reaper.track_removed().subscribe(|t| println!("Track {:?} removed", t));
//! let project = reaper.get_current_project();
//! let track = project.get_track_by_index(0).ok_or("no tracks")?;
//! project.remove_track(&track);
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```

#[macro_use]
mod regex_util;

mod log_util;
pub use log_util::*;

mod event_loop_executor;
use event_loop_executor::*;

mod reaper;
pub use reaper::*;

mod reaper_simple;
pub use reaper_simple::*;

mod project;
pub use project::*;

mod track;
pub use track::*;

mod track_send;
pub use track_send::*;

mod fx;
pub use fx::*;

mod fx_parameter;
pub use fx_parameter::*;

mod helper_control_surface;

mod section;
pub use section::*;

mod action;
pub use action::*;

mod guid;
pub use guid::*;

mod fx_chain;
pub use fx_chain::*;

mod midi_input_device;
pub use midi_input_device::*;

mod midi_output_device;
pub use midi_output_device::*;

mod volume;
pub use volume::*;

mod pan;
pub use pan::*;

mod tempo;
pub use tempo::*;

mod chunk;
pub use chunk::*;

mod action_character;
pub use action_character::*;

mod undo_block;

mod normalized_value;

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;

    #[bench]
    fn thread_comparison_speed(b: &mut Bencher) {
        let main_thread_id = std::thread::current().id();
        b.iter(|| {
            std::thread::spawn(|| 5);
            let current_thread_id = test::black_box(std::thread::current().id());
            assert_eq!(current_thread_id, main_thread_id);
        });
    }
}
