use super::{MediaItem, MediaItemTake, MediaTrack, ReaProject, TrackEnvelope};
use crate::{concat_reaper_strs, PcmSource, ReaperStr, ReaperStringArg};

use std::borrow::Cow;
use std::os::raw::c_void;

/// Validatable REAPER pointer.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ReaperPointer<'a> {
    MediaTrack(MediaTrack),
    ReaProject(ReaProject),
    MediaItem(MediaItem),
    MediaItemTake(MediaItemTake),
    TrackEnvelope(TrackEnvelope),
    PcmSource(PcmSource),
    /// If a variant is missing in this enum, you can use this custom one as a resort.
    ///
    /// Use [`custom()`] to create this variant.
    ///
    /// [`custom()`]: #method.custom
    Custom {
        type_name: Cow<'a, ReaperStr>,
        pointer: *mut c_void,
    },
}

impl<'a> ReaperPointer<'a> {
    /// Convenience function for creating a [`Custom`] pointer.
    ///
    /// **Don't** include the trailing asterisk (`*`)! It will be added automatically.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(
        pointer: *mut c_void,
        type_name: impl Into<ReaperStringArg<'a>>,
    ) -> ReaperPointer<'a> {
        ReaperPointer::Custom {
            pointer,
            type_name: type_name.into().into_inner(),
        }
    }

    pub(crate) fn key_into_raw(self) -> Cow<'a, ReaperStr> {
        use ReaperPointer::*;
        match self {
            MediaTrack(_) => reaper_str!("MediaTrack*").into(),
            ReaProject(_) => reaper_str!("ReaProject*").into(),
            MediaItem(_) => reaper_str!("MediaItem*").into(),
            MediaItemTake(_) => reaper_str!("MediaItem_Take*").into(),
            TrackEnvelope(_) => reaper_str!("TrackEnvelope*").into(),
            PcmSource(_) => reaper_str!("PCM_source*").into(),
            Custom {
                pointer: _,
                type_name,
            } => concat_reaper_strs(type_name.as_ref(), reaper_str!("*")).into(),
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
            PcmSource(p) => p.to_raw() as *mut _,
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
impl_from_ptr_to_variant!(PcmSource, PcmSource);
