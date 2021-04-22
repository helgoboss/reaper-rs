#![allow(non_snake_case)]
use reaper_low::{create_cpp_to_rust_pcm_source, raw};
use ref_cast::RefCast;

use crate::util::{create_passing_c_str, with_string_buffer};
use crate::{
    BorrowedMidiEventList, DurationInBeats, DurationInSeconds, ExtendedArgs, Hwnd, Hz,
    MediaItemTake, PositionInSeconds, ReaperFunctionError, ReaperFunctionResult, ReaperStr,
    ReaperString,
};
use reaper_low::raw::{PCM_source, PCM_source_peaktransfer_t, PCM_source_transfer_t, HWND__};
use std::borrow::Borrow;
use std::error::Error;
use std::fmt;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr::{null, null_mut, NonNull};

/// Pointer to a PCM source transfer.
//
// Case 2: Internals exposed: yes | vtable: no
// ===========================================
#[derive(PartialEq, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedPcmSourceTransfer(raw::PCM_source_transfer_t);

impl BorrowedPcmSourceTransfer {
    /// Returns the pointer to this source transfer.
    pub fn as_ptr(&self) -> NonNull<raw::PCM_source_transfer_t> {
        NonNull::from(&self.0)
    }

    /// Returns the list of MIDI events to be filled.
    pub fn midi_event_list(&self) -> &BorrowedMidiEventList {
        if self.0.midi_events.is_null() {
            panic!("PCM source transfer didn't provide MIDI event list");
        }
        BorrowedMidiEventList::ref_cast(unsafe { &*self.0.midi_events })
    }
}

/// Pointer to a PCM source peak transfer.
//
// Case 2: Internals exposed: yes | vtable: no
// ===========================================
#[derive(PartialEq, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedPcmSourcePeakTransfer(raw::PCM_source_peaktransfer_t);

impl BorrowedPcmSourcePeakTransfer {
    /// Returns the pointer to this source peak transfer.
    pub fn as_ptr(&self) -> NonNull<raw::PCM_source_peaktransfer_t> {
        NonNull::from(&self.0)
    }
}

/// Pointer to a project state context.
//
// Case 3: Internals exposed: no | vtable: yes
// ===========================================
#[derive(Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedProjectStateContext(raw::ProjectStateContext);

impl BorrowedProjectStateContext {
    /// Returns the pointer to this context.
    pub fn as_ptr(&self) -> NonNull<raw::ProjectStateContext> {
        NonNull::from(&self.0)
    }
}

/// Owned PCM source.
///
/// This PCM source automatically destroys the associated C++ `PCM_source` when dropped.
#[derive(Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct OwnedPcmSource(pub(crate) PcmSource);

impl OwnedPcmSource {
    /// Takes ownership of the given source.
    ///
    /// # Safety
    ///
    /// You must guarantee that the given source is currently owner-less, otherwise double-free or
    /// use-after-free can occur.
    pub unsafe fn new_unchecked(inner: PcmSource) -> Self {
        Self(inner)
    }

    /// Returns the inner pointer **without** destroying the source.
    ///
    /// # Safety
    ///
    /// You can run into a memory leak or crash if you don't manage the lifetime of the returned
    /// source correctly.  
    pub unsafe fn leak(self) -> PcmSource {
        let manually_dropped = std::mem::ManuallyDrop::new(self);
        manually_dropped.0
    }
}

impl Drop for OwnedPcmSource {
    fn drop(&mut self) {
        unsafe {
            reaper_low::delete_cpp_pcm_source(self.0.into_inner());
        }
    }
}

impl AsRef<BorrowedPcmSource> for OwnedPcmSource {
    fn as_ref(&self) -> &BorrowedPcmSource {
        unsafe { self.0.as_ref() }
    }
}

impl Borrow<BorrowedPcmSource> for OwnedPcmSource {
    fn borrow(&self) -> &BorrowedPcmSource {
        self.as_ref()
    }
}

impl Deref for OwnedPcmSource {
    type Target = BorrowedPcmSource;

    fn deref(&self) -> &BorrowedPcmSource {
        self.as_ref()
    }
}

impl Clone for OwnedPcmSource {
    fn clone(&self) -> OwnedPcmSource {
        self.duplicate()
            .expect("this source doesn't support duplication")
    }
}

