//! This module contains the medium-level API of *reaper-rs*.
//!
//! To get started, have a look at the [`Reaper`] struct.
//!
//! # Hints
//!
//! - Whenever you find an identifier in this crate that ends with `index`, you can assume it's a
//!   *zero-based* integer. That means the first index is 0, not 1!
//!
//!
//! # Design goals
//!
//! The ultimate goal of the medium-level API is to provide all functions offered by the low-level
//! API, but in an idiomatic and type-safe way which doesn't require the consumer to use `unsafe`
//! all over the place. The result is still a plain list of functions, where each function is named
//! like its original with only minor changes. Going all object-oriented, using reactive
//! extensions, introducing a fluid API, finding function names that make more sense - all of that
//! is intentionally *out of scope*. The medium-level should stay close to the original SDK. This
//! has the benefit that Lua and C++ code seen in forum threads, blogs and existing extensions
//! can help even when writing plug-ins in Rust.
//!
//! # Design principles
//!
//! In order to achieve these goals, this API follows a bunch of design principles.
//!
//! ## Follow Rust naming conventions
//!
//! Most low-level functions and types don't follow the Rust naming conventions. We adjust them
//! accordingly while still staying as close as possible to the original name.
//!
//! ## Use unsigned integers where appropriate
//!
//! Don't use signed integers when it's totally clear that a number can never be negative.
//! Example: [`insert_track_at_index()`](struct.ReaperFunctions.html#method.insert_track_at_index)
//!
//! ## Use enums where appropriate
//!
//! We want more type-safety and more readable code. Enums can contribute to that a lot. Here's how
//! we use them:
//!
//! 1. If an integer represents a limited set of options which can be easily named, introduce an
//!    enum. Example: [`AutomationMode`](enum.AutomationMode.html)
//!
//! 2. If the original function takes or returns a string and there's a clear set of predefined
//!    options, introduce an enum. Example: [`TrackInfoKey`](enum.TrackInfoKey.html)
//!
//! 3. If the original function uses a bool and the name of the function doesn't give that bool
//!    meaning, introduce an enum. Example: [`UndoBehavior`](enum.UndoBehavior.html)
//!
//! 4. If a function can have different mutually exclusive results, introduce an enum. Example:
//!    [`GetLastTouchedFxResult`](enum.GetLastTouchedFxResult.html)
//!
//! 5. If a function has several parameters of which only certain combinations are valid, introduce
//!    an enum for combining those. Example: [`ActionValueChange`](enum.ActionValueChange.html)
//!
//! 6. If a function takes a parameter which describes how another parameter is interpreted,
//!    introduce an enum. Example: [`ValueChange`](enum.ValueChange.html)
//!
//! 7. If a function takes an optional value and one cannot conclude from the function name what a
//!    `None` would mean, introduce an enum. Example: [`ProjectContext`](enum.ProjectContext.html)
//!
//! The first design didn't have many enums. Then, with every enum introduced in the medium-level
//! API, I could watch the high-level API code getting cleaner, more understandable and often even
//! shorter. And even more importantly, I spotted some API usage bugs!
//!
//! ## Adjust return types where appropriate
//!
//! 1. Use `bool` instead of `i32` as return value type for "yes or no" functions. Example:
//!    [`is_in_real_time_audio()`](struct.ReaperFunctions.html#method.is_in_real_time_audio)
//! 2. Use return values instead of output parameters. Example:
//!    [`gen_guid()`](struct.ReaperFunctions.html#method.gen_guid)
//! 3. If a function has multiple results, introduce and return a struct for aggregating them.
//!    Example: [`get_focused_fx()`](struct.ReaperFunctions.html#method.get_focused_fx)
//! 4. If a function can return a value which represents that something is not present,
//!    return an `Option`. Example:
//!    [`named_command_lookup()`](struct.ReaperFunctions.html#method.named_command_lookup)
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
//! ## Use pointer wrappers where appropriate
//!
//! When we deal with REAPER, we have to deal with pointers. REAPER often returns pointers and we
//! can't give them a sane lifetime annotation. Depending on the type of plug-in and the type of
//! pointer, some are rather static from the perspective of the plug-in and others can come and go
//! anytime. In any case, just turning them into `'static` references would be plain wrong. However,
//! annotating them with a bounded lifetime `'a` (correlated to another lifetime) is also often
//! impossible, because mostly we don't have another lifetime which can serve as frame of reference.
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
//! - [`raw::MediaTrack`](../reaper_rs_low/raw/struct.MediaTrack.html) →
//!   [`MediaTrack`](type.MediaTrack.html)
//! - [`raw::ReaProject`](../reaper_rs_low/raw/struct.ReaProject.html) →
//!   [`ReaProject`](type.ReaProject.html)
//! - [`raw::MediaItem_Take`](../reaper_rs_low/raw/struct.MediaItem_Take.html) →
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
//!   version of that struct, prefixed with `Medium`. Ideally it should wrap the raw struct.
//!
//! #### Explanation
//!
//! Unlike [`raw::MediaTrack`](../reaper_rs_low/raw/struct.MediaTrack.html) and friends, these
//! structs are *not* opaque. Still, we need them as pointers and they have the same lifetime
//! considerations. The difference is that we add type-safe methods to them in order to lift their
//! members to medium-level API style.
//!
//! #### Examples
//!
//! - [`raw::KbdSectionInfo`](../reaper_rs_low/raw/struct.KbdSectionInfo.html) →
//!   [`KbdSectionInfo`](struct.KbdSectionInfo.html) & `MediumKdbSectionInfo` (not yet existing)
//! - [`raw::audio_hook_register_t`](../reaper_rs_low/raw/struct.audio_hook_register_t.html) →
//!   [`AudioHookRegister`](struct.AudioHookRegister.html) &
//!   [`MediumAudioHookRegister`](struct.MediumAudioHookRegister.html)
//! - [`raw::gaccel_register_t`](../reaper_rs_low/raw/struct.gaccel_register_t.html) →
//!   `GaccelRegister` (not yet existing) &
//!   [`MediumGaccelRegister`](struct.MediumGaccelRegister.html)
//! - [`raw::ACCEL`](../reaper_rs_low/raw/struct.ACCEL.html) → `Accel` (not yet existing) &
//!   [`MediumAccel`](struct.MediumAccel.html)
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
//!   Rust): Create a new trait prefixed with `Medium` which can be implemented by the consumer.
//!   This also needs appropriate companion C code in the low-level API.
//!
//! #### Examples
//!
//! - [`raw::IReaperControlSurface`](../reaper_rs_low/raw/struct.IReaperControlSurface.html) →
//!   `ReaperControlSurface` (not yet existing) &
//!   [`MediumReaperControlSurface`](struct.MediumReaperControlSurface.html)
//! - [`raw::midi_Input`](../reaper_rs_low/raw/struct.midi_Input.html) →
//!   [`MidiInput`](struct.MidiInput.html) &
//! - [`raw::MIDI_eventlist`](../reaper_rs_low/raw/struct.MIDI_eventlist.html) →
//!   [`MidiEventList`](struct.MidiEventList.html) &
//! - `PCM_source` → `PcmSource` & `MediumPcmSource` (both not yet existing)
//!
//! ## Use convenience functions where necessary
//!
//! In general, the medium-level API shouldn't have too much additional magic and convenience.
//! However, there are some low-level functions which are true allrounders. With allrounders it's
//! often difficult to find accurate signatures and impossible to avoid `unsafe`. Adding multiple
//! convenience functions can sometimes help with that, at least with making them a *bit* more
//! safe to use.
//! Examples:
//! [`get_set_media_track_info()`](struct.ReaperFunctions.html#method.get_set_media_track_info),
//! [`plugin_register_add_command_id()`](struct.Reaper.html#method.plugin_register_add_command_id)
//!
//! ## CONTINUE Strings
//!
//! Doc: Explain that returning CString instead of String is because we also expect CStrings
//! as ideal arguments (for good reasons). It would not be symmetric to return Strings then.
//!
//! - When there are string output parameters which can be passed a null pointer, trigger this null
//!   pointer case when a buffer size of 0 is passed, also use Cow in this case in order to have a
//!   cheap empty string in null-pointer case
//!
//! ## Panic/Error/Safety strategy
//! - In all REAPER functions which can fail (mostly indicated by returning false or -1), return
//!   Result
//! - Panics if function not available (we should make sure on plug-in load that all necessary
//!   functions are available)
//! - Panic if pointer can be checked and we discovered an invalid pointer
//!     - Should we panic if ptr invalid or return result? Returning None in case it's an Option
//!       result is bad because this is not always the case. I think we should panic. Otherwise our
//!       nice signatures get cluttered up with results. It should be declared as general
//!       precondition for all methods that the passed pointers are valid. This can be made sure by
//!       either always fetching them (jf: "Ideally you shouldn't hang on to pointers longer than
//!       you need them") or by using reactive programming (react when object gets removed).
//! - If we can guarantee there's no UB, we should do it, if not, we should mark the method unsafe.
//!     - However, Or perhaps more importantly, it's often not possible because we would need a
//!       contect ReaProject* pointer in order to carry out the validation.
//! - If we do pointer checks, provide an additional _unchecked variant
//! - If we can only guarantee no UB if we additionally store an ID or something (e.g. section ID
//!   here) and always refetch it ... don't do it. Stay unsafe. Too presumptuous, too heavy-weight
//!   for the medium-level API. Leave that to high-level, should take care to make functions
//!   absolutely safe to use.
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
//! [`Reaper`]: struct.Reaper.html

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

mod audio_hook_register;
pub use audio_hook_register::*;

mod infostruct_keeper;

mod control_surface;
pub use control_surface::*;

mod midi;
pub use midi::*;

mod reaper;
pub use reaper::*;

mod reaper_functions;
pub use reaper_functions::*;

mod util;
pub use util::*;

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
