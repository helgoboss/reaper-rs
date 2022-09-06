use crate::{Project, Reaper};
use reaper_low::{
    copy_heap_buf_to_buf, create_heap_buf, load_pcm_source_state_from_buf,
    save_pcm_source_state_to_heap_buf,
};
use reaper_medium::{
    BorrowedPcmSource, Bpm, DurationInSeconds, ExtGetPooledMidiIdResult, MidiImportBehavior,
    OwnedPcmSource, PcmSource, ReaperFunctionError, ReaperStringArg,
};
use ref_cast::RefCast;
use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

/// Pointer to a PCM source that's owned and managed by REAPER.
///
/// Whenever a function is called via `Deref`, a validation check will be done. If it doesn't
/// succeed, reaper-rs will panic (better than crashing).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct ReaperSource(PcmSource);

impl ReaperSource {
    pub fn new(raw: PcmSource) -> Self {
        Self(raw)
    }

    pub fn raw(&self) -> PcmSource {
        self.0
    }

    /// This checks if the source is used within REAPER.
    pub fn is_used_by_reaper(&self) -> bool {
        Reaper::get().medium_reaper().validate_ptr(self.0)
    }

    pub fn is_valid_in_project(&self, project: Project) -> bool {
        Reaper::get()
            .medium_reaper()
            .validate_ptr_2(project.context(), self.0)
    }

    // fn make_sure_is_valid(&self) {
    //     if !self.is_used_by_reaper() {
    //         panic!("PCM source pointer is not valid anymore in REAPER")
    //     }
    // }
}

impl AsRef<BorrowedSource> for ReaperSource {
    fn as_ref(&self) -> &BorrowedSource {
        // TODO-high We can't double check if the source still exists because we only have a method
        //  to check if the source is still valid as far as REAPER knows. But that would exclude
        //  working with sources that exist but REAPER doesn't know about, i.e. our own non-item
        //  sources. We should add some logic to recognize when it's our source and when not.
        // self.make_sure_is_valid();
        BorrowedSource::ref_cast(BorrowedPcmSource::from_raw(unsafe { self.0.as_ref() }))
    }
}

impl AsMut<BorrowedSource> for ReaperSource {
    fn as_mut(&mut self) -> &mut BorrowedSource {
        // TODO-high See AsRef
        // self.make_sure_is_valid();
        BorrowedSource::ref_cast_mut(BorrowedPcmSource::from_raw_mut(unsafe { self.0.as_mut() }))
    }
}

impl Deref for ReaperSource {
    type Target = BorrowedSource;

    fn deref(&self) -> &BorrowedSource {
        self.as_ref()
    }
}

#[derive(Eq, PartialEq, Hash, Debug, RefCast)]
#[repr(transparent)]
pub struct BorrowedSource(BorrowedPcmSource);

impl BorrowedSource {
    pub fn from_raw(raw: &BorrowedPcmSource) -> &Self {
        Self::ref_cast(raw)
    }

    pub fn as_raw(&self) -> &BorrowedPcmSource {
        &self.0
    }

    pub fn file_name(&self) -> Option<PathBuf> {
        self.0.get_file_name(|path| path.map(|p| p.to_owned()))
    }