/// Pointer to a PCM source.
//
// Copy and clone because it's one of the types for which `validate_ptr_2` can be called to check
// pointer validity.
//
// Case 3: Internals exposed: no | vtable: yes
// ===========================================
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct PcmSource(pub(crate) NonNull<raw::PCM_source>);

impl PcmSource {
    /// Creates this pointer by wrapping the given non-null pointer to the low-level PCM source.
    pub fn new(raw: NonNull<raw::PCM_source>) -> Self {
        Self(raw)
    }

    /// Returns the wrapped non-null pointer to the low-level PCM source.
    pub fn into_inner(self) -> NonNull<raw::PCM_source> {
        self.0
    }

    /// Returns a pointer to the low-level PCM source.
    pub fn to_raw(&self) -> *mut raw::PCM_source {
        self.0.as_ptr()
    }

    /// Turns this pointer into a reference.
    ///
    /// # Safety
    ///
    /// For all we know this pointer might be stale. For PCM sources used within REAPER, you can
    /// use [`validate_ptr_2()`] to make sure the pointer is (still) valid.
    ///
    /// [`validate_ptr_2()`]: struct.Reaper.html#method.validate_ptr_2
    pub unsafe fn as_ref(&self) -> &BorrowedPcmSource {
        BorrowedPcmSource::ref_cast(&*self.0.as_ref())
    }
}

/// Borrowed (reference-only) PCM source.
#[derive(Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedPcmSource(raw::PCM_source);

impl BorrowedPcmSource {
    /// Returns the pointer.
    pub fn as_ptr(&self) -> PcmSource {
        PcmSource(NonNull::from(&self.0))
    }

    /// Duplicates this source.
    pub fn duplicate(&self) -> Option<OwnedPcmSource> {
        let raw_duplicate = self.0.Duplicate();
        NonNull::new(raw_duplicate)
            .map(PcmSource)
            .map(OwnedPcmSource)
    }

    /// Returns if this source is available.
    pub fn is_available(&self) -> bool {
        self.0.IsAvailable()
    }

    /// If called with false, closes files etc.
    pub fn set_available(&self, available: bool) {
        self.0.SetAvailable(available);
    }

    /// Grants temporary access to the type of this source.
    ///
    /// This type should not be empty but if a third-party source provider doesn't get it right,
    /// this can still happen. An empty string is also used as fallback if the third-party source
    /// returns a null pointer.
    pub fn get_type<R>(&self, use_type: impl FnOnce(&ReaperStr) -> R) -> R {
        let t = unsafe { self.get_type_unchecked() };
        use_type(t)
    }

    /// Returns the type of this source.
    ///
    /// # Safety
    ///
    /// Returned string's lifetime is unbounded.
    pub unsafe fn get_type_unchecked(&self) -> &ReaperStr {
        let ptr = self.0.GetType();
        create_passing_c_str(ptr).unwrap_or_default()
    }

    /// Grants temporary access to the file of this source.
    ///
    /// `None` is a valid result. In that case it's not purely a file. Takes care of converting an
    /// empty path to `None`.
    pub fn get_file_name<R>(&self, use_file: impl FnOnce(Option<&Path>) -> R) -> R {
        let file_name = unsafe { self.get_file_name_unchecked() };
        let path = if let Some(n) = file_name {
            Some(Path::new(n.to_str()))
        } else {
            None
        };
        use_file(path)
    }

    /// Returns the file of this source.
    ///
    /// `None` is a valid result. In that case it's not purely a file. Takes care of converting an
    /// empty path to `None`.
    ///
    /// # Safety
    ///
    /// Returned string's lifetime is unbounded.
    pub unsafe fn get_file_name_unchecked(&self) -> Option<&ReaperStr> {
        let ptr = self.0.GetFileName();
        let file_name = create_passing_c_str(ptr);
        if let Some(reaper_str) = file_name {
            if reaper_str.to_str().is_empty() {
                None
            } else {
                Some(reaper_str)
            }
        } else {
            None
        }
    }

    /// Returns `true` if supported. Only call when offline.
    pub fn set_file_name(&self, new_file_name: Option<&Path>) -> bool {
        if let Some(p) = new_file_name {
            let file_name_str = p.to_str().expect("file name is not valid UTF-8");
            let file_name_reaper_string = ReaperString::from_str(file_name_str);
            unsafe { self.0.SetFileName(file_name_reaper_string.as_ptr()) }
        } else {
            unsafe { self.0.SetFileName(null()) }
        }
    }

