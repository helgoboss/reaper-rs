#![deny(broken_intra_doc_links)]
//! This crate contains the high-level API of [reaper-rs](https://github.com/helgoboss/reaper-rs).
//!
//! **This API is not polished yet and will still undergo many changes!**
//!
//! # Example
//!
//! ```no_run
//! # let reaper = reaper_high::Reaper::get();
//!
//! reaper.show_console_msg("Hello world from reaper-rs high-level API!");
//! let project = reaper.current_project();
//! let track = project.track_by_index(0).ok_or("no tracks")?;
//! project.remove_track(&track);
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```

#[macro_use]
mod regex_util;

mod log_util;
pub use log_util::*;

mod debug_util;
pub use debug_util::*;

pub mod run_loop_executor;

pub mod local_run_loop_executor;

mod helper_control_surface;

mod reaper;
pub use reaper::*;

mod main_task_middleware;
pub use main_task_middleware::*;

mod main_future_middleware;
pub use main_future_middleware::*;

mod reaper_simple;
pub use reaper_simple::*;

mod project;
pub use project::*;

mod track;
pub use track::*;

mod take;
pub use take::*;

mod track_route;
pub use track_route::*;

mod fx;
pub use fx::*;

mod fx_parameter;
pub use fx_parameter::*;

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

mod play_rate;
pub use play_rate::*;

mod pan;
pub use pan::*;

mod width;
pub use width::*;

mod tempo;
pub use tempo::*;

mod chunk;
pub use chunk::*;

mod item;
pub use item::*;

mod source;
pub use source::*;

mod action_character;
pub use action_character::*;

#[cfg(feature = "serde")]
mod meter_middleware;
#[cfg(feature = "serde")]
pub use meter_middleware::*;

mod undo_block;

mod normalized_value;

mod middleware_control_surface;
pub use middleware_control_surface::*;

mod change_detection_middleware;
pub use change_detection_middleware::*;

mod option_util;

mod bookmark;
pub use bookmark::*;