    pub fn r#type(&self) -> String {
        self.0.get_type(|t| t.to_string())
    }

    pub fn length(&self) -> Result<DurationInSeconds, ReaperFunctionError> {
        self.0.get_length()
    }

    pub fn duplicate(&self) -> Option<OwnedSource> {
        let raw_duplicate = self.0.duplicate()?;
        Some(OwnedSource::new(raw_duplicate))
    }

    // We return a medium-level source because at this point we don't know if the parent is a
    // REAPER-managed source or not.
    pub fn parent_source(&self) -> Option<PcmSource> {
        let raw = self.0.get_source()?;
        Some(raw)
    }

    // We return a medium-level source because at this point we don't know if the root is a
    // REAPER-managed source or not.
    pub fn root_source(&self) -> PcmSource {
        let mut source_ptr = self.0.as_ptr();
        loop {
            let source = BorrowedPcmSource::from_raw(unsafe { source_ptr.as_ref() });
            if let Some(parent) = source.get_source() {
                source_ptr = parent;
            } else {
                return source_ptr;
            }
        }
    }

    pub fn pooled_midi_id(&self) -> Result<ExtGetPooledMidiIdResult, ReaperFunctionError> {
        self.0.ext_get_pooled_midi_id()
    }

    pub fn remove_from_midi_pool(&self) -> Result<(), ReaperFunctionError> {
        self.0.ext_remove_from_midi_pool()
    }

    pub fn set_preview_tempo(&self, tempo: Option<Bpm>) -> Result<(), ReaperFunctionError> {
        self.0.ext_set_preview_tempo(tempo)
    }

    pub fn export_to_file(&self, file: &Path) -> Result<(), ReaperFunctionError> {
        self.0.ext_export_to_file(file)
    }

    pub fn state_chunk(&self) -> String {
        let heap_buf = create_heap_buf();
        let size = unsafe {
            save_pcm_source_state_to_heap_buf(self.0.as_ref() as *const _ as *mut _, heap_buf)
        };
        let mut buffer = vec![0u8; size as usize];
        unsafe { copy_heap_buf_to_buf(heap_buf, buffer.as_mut_ptr()) };
        // I think it's safe to assume that the content written to the buffer is made up by multiple
        // segments, each of which is a proper UTF-8-encoded line (not containing newlines or
        // carriage returns). Each segment is separated by a nul byte. So if we convert each nul
        // byte to a newline, we should obtain a proper UTF-8-encoded string (which doesn't contain
        // a trailing nul byte)!
        for b in &mut buffer {
            if *b == b'\0' {
                *b = b'\n';
            }
        }
        String::from_utf8(buffer).expect("not UTF-8")
    }

    pub fn set_state_chunk<'a>(
        &mut self,
        first_line: impl Into<ReaperStringArg<'a>>,
        chunk: String,
    ) -> Result<(), &'static str> {
        let mut buffer: Vec<u8> = chunk.into();
        for b in &mut buffer {
            if *b == b'\n' {
                *b = b'\0';
            }
        }
        let result = unsafe {
            load_pcm_source_state_from_buf(
                self.0.as_ref() as *const _ as *mut _,
                first_line.into().into_inner().as_c_str().as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len() as _,
            )
        };
        if result < 0 {
            return Err("couldn't load PCM source state");
        }
        Ok(())
    }
}

/// Owned PCM source.
#[derive(Debug)]
#[repr(transparent)]
pub struct OwnedSource(OwnedPcmSource);

impl OwnedSource {
    pub fn new(raw: OwnedPcmSource) -> Self {
        Self(raw)
    }

    pub fn into_raw(self) -> OwnedPcmSource {
        self.0
    }

    pub fn from_file(
        file: &Path,
        import_behavior: MidiImportBehavior,
    ) -> Result<Self, &'static str> {
        let raw = Reaper::get()
            .medium_reaper()
            .pcm_source_create_from_file_ex(file, import_behavior)
            .map_err(|_| "couldn't create PCM source by file")?;
        Ok(Self(raw))
    }

    pub fn from_type(source_type: &str) -> Result<Self, &'static str> {
        let raw = Reaper::get()
            .medium_reaper()
            .pcm_source_create_from_type(source_type)
            .map_err(|_| "couldn't create PCM source by type")?;
        Ok(Self(raw))
    }
}

impl AsRef<BorrowedSource> for OwnedSource {
    fn as_ref(&self) -> &BorrowedSource {
        BorrowedSource::ref_cast(self.0.as_ref())
    }
}

impl AsMut<BorrowedSource> for OwnedSource {
    fn as_mut(&mut self) -> &mut BorrowedSource {
        BorrowedSource::ref_cast_mut(self.0.as_mut())
    }
}

impl Borrow<BorrowedSource> for OwnedSource {
    fn borrow(&self) -> &BorrowedSource {
        self.as_ref()
    }
}

impl ToOwned for BorrowedSource {
    type Owned = OwnedSource;

    fn to_owned(&self) -> OwnedSource {
        self.duplicate().expect("source not cloneable")
    }
}

impl Deref for OwnedSource {
    type Target = BorrowedSource;

    fn deref(&self) -> &BorrowedSource {
        self.as_ref()
    }
}

impl DerefMut for OwnedSource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl Clone for OwnedSource {
    fn clone(&self) -> OwnedSource {
        self.duplicate()
            .expect("this source doesn't support duplication")
    }
}