    /// Returns the parent source, if any.
    pub fn get_source(&self) -> Option<PcmSource> {
        let ptr = self.0.GetSource();
        NonNull::new(ptr).map(PcmSource)
    }

    pub fn set_source(&self, source: Option<PcmSource>) {
        let ptr = source.map(|s| s.to_raw()).unwrap_or(null_mut());
        unsafe {
            self.0.SetSource(ptr);
        }
    }

    /// Returns number of channels.
    pub fn get_num_channels(&self) -> Option<u32> {
        let n = self.0.GetNumChannels();
        if n < 0 {
            return None;
        }
        Some(n as _)
    }

    /// Returns preferred sample rate. If `None` then it is assumed to be silent (or MIDI).
    pub fn get_sample_rate(&self) -> Option<Hz> {
        let r = self.0.GetSampleRate();
        if r < 1.0 {
            return None;
        }
        Some(Hz::new(r))
    }

    /// Returns the length of this source.
    ///
    /// # Errors
    ///
    /// Returns an error if this source doesn't return a valid duration.
    pub fn get_length(&self) -> ReaperFunctionResult<DurationInSeconds> {
        let length = self.0.GetLength();
        if length < 0.0 {
            return Err(ReaperFunctionError::new("source doesn't return length"));
        }
        Ok(DurationInSeconds::new(length))
    }

    /// Returns length in beats if supported.
    pub fn get_length_beats(&self) -> Option<DurationInBeats> {
        let length = self.0.GetLengthBeats();
        if length < 0.0 {
            return None;
        }
        Some(DurationInBeats::new(length))
    }

    /// Returns bits/sample, if available. Only used for metadata purposes, since everything
    /// returns as doubles anyway.
    pub fn get_bits_per_sample(&self) -> u32 {
        self.0.GetBitsPerSample() as u32
    }

    /// Returns `None` if not supported.
    pub fn get_preferred_position(&self) -> Option<PositionInSeconds> {
        let pos = self.0.GetPreferredPosition();
        if pos < 0.0 {
            return None;
        }
        Some(PositionInSeconds::new(pos))
    }

    /// Unstable!!!
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn properties_window(&self, parent_window: Option<Hwnd>) -> i32 {
        let ptr = parent_window.map(|w| w.as_ptr()).unwrap_or(null_mut());
        self.0.PropertiesWindow(ptr)
    }

    /// Unstable!!!
    ///
    /// # Safety
    ///
    /// API still unstable.
    pub unsafe fn get_samples(&self, block: &BorrowedPcmSourceTransfer) {
        self.0.GetSamples(block.as_ptr().as_ptr());
    }

    /// Unstable!!!
    ///
    /// # Safety
    ///
    /// API still unstable.
    pub unsafe fn get_peak_info(&self, block: &BorrowedPcmSourcePeakTransfer) {
        self.0.GetPeakInfo(block.as_ptr().as_ptr());
    }

    /// Unstable!!!
    ///
    /// # Safety
    ///
    /// API still unstable.
    pub unsafe fn save_state(&self, context: &BorrowedProjectStateContext) {
        self.0.SaveState(context.as_ptr().as_ptr());
    }

    /// Unstable!!!
    ///
    /// # Safety
    ///
    /// API still unstable.
    pub unsafe fn load_state(
        &self,
        first_line: &ReaperStr,
        context: &BorrowedProjectStateContext,
    ) -> Result<(), Box<dyn Error>> {
        let res = self
            .0
            .LoadState(first_line.as_ptr(), context.as_ptr().as_ptr());
        if res == -1 {
            return Err("load state failed".into());
        }
        Ok(())
    }

    /// Builds peaks for files.
    pub fn peaks_clear(&self, delete_file: bool) {
        self.0.Peaks_Clear(delete_file);
    }

    /// Returns `true` if building is opened, otherwise it may mean building isn't necessary.
    pub fn peaks_build_begin(&self) -> bool {
        self.0.PeaksBuild_Begin() != 0
    }

    /// Returns `true` if building should continue.
    pub fn peaks_build_run(&self) -> bool {
        self.0.PeaksBuild_Run() != 0
    }

