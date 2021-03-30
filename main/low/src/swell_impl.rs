#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
use crate::{
    bindings::root, register_plugin_destroy_hook, PluginContext, Swell, SwellFunctionPointers,
};

// This is safe (see https://doc.rust-lang.org/std/sync/struct.Once.html#examples-1).
static mut INSTANCE: Option<Swell> = None;

/// This impl block contains functions which exist in SWELL as macros and therefore are not picked
/// up by `bindgen`.
impl Swell {
    /// Makes the given instance available globally.
    ///
    /// After this has been called, the instance can be queried globally using `get()`.
    ///
    /// This can be called once only. Subsequent calls won't have any effect!
    pub fn make_available_globally(functions: Swell) {
        static INIT_INSTANCE: std::sync::Once = std::sync::Once::new();
        unsafe {
            INIT_INSTANCE.call_once(|| {
                INSTANCE = Some(functions);
                register_plugin_destroy_hook(|| INSTANCE = None);
            });
        }
    }

    /// Gives access to the instance which you made available globally before.
    ///
    /// # Panics
    ///
    /// This panics if [`make_available_globally()`] has not been called before.
    ///
    /// [`make_available_globally()`]: fn.make_available_globally.html
    pub fn get() -> &'static Swell {
        unsafe {
            INSTANCE
                .as_ref()
                .expect("call `make_available_globally()` before using `get()`")
        }
    }

    /// Gives access to the SWELL function pointers.
    pub fn pointers(&self) -> &SwellFunctionPointers {
        &self.pointers
    }

    /// Returns the plug-in context.
    pub fn plugin_context(&self) -> &PluginContext {
        self.plugin_context
            .as_ref()
            .expect("plug-in context not available on demo instances")
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn CreateDialogParam(
        &self,
        hinst: root::HINSTANCE,
        resid: *const ::std::os::raw::c_char,
        par: root::HWND,
        dlgproc: root::DLGPROC,
        param: root::LPARAM,
    ) -> root::HWND {
        #[cfg(target_family = "unix")]
        {
            self.SWELL_CreateDialog(
                root::SWELL_curmodule_dialogresource_head,
                resid,
                par,
                dlgproc,
                param,
            )
        }
        #[cfg(target_family = "windows")]
        #[allow(clippy::cast_ptr_alignment)]
        {
            // TODO-low winapi-rs is expecting the dlgproc function pointer to be `extern "system"`.
            //  What we have is `extern "C"`. This caught cause issues on Windows i686 (32-bit)
            //  builds. However, in practice it didn't show any issues (tested with ReaLearn). So
            //  probably not that  important.
            winapi::um::winuser::CreateDialogParamW(
                hinst as _,
                resid as _,
                par as _,
                std::mem::transmute(dlgproc),
                param,
            ) as _
        }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn LoadMenu(
        &self,
        hinst: root::HINSTANCE,
        resid: *const ::std::os::raw::c_char,
    ) -> root::HMENU {
        #[cfg(target_family = "unix")]
        {
            self.SWELL_LoadMenu(root::SWELL_curmodule_menuresource_head, resid)
        }
        #[cfg(target_family = "windows")]
        #[allow(clippy::cast_ptr_alignment)]
        {
            winapi::um::winuser::LoadMenuW(hinst as _, resid as _) as _
        }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn FillRect(&self, ctx: root::HDC, r: *const root::RECT, br: root::HBRUSH) {
        #[cfg(target_family = "unix")]
        {
            self.SWELL_FillRect(ctx, r, br);
        }
        #[cfg(target_family = "windows")]
        #[allow(clippy::cast_ptr_alignment)]
        {
            winapi::um::winuser::FillRect(ctx as _, r as _, br as _);
        }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn DrawText(
        &self,
        ctx: root::HDC,
        buf: *const ::std::os::raw::c_char,
        len: ::std::os::raw::c_int,
        r: *mut root::RECT,
        align: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        #[cfg(target_family = "unix")]
        {
            self.SWELL_DrawText(ctx, buf, len, r, align)
        }
        #[cfg(target_family = "windows")]
        #[allow(clippy::cast_ptr_alignment)]
        {
            let utf16_string = utf8_to_16(buf);
            let result = winapi::um::winuser::DrawTextW(
                ctx as _,
                utf16_string.as_ptr(),
                len,
                r as _,
                align as _,
            );
            std::mem::drop(utf16_string);
            result
        }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn SetWindowText(
        &self,
        hwnd: root::HWND,
        text: *const ::std::os::raw::c_char,
    ) -> root::BOOL {
        #[cfg(target_family = "unix")]
        {
            self.SetDlgItemText(hwnd, 0, text)
        }
        #[cfg(target_family = "windows")]
        {
            let utf16_string = utf8_to_16(text);
            let result = winapi::um::winuser::SetWindowTextW(hwnd as _, utf16_string.as_ptr());
            std::mem::drop(utf16_string);
            result as _
        }
    }

    /// Attention: Whereas the Windows original returns a length, this just returns success.
    ///
    /// In order to avoid surprises, on Windows it will behave like this, too.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn GetWindowText(
        &self,
        hwnd: root::HWND,
        lpString: root::LPSTR,
        nMaxCount: std::os::raw::c_int,
    ) -> root::BOOL {
        #[cfg(target_family = "unix")]
        {
            self.GetDlgItemText(hwnd, 0, lpString, nMaxCount)
        }
        #[cfg(target_family = "windows")]
        {
            let len = with_utf16_to_8(lpString, nMaxCount, |buffer, max_size| {
                winapi::um::winuser::GetWindowTextW(hwnd as _, buffer, max_size) as _
            });
            // Just return whether successful in order to conform to SWELL.
            if len == 0 { 0 } else { 1 }
        }
    }

    pub fn RGB(r: u8, g: u8, b: u8) -> root::DWORD {
        #[cfg(target_family = "unix")]
        {
            // SWELL says: "the byte ordering of RGB() etc is different than on win32"
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        }
        #[cfg(target_family = "windows")]
        {
            (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
        }
    }

    pub fn GetRValue(color: root::DWORD) -> u8 {
        #[cfg(target_family = "unix")]
        {
            ((color >> 16) & 0xff) as _
        }
        #[cfg(target_family = "windows")]
        {
            (color & 0xff) as _
        }
    }

    pub fn GetGValue(color: root::DWORD) -> u8 {
        ((color >> 8) & 0xff) as _
    }

    pub fn GetBValue(color: root::DWORD) -> u8 {
        #[cfg(target_family = "unix")]
        {
            (color & 0xff) as _
        }
        #[cfg(target_family = "windows")]
        {
            ((color >> 16) & 0xff) as _
        }
    }
}

/// This impl block contains functions which delegate to native win32 functions but need some
/// character encoding conversion.
///
/// SWELL uses UTF-8-encoded strings as byte arrays (`*const i8`), exactly like REAPER itself.
/// Windows uses UTF-16-encoded strings as u16 arrays (`*const u16`). It's very convenient that we
/// can use UTF-8 throughout: Rust, REAPER, SWELL ... just Windows was missing.
#[cfg(target_family = "windows")]
impl Swell {
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn SendMessage(
        &self,
        hwnd: root::HWND,
        msg: root::UINT,
        wParam: root::WPARAM,
        lParam: root::LPARAM,
    ) -> root::LRESULT {
        if lParam != 0 && lparam_is_string(msg) {
            let utf16_string = utf8_to_16(lParam as _);
            let result = winapi::um::winuser::SendMessageW(
                hwnd as _,
                msg,
                wParam,
                utf16_string.as_ptr() as _,
            );
            std::mem::drop(utf16_string);
            result
        } else {
            winapi::um::winuser::SendMessageW(hwnd as _, msg, wParam, lParam)
        }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn PostMessage(
        &self,
        hwnd: root::HWND,
        msg: root::UINT,
        wParam: root::WPARAM,
        lParam: root::LPARAM,
    ) -> root::BOOL {
        if lParam != 0 && lparam_is_string(msg) {
            let utf16_string = utf8_to_16(lParam as _);
            let result = winapi::um::winuser::PostMessageW(
                hwnd as _,
                msg,
                wParam,
                utf16_string.as_ptr() as _,
            );
            std::mem::drop(utf16_string);
            result as _
        } else {
            winapi::um::winuser::PostMessageW(hwnd as _, msg, wParam, lParam) as _
        }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn MessageBox(
        &self,
        hwndParent: root::HWND,
        text: *const ::std::os::raw::c_char,
        caption: *const ::std::os::raw::c_char,
        type_: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
        let text_utf16 = utf8_to_16(text);
        let caption_utf16 = utf8_to_16(caption);
        let result = winapi::um::winuser::MessageBoxW(
            hwndParent as _,
            text_utf16.as_ptr() as _,
            caption_utf16.as_ptr() as _,
            type_ as _,
        );
        std::mem::drop(text_utf16);
        result as _
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn SetMenuItemInfo(
        &self,
        hMenu: root::HMENU,
        pos: ::std::os::raw::c_int,
        byPos: root::BOOL,
        mi: *mut root::MENUITEMINFO,
    ) -> root::BOOL {
        let mi = *mi;
        let mut utf16_mi = utf8_to_16_menu_item_info(&mi);
        if menu_item_needs_string_conversion(mi) {
            // Sets text. Must convert it.
            let mut utf16_string = utf8_to_16(mi.dwTypeData);
            utf16_mi.dwTypeData = utf16_string.as_mut_ptr();
            let result = winapi::um::winuser::SetMenuItemInfoW(
                hMenu as _,
                pos as _,
                byPos as _,
                &utf16_mi as *const _,
            );
            std::mem::drop(utf16_string);
            result as _
        } else {
            // Doesn't set text. No conversion necessary.
            let result = winapi::um::winuser::SetMenuItemInfoW(
                hMenu as _,
                pos as _,
                byPos as _,
                &utf16_mi as *const _,
            );
            result as _
        }
    }

    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn InsertMenuItem(
        &self,
        hMenu: root::HMENU,
        pos: ::std::os::raw::c_int,
        byPos: root::BOOL,
        mi: *mut root::MENUITEMINFO,
    ) {
        let mi = *mi;
        let mut utf16_mi = utf8_to_16_menu_item_info(&mi);
        if menu_item_needs_string_conversion(mi) {
            // Sets text. Must convert it.
            let mut utf16_string = utf8_to_16(mi.dwTypeData);
            utf16_mi.dwTypeData = utf16_string.as_mut_ptr();
            let result = winapi::um::winuser::InsertMenuItemW(
                hMenu as _,
                pos as _,
                byPos as _,
                &utf16_mi as *const _,
            );
            std::mem::drop(utf16_string);
        } else {
            // Doesn't set text. No conversion necessary.
            let result = winapi::um::winuser::InsertMenuItemW(
                hMenu as _,
                pos as _,
                byPos as _,
                &utf16_mi as *const _,
            );
        }
    }

    /// **Attention:** This doesn't yet support `dwTypeData` (always `null` currently).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    pub unsafe fn GetMenuItemInfo(
        &self,
        hMenu: root::HMENU,
        pos: ::std::os::raw::c_int,
        byPos: root::BOOL,
        mi: *mut root::MENUITEMINFO,
    ) -> root::BOOL {
        let mut mi = *mi;
        if !mi.dwTypeData.is_null() {
            todo!("Getting string information from menu item is not yet implemented.")
        }
        let mut utf16_mi = utf8_to_16_menu_item_info(&mi);
        let result = winapi::um::winuser::GetMenuItemInfoW(
            hMenu as _,
            pos as _,
            byPos as _,
            &mut utf16_mi as _,
        );
        mi.cbSize = utf16_mi.cbSize;
        mi.fMask = utf16_mi.fMask;
        mi.fType = utf16_mi.fType;
        mi.fState = utf16_mi.fState;
        mi.wID = utf16_mi.wID;
        mi.hSubMenu = utf16_mi.hSubMenu as _;
        mi.hbmpChecked = utf16_mi.hbmpChecked as _;
        mi.hbmpUnchecked = utf16_mi.hbmpUnchecked as _;
        mi.dwItemData = utf16_mi.dwItemData;
        mi.dwTypeData = std::ptr::null_mut();
        mi.cch = utf16_mi.cch as _;
        mi.hbmpItem = utf16_mi.hbmpItem as _;
        result as _
    }

    /// On Windows this is a constant but in SWELL this is a macro which translates to a function
    /// call.
    pub fn CF_TEXT(&self) -> root::UINT {
        #[cfg(target_family = "unix")]
        {
            unsafe { self.RegisterClipboardFormat(c_str_macro::c_str!("SWELL__CF_TEXT").as_ptr()) }
        }
        #[cfg(target_family = "windows")]
        1
    }
}

impl std::fmt::Debug for SwellFunctionPointers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwellFunctionPointers")
            .field("loaded_count", &self.loaded_count)
            .field("total_count", &Self::TOTAL_COUNT)
            .finish()
    }
}

/// Converts the given UTF-8 C-style string (nul terminator) to an UTF-16 C-style string.
///
/// # Safety
///
/// You must ensure that the given string points to an UTF-8 encoded C-style string.
#[cfg(target_family = "windows")]
pub(crate) unsafe fn utf8_to_16(raw_utf8: *const std::os::raw::c_char) -> Vec<u16> {
    use std::ffi::{CStr, OsStr};
    use std::iter::once;
    // Assumes that the given pointer points to a C-style string.
    let utf8_c_str = CStr::from_ptr(raw_utf8);
    // Interpret that string as UTF-8-encoded. Fall back to replacement characters if not.
    let str = utf8_c_str.to_string_lossy();
    // Now reencode it as UTF-16.
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(str.as_ref())
        .encode_wide()
        .chain(once(0))
        .collect()
}

/// Creates a UTF-16 buffer (to be filled by the given function) and writes it as UTF-8 to the given
/// target buffer.
///
/// `max_size` must include nul terminator. The given function must return the actual string length
/// *without* nul terminator.
#[cfg(target_family = "windows")]
pub(crate) unsafe fn with_utf16_to_8(
    utf8_target_buffer: *mut std::os::raw::c_char,
    requested_max_size: std::os::raw::c_int,
    fill_utf16_source_buffer: impl FnOnce(*mut u16, std::os::raw::c_int) -> usize,
) -> usize {
    let mut utf16_vec: Vec<u16> = Vec::with_capacity(requested_max_size as usize);
    // Returns length *without* nul terminator.
    let len = fill_utf16_source_buffer(utf16_vec.as_mut_ptr(), requested_max_size);
    if len == 0 {
        return 0;
    }
    utf16_vec.set_len(len);
    // nul terminator will not be part of the string because len doesn't include it!
    let string = String::from_utf16_lossy(&utf16_vec);
    let c_string = match std::ffi::CString::new(string) {
        Ok(s) => s,
        Err(_) => {
            // String contained 0 byte. This would end a C-style string abruptly.
            return 0;
        }
    };
    let source_bytes = c_string.as_bytes_with_nul();
    let target_bytes =
        std::slice::from_raw_parts_mut(utf8_target_buffer, requested_max_size as usize);
    let source_bytes_signed = &*(source_bytes as *const [u8] as *const [i8]);
    target_bytes[..source_bytes.len()].copy_from_slice(source_bytes_signed);
    len
}

/// For all messages which contain a string payload, convert the string's encoding.
#[cfg(target_family = "windows")]
fn lparam_is_string(msg: root::UINT) -> bool {
    use crate::raw;
    // There are probably more than just those two. Add as soon as needed.
    matches!(msg, raw::CB_INSERTSTRING | raw::CB_ADDSTRING)
}

/// cbSize doesn't matter.
/// Converts everything except `dwTypeData` (needs special treatment).
#[cfg(target_family = "windows")]
fn utf8_to_16_menu_item_info(mi: &root::MENUITEMINFO) -> winapi::um::winuser::MENUITEMINFOW {
    winapi::um::winuser::MENUITEMINFOW {
        cbSize: std::mem::size_of::<winapi::um::winuser::MENUITEMINFOW>() as _,
        fMask: mi.fMask,
        fType: mi.fType,
        fState: mi.fState,
        wID: mi.wID,
        hSubMenu: mi.hSubMenu as _,
        hbmpChecked: mi.hbmpChecked as _,
        hbmpUnchecked: mi.hbmpUnchecked as _,
        dwItemData: mi.dwItemData,
        dwTypeData: std::ptr::null_mut(),
        cch: mi.cch as _,
        hbmpItem: mi.hbmpItem as _,
    }
}

#[cfg(target_family = "windows")]
fn menu_item_needs_string_conversion(mi: root::MENUITEMINFO) -> bool {
    // Super important to use `raw` constants here because the SWELL constant values deviate
    // from the Windows constants!!!
    use crate::raw;
    (mi.fMask & raw::MIIM_TYPE) != 0 && (mi.fMask & raw::MIIM_DATA) != 0
}
