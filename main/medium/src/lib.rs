#![doc(html_root_url = "https://docs.rs/reaper-medium/0.1.0")]
#![deny(broken_intra_doc_links)]

//! This crate contains the medium-level API of [reaper-rs](https://github.com/helgoboss/reaper-rs).
//!
//! To get started, have a look at the [`ReaperSession`] struct.
//!
//! # General usage hints
//!
//! - Whenever you find an identifier in this crate that ends with `index`, you can assume it's a
//!   *zero-based* integer. That means the first index is 0, not 1!
//!
//! # Example
//!
//! ```no_run
//! # let session = reaper_medium::ReaperSession::default();
//! use reaper_medium::ProjectContext::CurrentProject;
//!
//! let reaper = session.reaper();
//! reaper.show_console_msg("Hello world from reaper-rs medium-level API!");
//! let track = reaper.get_track(CurrentProject, 0).ok_or("no tracks")?;
//! unsafe { reaper.delete_track(track); }
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```
//!
//! # Design goals
//!
//! The ultimate goal of the medium-level API is to provide all functions offered by the low-level
//! API, but in an idiomatic and type-safe way. The result is still a plain list of functions, where
//! each function is basically named like its original. Going all object-oriented,
//! using reactive extensions, introducing a fluid API, finding function names that make more sense
//! ... all of that is intentionally *out of scope*. The medium-level API is intended to stay close
//! to the original API. This has the benefit that ReaScript (e.g. Lua) and C++ code seen in forum
//! threads, blogs and existing extensions can be helpful even for writing plug-ins in Rust.
//!
//! # Design principles
//!
//! In order to achieve these goals, this API follows a bunch of design principles.
//!
//! ## Follow Rust naming conventions
//!
//! Most low-level functions and types don't follow the Rust naming conventions. We adjust them
//! accordingly while still staying as close as possible to the original names.
//!
//! ## Use unsigned integers where appropriate
//!
//! We don't use signed integers when it's totally clear that a number can never be negative.
//! Example: [`insert_track_at_index()`](struct.Reaper.html#method.insert_track_at_index)
//!
//! ## Use enums where appropriate
//!
//! We want more type safety and more readable code. Enums can contribute to that a lot. Here's how
//! we use them:
//!
//! 1. If the original function uses an integer which represents a limited set of
//!    options that can be easily named, we introduce an enum. Example:
//! [`get_track_automation_mode()`](struct.Reaper.html#method.insert_track_at_index),
//! [`AutomationMode`](enum.AutomationMode.html)
//!
//! 2. If the original function uses a string and there's a clear set of predefined
//!    options, we introduce an enum. Example:
//! [`get_media_track_info_value()`](struct.Reaper.html#method.get_media_track_info_value),
//! [`TrackAttributeKey`](enum.TrackAttributeKey.html)
//!
//! 3. If the original function uses a bool and the name of the function doesn't give that bool
//!    meaning, introduce an enum. Example:
//! [`set_current_bpm()`](struct.Reaper.html#method.set_current_bpm),
//! [`UndoBehavior`](enum.UndoBehavior.html)
//!
//! 4. If the original function can have different mutually exclusive results, introduce an enum.
//!    Example:
//! [`get_last_touched_fx()`](struct.Reaper.html#method.get_last_touched_fx),
//! [`GetLastTouchedFxResult`](enum.GetLastTouchedFxResult.html)
//!
//! 5. If the original function has several parameters of which only certain combinations are valid,
//!    introduce an enum for combining those. Example:
//! [`kbd_on_main_action_ex()`](struct.Reaper.html#method.kbd_on_main_action_ex),
//! [`ActionValueChange`](enum.ActionValueChange.html)
//!
//! 6. If the original function takes a parameter which describes how another parameter is
//!    interpreted, introduce an enum. Example:
//! [`csurf_on_pan_change_ex()`](struct.Reaper.html#method.csurf_on_pan_change_ex),
//! [`ValueChange`](enum.ValueChange.html)
//!
//! 7. If the original function takes an optional value and one cannot conclude from the function
//!    name what a `None` would mean, introduce an enum. Example:
//! [`count_tracks()`](struct.Reaper.html#method.count_tracks),
//! [`ProjectContext`](enum.ProjectContext.html)
//!
//! The first design didn't have many enums. Then, with every enum introduced in the medium-level
//! API, the high-level API code was getting cleaner, more understandable and often even shorter.
//! More importantly, some API usage bugs suddenly became obvious!
//!
//! ## Adjust return types where appropriate
//!
//! 1. Use `bool` instead of `i32` as return value type for "yes or no" functions. Example:
//!    [`is_in_real_time_audio()`](struct.Reaper.html#method.is_in_real_time_audio)
//! 2. Use return values instead of output parameters. Example:
//!    [`gen_guid()`](struct.Reaper.html#method.gen_guid)
//! 3. If a function has multiple results, introduce and return a struct for aggregating them.
//!    Example: [`get_focused_fx()`](struct.Reaper.html#method.get_focused_fx)
//! 4. If a function can return a value which represents that something is not present,
//!    return an `Option`. Example:
//!    [`named_command_lookup()`](struct.Reaper.html#method.named_command_lookup)
//!
//! ## Use newtypes where appropriate
//!
//! 1. If a value represents an ID, introduce a newtype. Example:
//!    [`CommandId`](struct.CommandId.html)
//! 2. If a number value is restricted in its value range, represents a mathematical unit or can be
//!    easily confused, consider introducing a meaningful newtype. Example:
//!    [`ReaperVolumeValue`](struct.ReaperVolumeValue.html)
//!
//! We *don't* use newtypes for numbers that represent indexes.
//!
//! ## Use convenience functions where necessary
//!
//! In general, the medium-level API shouldn't have too much additional magic and convenience.
//! However, there are some low-level functions which are true allrounders. With allrounders it's
//! often difficult to find accurate signatures and impossible to avoid `unsafe`. Adding multiple
//! convenience functions can sometimes help with that, at least with making them a *bit* more
//! safe to use.
//! Examples:
//! [`get_set_media_track_info()`](struct.Reaper.html#method.get_set_media_track_info),
//! [`plugin_register_add_command_id()`]
//!
//! ## Make it easy to work with strings
//!
//! - String parameters are used as described in [`ReaperStringArg`](struct.ReaperStringArg.html).
//!   Example: [`string_to_guid()`](struct.Reaper.html#method.string_to_guid)
//! - Strings in return positions are dealt with in different ways:
//!     - When returning an owned string, we return [`ReaperString`](struct.ReaperString.html).
//!       Consumers can easily convert them to regular Rust strings when needed. Example:
//!       [`guid_to_string()`](struct.Reaper.html#method.guid_to_string)
//!     - When returning a string owned by REAPER and we know that string has a static lifetime, we
//!       return a `'static` reference. Example:
//!       [`get_app_version()`](struct.Reaper.html#method.get_app_version)
//!     - When returning a string owned by REAPER and we can't give it a proper lifetime annotation
//!       (in most cases we can't), we grant the user only temporary access to that string by taking
//!       a closure with a `&`[`ReaperStr`](struct.ReaperStr.html) argument which is executed right
//!       away. Example: [`undo_can_undo_2()`](struct.Reaper.html#method.undo_can_undo_2)
//! - Strings in enums are often `Cow<ReaperStr>` because we want them to be flexible enough to
//!   carry both owned and borrowed strings.
//!
//! ## Use pointer wrappers where appropriate
//!
//! When we deal with REAPER, we have to deal with pointers. REAPER often returns pointers and we
//! can't give them a sane lifetime annotation. Depending on the type of plug-in and the type of
//! pointer, some are rather static from the perspective of the plug-in and others can come and go
//! anytime. In any case, just turning them into `'static` references would be plain wrong. At the
//! same time, annotating them with a bounded lifetime `'a` (correlated to another lifetime) is
//! often impossible either, because mostly we don't have another lifetime at the disposal which can
//! serve as frame of reference.
//!
//! In most cases the best we can do is passing pointers around. How exactly this is done,
//! depends on the characteristics of the pointed-to struct and how it is going to be used.
//!
//! ### Case 1: Internals not exposed | no vtable
//!
//! #### Strategy
//!
//! - Use `NonNull` pointers directly
//! - Make them more accessible by introducing an alias
//!
//! #### Explanation
//!
//! Such structs are relevant for the consumers *as pointers only*. Because they are
//! completely opaque (internals not exposed, not even a vtable). We don't create a newtype because
//! the `NonNull` guarantee is all we need and we will never provide any methods on them (no vtable
//! emulation, no convenience methods). Using a wrapper just for reasons of symmetry would not be
//! good because it comes with a cost (more code to write, less substitution possibilities) but in
//! this case without any benefit.
//!
//! #### Examples
//!
//! - [`raw::MediaTrack`](../reaper_low/raw/struct.MediaTrack.html) →
//!   [`MediaTrack`](type.MediaTrack.html)
//! - [`raw::ReaProject`](../reaper_low/raw/struct.ReaProject.html) →
//!   [`ReaProject`](type.ReaProject.html)
//! - [`raw::MediaItem_Take`](../reaper_low/raw/struct.MediaItem_Take.html) →
//!   [`MediaItemTake`](type.MediaItemTake.html)
//!
//! ### Case 2: Internals exposed | no vtable
//!
//! #### Strategy
//!
//! - *Don't* create an alias for a `NonNull` pointer! In situations where just the pointer is
//!   interesting and not the internals, write `NonNull<...>` everywhere.
//! - If the consumer shall get access to the internals: Wrap the `NonNull` pointer in a public
//!   newtype. This newtype should expose the internals in a way which is idiomatic for Rust (like
//!   the rest of the medium-level API does).
//! - If the consumer needs to be able to create such a struct: Provide an idiomatic Rust factory
//!   function. If that's not enough because the raw struct is not completely owned, write an owned
//!   version of that struct, prefixed with `Owned`. Ideally it should wrap the raw struct.
//! - Sometimes when having both an owned struct *and* a pointer wrapper, it can be useful to also
//!   introduce a borrowed reference-only struct. The owned struct can conveniently deref to the
//!   borrowed struct. The pointer wrapper can provide an unsafe `as_ref()` which returns a
//!   reference to the borrowed struct. See case 3 for an example (`PCM_source`).
//!
//! #### Explanation
//!
//! Unlike [`raw::MediaTrack`](../reaper_low/raw/struct.MediaTrack.html) and friends, these
//! structs are *not* opaque. Still, we need them as pointers and they have the same lifetime
//! considerations. The difference is that we add type-safe methods to them in order to lift their
//! members to medium-level API style.
//!
//! #### Examples
//!
//! - [`raw::KbdSectionInfo`](../reaper_low/raw/struct.KbdSectionInfo.html) →
//!   [`KbdSectionInfo`](struct.KbdSectionInfo.html) & `MediumKdbSectionInfo` (not yet existing)
//! - [`raw::audio_hook_register_t`](../reaper_low/raw/struct.audio_hook_register_t.html) →
//!   [`AudioHookRegister`](struct.AudioHookRegister.html) &
//!   [`OwnedAudioHookRegister`](struct.OwnedAudioHookRegister.html)
//! - [`raw::gaccel_register_t`](../reaper_low/raw/struct.gaccel_register_t.html) → `GaccelRegister`
//!   (not yet existing) & [`OwnedGaccelRegister`](struct.OwnedGaccelRegister.html)
//!
//! ### Case 3: Internals not exposed | vtable
//!
//! #### Strategy
//!
//! - *Don't* create an alias for a `NonNull` pointer! In situations where just the pointer is
//!   interesting and not the internals, write `NonNull<...>` everywhere.
//! - If the consumer shall get access to the virtual functions: Wrap `NonNull` pointer in a public
//!   newtype. This newtype should expose the virtual functions in a way which is idiomatic for
//!   Rust. It's intended for the communication from Rust to REAPER. This needs appropriate
//!   companion C code in the low-level API.
//! - If the consumer needs to be able to provide such a type (for communication from REAPER to
//!   Rust): Create a new trait which can be implemented by the consumer. This also needs
//!   appropriate companion C code in the low-level API.
//! - See case 2 strategy for dealing with cases where you need both a pointer wrapper and an owned
//!   struct.
//! - The most complete example which uses all of these techniques: `PCM_source`
//!
//! #### Examples
//!
//! - [`raw::IReaperControlSurface`](../reaper_low/raw/struct.IReaperControlSurface.html) →
//!   `ReaperControlSurface` (not yet existing) & [`ControlSurface`](trait.ControlSurface.html)
//! - [`raw::midi_Input`](../reaper_low/raw/struct.midi_Input.html) →
//!   [`MidiInput`](struct.MidiInput.html) &
//! - [`raw::MIDI_eventlist`](../reaper_low/raw/struct.MIDI_eventlist.html) →
//!   [`BorrowedMidiEventList`](struct.BorrowedMidiEventList.html) &
//! - [`raw::PCM_source`](../reaper_low/raw/struct.PCM_source.html) →
//!   [`OwnedPcmSource`](struct.OwnedPcmSource.html), [`PcmSource`](struct.PcmSource.html),
//!   [`BorrowedPcmSource`](struct.BorrowedPcmSource.html) &
//!   [`CustomPcmSource`](trait.CustomPcmSource.html)
//!
//! ## Panic/error/safety strategy
//!
//! - We panic if a REAPER function is not available, e.g. because it's an older REAPER version.
//!   Rationale: If *all* function signatures would be cluttered up with `Result`s, it would be an
//!   absolute nightmare to use the API. It's also not necessary: The consumer can always check if
//!   the function is there, and mostly it is (see
//!   [`reaper_low::Reaper`](../reaper_low/struct.Reaper.html)).
//! - We panic when passed parameters don't satisfy documented preconditions which can be easily
//!   satisfied by consumers. Rationale: This represents incorrect API usage.
//!     - Luckily, the need for precondition checks is mitigated by using lots of newtypes and
//!       enums, which don't allow parameters to be out of range in the first place.
//!   Example: [`track_fx_get_fx_name()`](struct.Reaper.html#method.track_fx_get_fx_name)
//! - When a function takes pointers, we generally mark it as `unsafe`. Rationale: Pointers can
//!   dangle (e.g. a pointer to a track dangles as soon as that track is removed). Passing a
//!   dangling pointer to a REAPER function can and often will make REAPER crash. Example:
//!   [`delete_track()`](struct.Reaper.html#method.delete_track)
//!     - That's a bit unfortunate, but unavoidable given the medium-level APIs design goal to stay
//!       close to the original API. The `unsafe` is a hint to the consumer to be extra careful with
//!       those functions.
//!     - The consumer *has* ways to ensure that the passed pointer is valid:
//!
//!          1. Using obtained pointers right away instead of caching them (preferred)
//!
//!          2. Using [`validate_ptr_2()`](struct.Reaper.html#method.validate_ptr_2) to
//!             check if the cached pointer is still valid.
//!          
//!          3. Using a
//!             [hidden control
//! surface](struct.ReaperSession.html#method.plugin_register_add_csurf_inst)             to be
//! informed whenever e.g. a `MediaTrack` is removed and invalidating the cached             pointer
//! accordingly.
//! - There's one exception to this: If the parameters passed to the function in question are enough
//!   to check whether the pointer is still valid, we do it, right in that function. If it's
//!   invalid, we panic. We use [`validate_ptr_2()`](struct.Reaper.html#method.validate_ptr_2) to
//!   check the pointer. Sadly, for all but project pointers it needs a project context to be able
//!   to validate a pointer. Otherwise we could apply this rule much more. Rationale: This allows us
//!   to remove the `unsafe` (if there was no other reason for it). That's not ideal either but it's
//!   far better than undefined behavior. Failing fast without crashing is one of the main design
//!   principles of *reaper-rs*. Because checking the pointer is an "extra" thing that the
//!   medium-level API does, we also offer an unsafe `_unchecked` variant of the same function,
//!   which doesn't do the check. Example:
//!   [`count_tracks()`](struct.Reaper.html#method.count_tracks) and
//!   [`count_tracks_unchecked()`](struct.Reaper.html#method.count_tracks_unchecked)
//! - If a REAPER function can return a value which represents that execution was not successful,
//!   return a `Result`. Example: [`string_to_guid()`](struct.Reaper.html#method.string_to_guid)
//!
//! Verdict: Making the API completely safe to use can't be done in the medium-level API. But it can
//! be done in the high-level API because it's not tied to the original REAPER flat function
//! signatures. For example, there could be a `Track` struct which holds a `ReaProject` pointer,
//! the track index and the track's GUID. With that combination it's possible to detect reliably
//! whether a track is still existing. Needless to say, this is far too opinionated for the
//! medium-level API.
//!
//!
//! ## Try to follow "zero-cost" principle
//!
//! If someone uses C++ or Rust instead of just settling with ReaScript, chances are that better
//! performance is at least one of the reasons. The medium-level API acknowledges that and tries
//! to be very careful not to introduce possibly performance-harming indirections. In general it
//! shouldn't do extra stuff. Just the things which are absolutely necessary to reach the design
//! goals mentioned above. This is essential for code that is intended to be executed in
//! the real-time audio thread (no heap allocations etc.).
//!
//! This is an important principle. It would be bad if it's necessary to reach out to the low-level
//! API whenever someone wants to do something performance-critical. The low-level API shouldn't
//! even be considered as a serious Rust API, it's too raw and unsafe for Rust standards.
//!
//!
//! [`ReaperSession`]: struct.ReaperSession.html
//! [`plugin_register_add_command_id()`]:
//! struct.ReaperSession.html#method.plugin_register_add_command_id

#[macro_use]
mod macros;

mod misc_enums;
pub use misc_enums::*;

mod misc_newtypes;
pub use misc_newtypes::*;

mod key_enums;
pub use key_enums::*;

mod fn_traits;
pub use fn_traits::*;

mod flags;
pub use flags::*;

mod reaper_pointer;
pub use reaper_pointer::*;

mod gaccel_register;
pub use gaccel_register::*;

mod preview_register;
pub use preview_register::*;

mod audio_hook_register;
pub use audio_hook_register::*;

mod keeper;

mod control_surface;
pub use control_surface::*;

mod midi;
pub use midi::*;

mod pcm_source;
pub use pcm_source::*;

mod reaper_session;
pub use reaper_session::*;

mod reaper;
pub use reaper::*;

mod util;
use util::*;

#[cfg(feature = "reaper-meter")]
#[doc(hidden)]
mod metering;

mod string_types;
pub use string_types::*;

mod recording_input;
pub use recording_input::*;

mod automation_mode;
pub use automation_mode::*;

mod message_box;
pub use message_box::*;

mod ptr_wrappers;
pub use ptr_wrappers::*;

mod errors;
pub use errors::*;

mod plugin_context;
pub use plugin_context::*;

mod mutex;
pub use mutex::*;