    /// Call when done.
    pub fn peaks_build_finish(&self) {
        self.0.PeaksBuild_Finish();
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass invalid pointers.
    pub unsafe fn extended(
        &self,
        call: i32,
        parm_1: *mut c_void,
        parm_2: *mut c_void,
        parm_3: *mut c_void,
    ) -> i32 {
        self.0.Extended(call, parm_1, parm_2, parm_3)
    }

    /// Unstable!!!
    ///
    /// If this source represents pooled MIDI data, this will return information about it.
    ///
    /// # Errors
    ///
    /// Returns an error if not supported.
    pub fn ext_get_pooled_midi_id(&self) -> ReaperFunctionResult<ExtGetPooledMidiIdResult> {
        let mut user_count: MaybeUninit<i32> = MaybeUninit::zeroed();
        let mut first_user: MaybeUninit<*mut raw::MediaItem_Take> = MaybeUninit::zeroed();
        let (id, supported) = with_string_buffer(40, |buffer, _| unsafe {
            self.0.Extended(
                raw::PCM_SOURCE_EXT_GETPOOLEDMIDIID as _,
                buffer as _,
                user_count.as_mut_ptr() as _,
                first_user.as_mut_ptr() as _,
            )
        });
        if supported == 0 {
            return Err(ReaperFunctionError::new(
                "PCM_SOURCE_EXT_GETPOOLEDMIDIID not supported by source",
            ));
        }
        Ok(ExtGetPooledMidiIdResult {
            id,
            // user_count: user_count.assume_init() as _,
            user_count: unsafe { user_count.assume_init() },
            first_user: {
                let ptr = unsafe { first_user.assume_init() };
                NonNull::new(ptr).unwrap()
            },
        })
    }

    /// Writes the content of this source to the given file.
    ///
    /// Only currently supported by MIDI but in theory any source could support this.
    ///
    /// # Errors
    ///
    /// Returns an error if not supported.
    pub fn ext_export_to_file(&self, file_name: &Path) -> ReaperFunctionResult<()> {
        let file_name_str = file_name.to_str().expect("file name is not valid UTF-8");
        let file_name_reaper_string = ReaperString::from_str(file_name_str);
        let supported = unsafe {
            self.0.Extended(
                raw::PCM_SOURCE_EXT_EXPORTTOFILE as _,
                file_name_reaper_string.as_ptr() as _,
                null_mut(),
                null_mut(),
            )
        };
        if supported == 0 {
            return Err(ReaperFunctionError::new(
                "PCM_SOURCE_EXT_EXPORTTOFILE not supported by source",
            ));
        }
        Ok(())
    }

    // /// Opens the editor for this source.
    // ///
    // /// # Errors
    // ///
    // /// Returns an error if not supported.
    // pub unsafe fn ext_open_editor(&self, hwnd: Hwnd, track_index: u32) ->
    // ReaperFunctionResult<()> {     let supported = self.0.as_ref().Extended(
    //         raw::PCM_SOURCE_EXT_OPENEDITOR as _,
    //         hwnd.as_ptr() as _,
    //         track_index as isize as _,
    //         null_mut(),
    //     );
    //     if supported == 0 {
    //         return Err(ReaperFunctionError::new(
    //             "PCM_SOURCE_EXT_OPENEDITOR not supported by source",
    //         ));
    //     }
    //     Ok(())
    // }
}

impl ToOwned for BorrowedPcmSource {
    type Owned = OwnedPcmSource;

