use crate::mutex::ReaperMutex;
use crate::{
    destroy_mutex_primitive, initialize_mutex_primitive, FlexibleOwnedPcmSource, MediaTrack,
    PositionInSeconds, ReaperMutexPrimitive, ReaperVolumeValue,
};
use reaper_low::raw;
use std::fmt;
use std::ptr::{null_mut, NonNull};

/// An owned preview register.
///
/// It owns PCM source, mutex and critical section (and manages its lifetime) but it doesn't own the
/// track of course.
//
// Case 2: Internals exposed: yes | vtable: no
// ===========================================
//
// A borrowed version is not necessary for now because as I see it, preview registers are *always*
// created by the consumer and never returned by REAPER itself. The only use case case would be
// interoperation with another extension but that would probably look differently anyway. If one
// day we have the need, we can introduce a borrowed version, move most methods to it and at a
// Deref implementation from owned to borrowed.
pub struct OwnedPreviewRegister {
    source: Option<FlexibleOwnedPcmSource>,
    register: raw::preview_register_t,
}

impl fmt::Debug for OwnedPreviewRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedPreviewRegister")
            .field("source", &self.source)
            .finish()
    }
}

impl OwnedPreviewRegister {
    /// Creates a preview register.
    ///
    /// Also takes care of initializing the mutex.
    pub fn new() -> OwnedPreviewRegister {
        Default::default()
    }

    pub fn src(&self) -> Option<&FlexibleOwnedPcmSource> {
        self.source.as_ref()
    }

    pub fn set_src(
        &mut self,
        src: Option<FlexibleOwnedPcmSource>,
    ) -> Option<FlexibleOwnedPcmSource> {
        let previous_source = std::mem::replace(&mut self.source, src);
        self.register.src = self
            .source
            .as_ref()
            .map(|s| s.as_ref().as_ptr().to_raw())
            .unwrap_or(null_mut());
        previous_source
    }

    pub fn volume(&self) -> ReaperVolumeValue {
        ReaperVolumeValue::new(self.register.volume)
    }

    pub fn set_volume(&mut self, volume: ReaperVolumeValue) {
        self.register.volume = volume.get()
    }

    pub fn cur_pos(&self) -> PositionInSeconds {
        PositionInSeconds::new(self.register.curpos)
    }

    pub fn set_cur_pos(&mut self, pos: PositionInSeconds) {
        self.register.curpos = pos.get();
    }

    pub fn is_looped(&self) -> bool {
        self.register.loop_
    }

    pub fn set_looped(&mut self, looped: bool) {
        self.register.loop_ = looped;
    }

    pub fn preview_track(&self) -> Option<MediaTrack> {
        NonNull::new(self.register.preview_track as *mut raw::MediaTrack)
    }

    pub fn set_preview_track(&mut self, track: Option<MediaTrack>) {
        self.register.preview_track = track.map(|t| t.as_ptr() as _).unwrap_or(null_mut());
    }

    /// Unstable!!!
    // TODO-high-unstable Improve API. This can be either a track index or a HW output channel or
    //  none. preview_track only has an effect if this is none.
    pub fn out_chan(&self) -> i32 {
        self.register.m_out_chan
    }

    pub fn set_out_chan(&mut self, value: i32) {
        self.register.m_out_chan = value;
    }
}

impl Default for OwnedPreviewRegister {
    fn default() -> Self {
        let mut register = raw::preview_register_t {
            #[cfg(windows)]
            cs: Default::default(),
            #[cfg(unix)]
            mutex: unsafe { std::mem::zeroed() },
            ..Default::default()
        };
        #[cfg(windows)]
        initialize_mutex_primitive(&mut register.cs);
        #[cfg(unix)]
        initialize_mutex_primitive(&mut register.mutex);
        Self {
            source: None,
            register,
        }
    }
}

impl Drop for OwnedPreviewRegister {
    fn drop(&mut self) {
        // The source destroys itself.
        #[cfg(windows)]
        destroy_mutex_primitive(&mut self.register.cs);
        #[cfg(unix)]
        destroy_mutex_primitive(&mut self.register.mutex);
    }
}

impl AsRef<raw::preview_register_t> for OwnedPreviewRegister {
    fn as_ref(&self) -> &raw::preview_register_t {
        &self.register
    }
}

// We want to have access to the raw register pointer even without locking the mutex. Necessary
// because of our `SharedKeeper` type bounds.
impl AsRef<raw::preview_register_t> for ReaperMutex<OwnedPreviewRegister> {
    fn as_ref(&self) -> &raw::preview_register_t {
        let data = unsafe { &*self.data.get() };
        data.as_ref()
    }
}

impl AsRef<ReaperMutexPrimitive> for OwnedPreviewRegister {
    fn as_ref(&self) -> &ReaperMutexPrimitive {
        #[cfg(windows)]
        {
            &self.register.cs
        }
        #[cfg(unix)]
        {
            &self.register.mutex
        }
    }
}
