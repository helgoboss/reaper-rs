use crate::{
    AnyThread, Hinstance, Hwnd, MainThreadOnly, MediaItemTake, MediaTrack, ReaProject,
    ReaperStringArg, TrackFxLocation,
};
use reaper_low::raw;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr::{null_mut, NonNull};
use vst::api::AEffect;

/// This represents the context in which this REAPER plug-in runs.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PluginContext<'a, UsageScope> {
    low: &'a reaper_low::PluginContext,
    p: PhantomData<UsageScope>,
}

impl<'a, UsageScope> PluginContext<'a, UsageScope> {
    pub(crate) fn new(low: &'a reaper_low::PluginContext) -> PluginContext<'a, UsageScope> {
        PluginContext {
            low,
            p: PhantomData,
        }
    }

    /// Returns a generic API function by its name.
    ///
    /// The returned `*mut c_void` function pointer must be cast to a function pointer type via
    /// [`transmute()`](std::mem::transmute). Please note that it can be `null` if the desired
    /// function is not registered.
    pub fn get_func<'b>(&self, name: impl Into<ReaperStringArg<'b>>) -> *mut c_void
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        unsafe { self.low.GetFunc(name.into().as_ptr()) }
    }

    /// On Windows, this returns the `HINSTANCE` passed to `DllMain` (VST plug-ins) or
    /// `ReaperPluginEntry` (extension plug-ins).
    ///
    /// The returned `HINSTANCE` represents the handle of the module (DLL) containing the plug-in.
    ///
    /// On Linux, this returns `None`.
    pub fn h_instance(&self) -> Option<Hinstance>
    where
        UsageScope: AnyThread,
    {
        NonNull::new(self.low.h_instance())
    }

    /// Returns whether we are currently in the main thread.
    pub fn is_in_main_thread(&self) -> bool
    where
        UsageScope: AnyThread,
    {
        self.low.is_in_main_thread()
    }

    /// Returns the type-specific plug-in context.
    pub fn type_specific(&self) -> TypeSpecificPluginContext
    where
        UsageScope: AnyThread,
    {
        use reaper_low::TypeSpecificPluginContext::*;
        match self.low.type_specific() {
            Extension(low) => TypeSpecificPluginContext::Extension(ExtensionPluginContext { low }),
            Vst(low) => TypeSpecificPluginContext::Vst(VstPluginContext { low }),
        }
    }

    fn require_main_thread(&self)
    where
        UsageScope: MainThreadOnly,
    {
        assert!(
            self.is_in_main_thread(),
            "called main-thread-only function from wrong thread"
        )
    }
}

/// Additional stuff available in the plug-in context specific to a certain plug-in type.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TypeSpecificPluginContext<'a> {
    /// This is an extension plug-in.
    Extension(ExtensionPluginContext<'a>),
    /// This is a VST plug-in.
    Vst(VstPluginContext<'a>),
}

/// Additional data available in the context of extension plug-ins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ExtensionPluginContext<'a> {
    low: &'a reaper_low::ExtensionPluginContext,
}

impl<'a> ExtensionPluginContext<'a> {
    /// Returns the caller version from `reaper_plugin_info_t`.
    pub fn caller_version(self) -> i32 {
        self.low.caller_version()
    }

    /// Returns the main window from `reaper_plugin_info_t`.
    pub fn hwnd_main(self) -> Hwnd {
        NonNull::new(self.low.hwnd_main()).expect("plug-in info doesn't contain main window handle")
    }
}

/// Additional data available in the context of VST plug-ins.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct VstPluginContext<'a> {
    low: &'a reaper_low::VstPluginContext,
}

impl<'a> VstPluginContext<'a> {
    /// Generic host callback function for communicating with REAPER from the VST plug-in.
    ///
    /// This is just a pass-through to the VST host callback.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn host_callback(
        self,
        effect: *mut AEffect,
        opcode: i32,
        index: i32,
        value: isize,
        ptr: *mut c_void,
        opt: f32,
    ) -> isize {
        self.low
            .host_callback(effect, opcode, index, value, ptr, opt)
    }

    /// Returns the REAPER project in which the given VST plug-in is running.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn request_containing_project(self, effect: NonNull<AEffect>) -> ReaProject {
        let ptr = self.request_context(effect, 3) as *mut raw::ReaProject;
        NonNull::new(ptr).expect("a VST should always run in the context of a project")
    }

    /// Returns the REAPER track on which the given VST plug-in resides.
    ///
    /// Returns `None` if the given plug-in is not running as track FX.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn request_containing_track(self, effect: NonNull<AEffect>) -> Option<MediaTrack> {
        let ptr = self.request_context(effect, 1) as *mut raw::MediaTrack;
        NonNull::new(ptr)
    }

    /// Returns the REAPER take in which the given VST plug-in resides.
    ///
    /// Returns `None` if the given plug-in is not running as take FX.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn request_containing_take(self, effect: NonNull<AEffect>) -> Option<MediaItemTake> {
        let ptr = self.request_context(effect, 2) as *mut raw::MediaItem_Take;
        NonNull::new(ptr)
    }

    /// Returns the location in the FX chain at which the given VST plug-in currently resides.
    ///
    /// Supported since REAPER v6.11. Returns `None` if not supported.
    ///
    /// Don't let the fact that this returns a [`TrackFxLocation`] confuse you. It also works for
    /// take and monitoring FX. If this is a take FX, it will return the index as variant
    /// [`NormalFxChain`]. If this is a monitoring FX, it will return the index as variant
    /// [`InputFxChain`].
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    ///
    /// [`TrackFxLocation`]: enum.TrackFxLocation.html
    /// [`NormalFxChain`]: enum.TrackFxLocation.html#variant.NormalFxChain
    /// [`InputFxChain`]: enum.TrackFxLocation.html#variant.InputFxChain
    pub unsafe fn request_containing_fx_location(
        self,
        effect: NonNull<AEffect>,
    ) -> Option<TrackFxLocation> {
        let result = self.request_context(effect, 6) as i32;
        if result <= 0 {
            // Not supported
            return None;
        }
        let raw_index = result - 1;
        Some(TrackFxLocation::from_raw(raw_index))
    }

    /// Returns the channel count of the REAPER track which contains the given VST plug-in.
    ///
    /// Returns 0 if the given plug-in is not running as track FX.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn request_containing_track_channel_count(self, effect: NonNull<AEffect>) -> i32 {
        self.request_context(effect, 5) as i32
    }

    unsafe fn request_context(self, effect: NonNull<AEffect>, request: isize) -> isize {
        #[allow(overflowing_literals)]
        self.host_callback(
            effect.as_ptr(),
            0xdead_beef,
            0xdead_f00e,
            request,
            null_mut(),
            0.0,
        )
    }
}