    fn to_owned(&self) -> OwnedPcmSource {
        self.duplicate().expect("source not cloneable")
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ExtGetPooledMidiIdResult {
    /// A GUID string with braces.
    // TODO-high-unstable Can this be empty?
    pub id: ReaperString,
    /// Number of takes which use this pooled MIDI data.
    // TODO-high-unstable Improve type
    pub user_count: i32,
    // TODO-high-unstable Can this be empty?
    pub first_user: MediaItemTake,
}

/// Consumers can implement this trait in order to provide own PCM source types.
pub trait CustomPcmSource {
    fn duplicate(&mut self) -> Option<OwnedPcmSource>;

    fn is_available(&mut self) -> bool;

    /// If called with false, close files etc.
    ///
    /// Optional.
    fn set_available(&mut self, args: SetAvailableArgs) {
        let _ = args;
    }

    fn get_type(&mut self) -> &ReaperStr;

    /// Return `None` if no file name (not purely a file).
    //
    // We can't let this return an `Option<&Path>` because we can't convert it into a C string slice
    // without conversion. It *must* be a reference to something that we own, that's simply how the
    // `PCM_source` interface is designed.
    fn get_file_name(&mut self) -> Option<&ReaperStr> {
        None
    }

    /// Return `true` if supported. This will only be called when offline.
    fn set_file_name(&mut self, args: SetFileNameArgs) -> bool;

    /// Return parent source, if any.
    fn get_source(&mut self) -> Option<PcmSource> {
        None
    }

    fn set_source(&mut self, args: SetSourceArgs) {
        let _ = args;
    }

    /// Return number of channels.
    fn get_num_channels(&mut self) -> Option<u32>;

    /// Return preferred sample rate. If `None` then it is assumed to be silent (or MIDI).
    fn get_sample_rate(&mut self) -> Option<Hz>;

    /// Length in seconds.
    fn get_length(&mut self) -> DurationInSeconds;

    /// Length in beats if supported.
    fn get_length_beats(&mut self) -> Option<DurationInBeats> {
        None
    }

    /// Return bits/sample, if available. Only used for metadata purposes, since everything
    /// returns as doubles anyway.
    fn get_bits_per_sample(&mut self) -> u32 {
        0
    }

    /// Return `None` if not supported.
    fn get_preferred_position(&mut self) -> Option<PositionInSeconds> {
        None
    }

    /// Unstable!!!
    // TODO-high-unstable Not sure what the return value means. Maybe use extensible enum.
    fn properties_window(&mut self, args: PropertiesWindowArgs) -> i32;

    fn get_samples(&mut self, args: GetSamplesArgs);

    fn get_peak_info(&mut self, args: GetPeakInfoArgs);

    fn save_state(&mut self, args: SaveStateArgs);

    fn load_state(&mut self, args: LoadStateArgs) -> Result<(), Box<dyn Error>>;

    /// Called by the peaks building UI to build peaks for files.
    fn peaks_clear(&mut self, args: PeaksClearArgs);

    /// Unstable!!!
    /// Return `true` if building is opened, otherwise it may mean building isn't necessary.
    // TODO-high-unstable Use extensible enum as return value.
    fn peaks_build_begin(&mut self) -> bool;

    /// Unstable!!!
    /// Return `true` if building should continue.
    // TODO-high-unstable Use extensible enum as return value.
    fn peaks_build_run(&mut self) -> bool;

    /// Called when done.
    fn peaks_build_finish(&mut self);

    /// Generic method which is called for many kinds of events. Prefer implementing the type-safe
    /// `ext_` methods instead!
    ///
    /// *reaper-rs* calls this method only if you didn't process the event already in one of the
    /// `ext_` methods. The meaning of the return value depends on the particular event type
    /// ([`args.call`]). In any case, returning 0 means that the event has not been handled.
    ///
    /// # Safety
    ///
    /// Implementing this is unsafe because you need to deal with raw pointers.
    ///
    /// [`args.call`]: struct.ExtendedArgs.html#structfield.call
    unsafe fn extended(&mut self, args: ExtendedArgs) -> i32 {
        let _ = args;
        0
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SetAvailableArgs {
    pub is_available: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SetFileNameArgs<'a> {
    pub new_file_name: Option<&'a Path>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SetSourceArgs {
    pub source: Option<PcmSource>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PropertiesWindowArgs {
    pub parent_window: Option<Hwnd>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GetSamplesArgs<'a> {
    pub block: &'a BorrowedPcmSourceTransfer,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GetPeakInfoArgs<'a> {
    pub block: &'a BorrowedPcmSourcePeakTransfer,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SaveStateArgs<'a> {
    pub context: &'a BorrowedProjectStateContext,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct LoadStateArgs<'a> {
    pub first_line: &'a ReaperStr,
    pub context: &'a BorrowedProjectStateContext,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PeaksClearArgs {
    pub delete_file: bool,
}

#[derive(Debug)]
struct PcmSourceAdapter<S: CustomPcmSource> {
    // Unlike `DelegatingControlSurface` we don't use a `Box` here because we don't need to store
    // multiple PCM sources of different types in one list in the medium-level API. We also don't
    // have the same "Give ownership to REAPER and get it back at some point" kind of usage. PCM
    // sources are more flexible in usage, e.g. it can also make sense to share them and
    // synchronize access via mutex (e.g. using the preview register API). Of course, using
    // monomorphization instead of dynamic dispatch also helps with performance - because PCM
    // sources are primarily used by in real-time threads!
    delegate: S,
}

impl<S: CustomPcmSource> PcmSourceAdapter<S> {
    pub fn new(delegate: S) -> Self {
        Self { delegate }
    }
}

impl<S: CustomPcmSource> reaper_low::PCM_source for PcmSourceAdapter<S> {
    fn Duplicate(&mut self) -> *mut PCM_source {
        self.delegate
            .duplicate()
            .map(|s| {
                let leaked = unsafe { s.leak() };
                leaked.to_raw()
            })
            .unwrap_or(null_mut())
    }

    fn IsAvailable(&mut self) -> bool {
        self.delegate.is_available()
    }

    fn SetAvailable(&mut self, avail: bool) {
        self.delegate.set_available(SetAvailableArgs {
            is_available: avail,
        });
    }

    fn GetType(&mut self) -> *const i8 {
        self.delegate.get_type().as_ptr()
    }

    fn GetFileName(&mut self) -> *const i8 {
        self.delegate
            .get_file_name()
            .map(|s| s.as_ptr())
            .unwrap_or(null())
    }

    fn SetFileName(&mut self, newfn: *const i8) -> bool {
        let new_file_name = if let Some(reaper_str) = unsafe { create_passing_c_str(newfn) } {
            let s = reaper_str.to_str();
            Some(Path::new(s))
        } else {
            None
        };
        let args = SetFileNameArgs { new_file_name };
        self.delegate.set_file_name(args)
    }

    fn GetSource(&mut self) -> *mut PCM_source {
        self.delegate
            .get_source()
            .map(|s| s.to_raw())
            .unwrap_or(null_mut())
    }

    fn SetSource(&mut self, src: *mut PCM_source) {
        let args = SetSourceArgs {
            source: NonNull::new(src).map(PcmSource),
        };
        self.delegate.set_source(args);
    }

    fn GetNumChannels(&mut self) -> i32 {
        self.delegate
            .get_num_channels()
            .map(|n| n as i32)
            .unwrap_or(-1)
    }

    fn GetSampleRate(&mut self) -> f64 {
        self.delegate
            .get_sample_rate()
            .map(|r| r.get())
            .unwrap_or_default()
    }

    fn GetLength(&mut self) -> f64 {
        self.delegate.get_length().get()
    }

    fn GetLengthBeats(&mut self) -> f64 {
        self.delegate
            .get_length_beats()
            .map(|l| l.get())
            .unwrap_or(-1.0)
    }

    fn GetBitsPerSample(&mut self) -> i32 {
        self.delegate.get_bits_per_sample() as i32
    }

    fn GetPreferredPosition(&mut self) -> f64 {
        self.delegate
            .get_preferred_position()
            .map(|p| p.get())
            .unwrap_or(-1.0)
    }

    fn PropertiesWindow(&mut self, hwndParent: *mut HWND__) -> i32 {
        let args = PropertiesWindowArgs {
            parent_window: NonNull::new(hwndParent),
        };
        self.delegate.properties_window(args)
    }

    fn GetSamples(&mut self, block: *mut PCM_source_transfer_t) {
        if block.is_null() {
            panic!("called PCM_source::GetSamples() with null block")
        }
        let block = BorrowedPcmSourceTransfer::ref_cast(unsafe { &*block });
        let args = GetSamplesArgs { block };
        self.delegate.get_samples(args);
    }

    fn GetPeakInfo(&mut self, block: *mut PCM_source_peaktransfer_t) {
        if block.is_null() {
            panic!("called PCM_source::GetPeakInfo() with null block")
        }
        let block = BorrowedPcmSourcePeakTransfer::ref_cast(unsafe { &*block });
        let args = GetPeakInfoArgs { block };
        self.delegate.get_peak_info(args);
    }

    fn SaveState(&mut self, ctx: *mut raw::ProjectStateContext) {
        if ctx.is_null() {
            panic!("called PCM_source::SaveState() with null block")
        }
        let context = BorrowedProjectStateContext::ref_cast(unsafe { &*ctx });
        let args = SaveStateArgs { context };
        self.delegate.save_state(args);
    }

    fn LoadState(&mut self, firstline: *const i8, ctx: *mut raw::ProjectStateContext) -> i32 {
        if ctx.is_null() {
            panic!("called PCM_source::LoadState() with null block")
        }
        let context = BorrowedProjectStateContext::ref_cast(unsafe { &*ctx });
        let first_line = unsafe { create_passing_c_str(firstline) };
        let args = LoadStateArgs {
            first_line: first_line.unwrap_or_default(),
            context,
        };
        if self.delegate.load_state(args).is_ok() {
            0
        } else {
            -1
        }
    }

    fn Peaks_Clear(&mut self, deleteFile: bool) {
        let args = PeaksClearArgs {
            delete_file: deleteFile,
        };
        self.delegate.peaks_clear(args);
    }

    fn PeaksBuild_Begin(&mut self) -> i32 {
        let opened = self.delegate.peaks_build_begin();
        if opened { 1 } else { 0 }
    }

    fn PeaksBuild_Run(&mut self) -> i32 {
        let more = self.delegate.peaks_build_run();
        if more { 1 } else { 0 }
    }

    fn PeaksBuild_Finish(&mut self) {
        self.delegate.peaks_build_finish();
    }

    fn Extended(
        &mut self,
        call: i32,
        parm1: *mut c_void,
        parm2: *mut c_void,
        parm3: *mut c_void,
    ) -> i32 {
        unsafe {
            self.delegate.extended(ExtendedArgs {
                call,
                parm_1: parm1,
                parm_2: parm2,
                parm_3: parm3,
            })
        }
    }
}

/// Either a REAPER PCM source or a custom one.
#[derive(Debug)]
pub enum FlexibleOwnedPcmSource {
    Reaper(OwnedPcmSource),
    Custom(CustomOwnedPcmSource),
}

impl AsRef<BorrowedPcmSource> for FlexibleOwnedPcmSource {
    fn as_ref(&self) -> &BorrowedPcmSource {
        match self {
            FlexibleOwnedPcmSource::Reaper(s) => s.as_ref(),
            FlexibleOwnedPcmSource::Custom(s) => s.as_ref(),
        }
    }
}

/// Represents an owned PCM source that is backed by a Rust [`CustomPcmSource`] trait
/// implementation.
///
/// [`CustomPcmSource`]: trait.CustomPcmSource.html
pub struct CustomOwnedPcmSource {
    // Those 2 belong together. `cpp_source` without `rust_source` = crash. Never let them apart!
    cpp_source: OwnedPcmSource,
    /// Never read but important to keep in memory.
    _rust_source: Box<Box<dyn reaper_low::PCM_source>>,
}

impl fmt::Debug for CustomOwnedPcmSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CustomOwnedPcmSource")
            .field("cpp_source", &self.cpp_source)
            .finish()
    }
}

impl AsRef<BorrowedPcmSource> for CustomOwnedPcmSource {
    fn as_ref(&self) -> &BorrowedPcmSource {
        self.cpp_source.as_ref()
    }
}

/// Unstable!!!
///
/// Creates a REAPER PCM source for the given custom Rust implementation and returns it.
//
// TODO-high-unstable Think of a good name.
pub fn create_custom_owned_pcm_source<S: CustomPcmSource + 'static>(
    custom_source: S,
) -> CustomOwnedPcmSource {
    let adapter = PcmSourceAdapter::new(custom_source);
    // Create the C++ counterpart source (we need to box the Rust side twice in order to obtain
    // a thin pointer for passing it to C++ as callback target).
    let rust_source: Box<Box<dyn reaper_low::PCM_source>> = Box::new(Box::new(adapter));
    let thin_ptr_to_adapter: NonNull<_> = rust_source.as_ref().into();
    let raw_cpp_source = unsafe { create_cpp_to_rust_pcm_source(thin_ptr_to_adapter) };
    let cpp_source = unsafe { OwnedPcmSource::new_unchecked(PcmSource::new(raw_cpp_source)) };
    CustomOwnedPcmSource {
        cpp_source,
        _rust_source: rust_source,
    }
}
