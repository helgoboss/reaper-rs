//! Provides all functions from `reaper_plugin_functions.h` with the following small improvements:
//! TODO-medium Doc
//! Doc: Explain that returning CString instead of String is because we also expect CStrings
//! as ideal arguments (for good reasons). It would not be symmetric to return Strings then.
//! Should be similar to Lua API, not too far away because there are lots of tutorials already.
//! At first I didn't have the enums in place and was closer to the low-level API. At the point when
//! I changed to enums I could watch how my high-level API code gets cleaner, more understandable
//! and often also less. In some cases I also discovered API usage bugs because of that. So I think
//! enums are a good choice here. Again, you can always resort to low-level API.
//! I think that with the right abstractions in place, you can build sophisticated extensions much
//! easier, faster and with less bugs because there's no need to take care of the same low-level
//! stuff again and again.
//! The medium-level API offers much more type safety and convenience. Still stays close to
//! original REAPER API and the ultimate goal is to expose every function with every possible
//! calling style of the low-level API, just
//! in a bit nicer and type-safe manner - so that at the end you don't have to resort to the
//! low-level API anymore and this gets a complete replacement. Some low-level functions can't be
//! rewritten in a type-safe way. In this case, new convenience functions are introduced.
//! Quite likely that someone who uses Rust (instead of e.g. Lua) does it also because of
//! performance reasons. medium-level API can be considered the first API. The low-level one is not
//! really supposed to be used directly. It's important that the "first" API still is sensitive
//! about performance and doesn't do too much extra.
//!
//! - Note about strings (both return and parameter)!
//! - When I say "index", I always mean zero-based
//!
//! - Snake-case function and parameter names
//! - Use bool instead of i32 as return value type for functions with obvious "yes or no" result
//! - Use ReaperStringPtr instead of raw c_char pointers as return value type (offers convenience
//!   functions)
//! - Use ReaperVoidPtr instead of raw c_void pointers as return value type (offers convenience
//!   functions)
//! - Use return values instead of output parameters
//! - When there are string output parameters which can be passed a null pointer, trigger this null
//!   pointer case when a buffer size of 0 is passed, also use Cow in this case in order to have a
//!   cheap empty string in null-pointer case
//! - When there are both return values and output parameters, return a tuple if there's just one
//!   output parameter and a struct if there are many output parameters
//! - In all REAPER functions which can fail (mostly indicated by returning false or -1), return
//!   Result
//! - In all REAPER functions which return things that might not be present, return Option
//! - Panics if function not available (we should make sure on plug-in load that all necessary
//!   functions are available)
//! - More restrictive number types where safely applicable (for increased safety, e.g. u32 instead
//!   of i32). In the unlikely case that the value range has to be extended in future, it's just a
//!   matter of removing safe casts on user-side code.
//
//  2. Should we panic if ptr invalid or return result? Returning None in case it's an Option
//     result is bad because this is not always the case. I think we should panic. Otherwise
//     our nice signatures get cluttered up with results. It should be declared as general
//     precondition for all methods that the passed pointers are valid. This can be made sure
//     by either always fetching them (jf: "Ideally you shouldn't hang on to pointers longer
//     than you need them") or by using reactive programming (react when object gets removed).
// Should we make this unsafe? I think this is no different than with other functions
//  in  Reaper struct that work on pointers whose lifetimes are not known. We should find ONE
//  solution. Probably it's good to follow this: If we can guarantee there's no UB, we should do
//  it, if not, we should mark the method unsafe. Is there any way to guarantee? I see this:
//  a) Use something like the ValidatePtr function if available. However, calling it for each
//     invocation is too presumptuous for an unopinionated medium-level API. Or perhaps more
//     importantly, it's often not possible because we would need a contect ReaProject* pointer
//     in order to carry out the validation.
//  b) Also store an ID or something (e.g. section ID here) and always refetch it. Same like
//     with a ... very presumptuous.
//  So none of this is really feasible on this API level. Which means that we must either rely
//  on REAPER itself not running into UB (waiting for Justin to comment on some functions) or
//  just mark the methods where this is not possible as unsafe. A higher-level API then should
//  take care of making things absolutely safe.
//
// # Newtype rules
//
// in some cases (like DbVal/Vol) it seems like medium-level API will get too much power. As soon as
// we provide methods on newtypes that invoke REAPER, we must stop! That's high-level stuff. We
// should follow clear rules when to introduce newtypes/enums:
//
// Clear and done:
//
// 1. If original function uses an integer and it reflects limited options which have names, of
// course introduce an enum 2. If original function uses a bool and the name of the function doesn't
// give that bool meaning, introduce an enum 3. If a function has multiple results, introduce a
// struct 4. If a function can have different results, introduce an enum
// 5. If a function has many parameters of which only certain combinations are valid, introduce an
// enum for combining those 6. If a function takes a parameter which describes how another parameter
// is interpreted, introduce a newtype (Absolute/Relative) 7. If a function takes an optional value
// (Option) and the function name doesn't make one see what a None means, introduce an enum
// 10. Introduce newtypes for IDs (could make sense especially when some also are returned - because
// we never want to confuse IDs) 8. If a function takes a number value and that number is restricted
// in its value range, introduce a meaningful newtype (e.g. Vol, Pan) ... could be quite useful,
// especially for conversions. And should be available to someone who just wants to deal with the
// medium-level API. The high-level API should just reuse those types really!
//
// Not done:
//
// 9. If a function takes whatever number, even if they can have the full range of a primitive,
// introduce a newtype just for safety and (sometimes) for seeing what's it at about at call site
// ... no 11. Introduce newtypes for indexes ... no
//
// # Pointer wrappers
// We obtain many pointers directly from REAPER and we can't give them a sane lifetime annotation.
// They are "rather" static from the perspective of the plug-in, yet they could come and go anytime,
// so 'static would be too optimistic. Annotating with a lifetime 'a - correlated to another
// lifetime - would be impossible because we don't have such another lifetime which can serve as
// frame of reference. So the best we can do is wrapping pointers. For all opaque structs we do that
// simply by creating a type alias to NonNull because NonNull maintains all the invariants we need
// (pointer not null) and opaque structs don't have methods which need to be lifted to medium-level
// API style. For non-opaque structs we wrap the NonNull in a newtype because we need to add
// medium-level API style methods. One of the responsibilities of the medium-level API is to use
// identifiers which follow the Rust conventions. It just happens that some of the C++ classes
// already conform to Rust conventions, so we won't rename them.
//
//

