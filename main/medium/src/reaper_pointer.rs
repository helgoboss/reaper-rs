use super::{MediaItem, MediaItemTake, MediaTrack, PcmSource, ReaProject, TrackEnvelope};
use crate::{concat_c_strs, ReaperStringArg};
use c_str_macro::c_str;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_void;

/// Possible REAPER pointer types which can be passed to `Reaper::validate_ptr_2()`.
///
/// Except for the trailing asterisk, the variants are named exactly like the strings which will be
/// passed to the low-level REAPER function because the medium-level API is designed to still be
/// close to the raw REAPER API.
///
/// Please raise a reaper-rs issue if you find that an enum variant is missing!
#[derive(Clone, Debug)]
pub enum ReaperPointer<'a> {
    MediaTrack(MediaTrack),
    ReaProject(ReaProject),
    MediaItem(MediaItem),
    MediaItemTake(MediaItemTake),
    TrackEnvelope(TrackEnvelope),
    PcmSource(PcmSource),
    /// If a variant is missing in this enum, you can use this custom one as a resort. Don't
    /// include the trailing asterisk (`*`)! It will be added to the call automatically.
    Custom {
        pointer: *mut c_void,
        type_name: Cow<'a, CStr>,
    },
}

impl<'a> ReaperPointer<'a> {
    pub fn custom(pointer: *mut c_void, type_name: impl Into<ReaperStringArg<'a>>) -> Self {
        Self::Custom {
            pointer,
            type_name: type_name.into().into_inner(),
        }
    }

    pub(crate) fn as_void(&self) -> *mut c_void {
        use ReaperPointer::*;
        match self {
            MediaTrack(p) => p.as_ptr() as *mut _,
            ReaProject(p) => p.as_ptr() as *mut _,
            MediaItem(p) => p.as_ptr() as *mut _,
            MediaItemTake(p) => p.as_ptr() as *mut _,
            TrackEnvelope(p) => p.as_ptr() as *mut _,
            PcmSource(p) => p.get().as_ptr() as *mut _,
            Custom { pointer, .. } => *pointer,
        }
    }
}

macro_rules! impl_from_ptr_wrapper_to_enum {
    ($name: ident) => {
        impl<'a> From<$name> for ReaperPointer<'a> {
            fn from(p: $name) -> Self {
                ReaperPointer::$name(p)
            }
        }
    };
}

impl_from_ptr_wrapper_to_enum!(MediaTrack);
impl_from_ptr_wrapper_to_enum!(ReaProject);
impl_from_ptr_wrapper_to_enum!(MediaItem);
impl_from_ptr_wrapper_to_enum!(MediaItemTake);
impl_from_ptr_wrapper_to_enum!(TrackEnvelope);
impl_from_ptr_wrapper_to_enum!(PcmSource);

impl<'a> From<ReaperPointer<'a>> for Cow<'a, CStr> {
    fn from(value: ReaperPointer<'a>) -> Self {
        use ReaperPointer::*;
        match value {
            MediaTrack(_) => c_str!("MediaTrack*").into(),
            ReaProject(_) => c_str!("ReaProject*").into(),
            MediaItem(_) => c_str!("MediaItem*").into(),
            MediaItemTake(_) => c_str!("MediaItem_Take*").into(),
            TrackEnvelope(_) => c_str!("TrackEnvelope*").into(),
            PcmSource(_) => c_str!("PCM_source*").into(),
            Custom { pointer, type_name } => concat_c_strs(type_name.as_ref(), c_str!("*")).into(),
        }
    }
}
