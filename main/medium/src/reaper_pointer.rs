use super::{MediaItem, MediaItemTake, MediaTrack, ReaProject, TrackEnvelope};
use crate::{concat_c_strs, ReaperStringArg};
use c_str_macro::c_str;
use reaper_low::raw;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::NonNull;

/// Validatable REAPER pointer.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ReaperPointer<'a> {
    MediaTrack(MediaTrack),
    ReaProject(ReaProject),
    MediaItem(MediaItem),
    MediaItemTake(MediaItemTake),
    TrackEnvelope(TrackEnvelope),
    PcmSource(NonNull<raw::PCM_source>),
    /// If a variant is missing in this enum, you can use this custom one as a resort.
    ///
    /// Use [`custom()`] to create this variant.
    ///
    /// [`custom()`]: #method.custom
    Custom {
        type_name: Cow<'a, CStr>,
        pointer: *mut c_void,
    },
}

impl<'a> ReaperPointer<'a> {
    /// Convenience function for creating a [`Custom`] pointer.
    ///
    /// **Don't** include the trailing asterisk (`*`)! It will be added automatically.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(pointer: *mut c_void, type_name: impl Into<ReaperStringArg<'a>>) -> Self {
        Self::Custom {
            pointer,
            type_name: type_name.into().into_inner(),
        }
    }

    pub(crate) fn key_into_raw(self) -> Cow<'a, CStr> {
        use ReaperPointer::*;
        match self {
            MediaTrack(_) => c_str!("MediaTrack*").into(),
            ReaProject(_) => c_str!("ReaProject*").into(),
            MediaItem(_) => c_str!("MediaItem*").into(),
            MediaItemTake(_) => c_str!("MediaItem_Take*").into(),
            TrackEnvelope(_) => c_str!("TrackEnvelope*").into(),
            PcmSource(_) => c_str!("PCM_source*").into(),
            Custom {
                pointer: _,
                type_name,
            } => concat_c_strs(type_name.as_ref(), c_str!("*")).into(),
        }
    }

    pub(crate) fn ptr_as_void(&self) -> *mut c_void {
        use ReaperPointer::*;
        match self {
            MediaTrack(p) => p.as_ptr() as *mut _,
            ReaProject(p) => p.as_ptr() as *mut _,
            MediaItem(p) => p.as_ptr() as *mut _,
            MediaItemTake(p) => p.as_ptr() as *mut _,
            TrackEnvelope(p) => p.as_ptr() as *mut _,
            PcmSource(p) => p.as_ptr() as *mut _,
            Custom { pointer, .. } => *pointer,
        }
    }
}

/// For just having to pass a NonNull pointer to `validate_ptr_2`. Very convenient!
macro_rules! impl_from_ptr_to_variant {
    ($struct_type: ty, $enum_name: ident) => {
        impl<'a> From<$struct_type> for ReaperPointer<'a> {
            fn from(p: $struct_type) -> Self {
                ReaperPointer::$enum_name(p)
            }
        }
    };
}

impl_from_ptr_to_variant!(MediaTrack, MediaTrack);
impl_from_ptr_to_variant!(ReaProject, ReaProject);
impl_from_ptr_to_variant!(MediaItem, MediaItem);
impl_from_ptr_to_variant!(MediaItemTake, MediaItemTake);
impl_from_ptr_to_variant!(TrackEnvelope, TrackEnvelope);
impl_from_ptr_to_variant!(NonNull<raw::PCM_source>, PcmSource);
