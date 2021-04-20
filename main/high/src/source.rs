use crate::{Reaper, Take};
use reaper_low::raw::PCM_source;
use reaper_medium::{
    DurationInSeconds, ExtGetPooledMidiIdResult, MidiImportBehavior, PcmSource, ReaperFunctionError,
};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;

/// Borrowed PCM source.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Source {
    raw: PcmSource,
    is_owned: bool,
}

impl Source {
    /// Creates a source that's owned by REAPER.
    ///
    /// Whenever a function is called, a validation check will be done. If it doesn't succeed,
    /// reaper-rs will panic (better than crashing).
    pub fn from_reaper(raw: PcmSource) -> Self {
        Self {
            raw,
            is_owned: false,
        }
    }

    pub fn raw(&self) -> PcmSource {
        self.raw
    }

    pub fn file_name(&self) -> Option<PathBuf> {
        self.make_sure_is_valid();
        let path_string = unsafe { self.raw.get_file_name(|name| name.map(|n| n.to_string()))? };
        if path_string.trim().is_empty() {
            return None;
        }
        Some(path_string.into())
    }

    pub fn r#type(&self) -> String {
        self.make_sure_is_valid();
        unsafe {
            self.raw
                .get_type(|t| t.expect("PCM source has no type").to_string())
        }
    }

    pub fn length(&self) -> Option<DurationInSeconds> {
        self.make_sure_is_valid();
        unsafe { self.raw.get_length() }
    }

    pub fn duplicate(&self) -> Option<OwnedSource> {
        self.make_sure_is_valid();
        unsafe {
            let raw_duplicate = self.raw.duplicate()?;
            Some(OwnedSource::new_unchecked(raw_duplicate))
        }
    }

    pub fn is_valid_in_reaper(&self) -> bool {
        Reaper::get().medium_reaper().validate_ptr(self.raw)
    }

    pub fn parent_source(&self) -> Option<Source> {
        let raw = unsafe { self.raw.get_source()? };
        Some(Self::from_reaper(raw))
    }

    pub fn root_source(&self) -> Source {
        let mut source = *self;
        loop {
            if let Some(parent) = source.parent_source() {
                source = parent;
            } else {
                return source;
            }
        }
    }

    pub fn pooled_midi_id(&self) -> Result<ExtGetPooledMidiIdResult, ReaperFunctionError> {
        self.make_sure_is_valid();
        unsafe { self.raw.ext_get_pooled_midi_id() }
    }

    pub fn export_to_file(&self, file: &Path) -> Result<(), ReaperFunctionError> {
        self.make_sure_is_valid();
        unsafe { self.raw.ext_export_to_file(file) }
    }

    fn make_sure_is_valid(&self) {
        if !self.is_owned && !self.is_valid_in_reaper() {
            panic!("PCM source is not valid anymore")
        }
    }
}

/// Owned PCM source.
#[derive(Debug)]
pub struct OwnedSource {
    source: Source,
}

impl OwnedSource {
    pub unsafe fn new_unchecked(raw: PcmSource) -> Self {
        Self {
            source: Source {
                raw,
                is_owned: true,
            },
        }
    }

    pub fn from_file(
        file: &Path,
        import_behavior: MidiImportBehavior,
    ) -> Result<Self, &'static str> {
        unsafe {
            let raw = Reaper::get()
                .medium_reaper()
                .pcm_source_create_from_file_ex(file, import_behavior)
                .map_err(|_| "couldn't create PCM source")?;
            Ok(Self::new_unchecked(raw))
        }
    }
}

impl Drop for OwnedSource {
    fn drop(&mut self) {
        unsafe {
            Reaper::get().medium_reaper().pcm_source_destroy(self.raw);
        }
    }
}

impl Deref for OwnedSource {
    type Target = Source;

    fn deref(&self) -> &Source {
        &self.source
    }
}

impl Clone for OwnedSource {
    fn clone(&self) -> OwnedSource {
        self.duplicate()
            .expect("this source doesn't support duplication")
    }
}