// Case 1: Internals exposed: no | vtable: no
// ==========================================
//
// ## Strategy
//
// - Use NonNull pointers directly
// - Make them more accessible by using a public alias
//
// ## Explanation
//
// These structs are relevant for the consumers, but only as pointers. Because those structs are
// completely opaque (internals not exposed, not even a vtable). We don't create a newtype because
// the NonNull guarantee is all we need and according to medium-level API design, we will never
// provide any methods on them (no vtable emulation, no convenience methods). Using a newtype just
// for reasons of symmetry would not be good because it comes with a cost (more code to write,
// less substitution possibilities) but in this case without any benefit.
//
// ## Examples
//
// - MediaTrack → MediaTrack
// - ReaProject → ReaProject
// - MediaItem_Take → MediaItemTake

// Case 2: Internals exposed: yes | vtable: no
// ===========================================
//
// ## Strategy
//
// - **Don't** create an alias for a NonNull pointer! In situations where just the pointer is
//   interesting and not the internals, just write `NonNull<...>` everywhere.
// - If the consumer shall get access to the internals: Wrap the NonNull pointer in a public
//   newtype. This newtype should expose the internals in a way which is idiomatic for Rust (like
//   the rest of the medium-level API does).
// - If the consumer needs to be able to create such a struct: Provide an idiomatic Rust factory
//   function. If that's not enough because the raw struct is not completely owned, write an owned
//   version of that struct, prefixed with `Medium`. Ideally it should wrap the raw struct.
//
// ## Explanation
//
// Each of the these structs is relevant to consumers, but unlike `MediaTrack` and Co. it points
// to a struct which is *not* opaque. Still, we need it as pointer and it has the same lifetime
// considerations. The difference is that we add type-safe methods to it in order to lift the
// members of that struct to medium-level API style.
//
// ## Examples
//
// - KbdSectionInfo → KbdSectionInfo & MediumKdbSectionInfo (not yet existing because not needed
//   yet)
// - audio_hook_register_t → AudioHookRegister & MediumAudioHookRegister
// - gaccel_register_t → GaccelRegister (not yet existing because not needed yet) &
//   MediumGaccelRegister
// - ACCEL → Accel (not yet existing because not needed yet) & MediumAccel
//
// Case 3: Internals exposed: no | vtable: yes
// ===========================================
//
// ## Strategy
//
// - **Don't** create an alias for a NonNull pointer! In situations where just the pointer is
//   interesting and not the internals, just write `NonNull<...>` everywhere.
// - If the consumer shall get access to the virtual functions: Wrap NonNull pointer in a public
//   newtype. This newtype should expose the virtual functions in a way which is idiomatic for Rust.
//   It's intended for the communication from Rust to REAPER. Hint: This needs companion C code (see
//   `impl midi_Input`)!
// - If the consumer needs to be able to provide such a type (for communication from REAPER to
//   Rust): Create a new trait prefixed with `Medium` which can be implemented by the consumer.
//   Hint: This needs companion C code in the low-level API (see `IReaperControlSurface`)!
//
// ## Examples
//
// - PCM_source → PcmSource & MediumPcmSource (both not yet existing because not needed yet)
// - IReaperControlSurface → ReaperControlSurface (not yet existing because not needed yet) &
//   MediumReaperControlSurface
// - midi_Input → MidiInput
// - MIDI_eventlist → MidiEventList

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
