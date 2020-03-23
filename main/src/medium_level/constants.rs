#![allow(non_camel_case_types)]
use c_str_macro::c_str;
use std::borrow::Cow;
use std::ffi::{CStr, CString};

// TODO-low Maybe don't use ENV suffix or even use PascalCase
// TODO-low Add more values
// TODO-low Rename
pub enum EnvChunkName {
    VOLENV,
    PANENV,
    Custom(&'static CStr),
}

impl From<EnvChunkName> for Cow<'static, CStr> {
    fn from(value: EnvChunkName) -> Self {
        use EnvChunkName::*;
        match value {
            VOLENV => c_str!("VOLENV").into(),
            PANENV => c_str!("PANENV").into(),
            Custom(name) => name.into(),
        }
    }
}

// TODO-low Rename
// TODO-low Maybe use PascalCase
pub enum MediaTrackInfoKey {
    B_FREEMODE,
    B_HEIGHTLOCK,
    B_MAINSEND,
    B_MUTE,
    B_PHASE,
    B_SHOWINMIXER,
    B_SHOWINTCP,
    C_BEATATTACHMODE,
    C_MAINSEND_OFFS,
    D_DUALPANL,
    D_DUALPANR,
    D_PAN,
    D_PANLAW,
    D_PLAY_OFFSET,
    D_VOL,
    D_WIDTH,
    F_MCP_FXSEND_SCALE,
    F_MCP_SENDRGN_SCALE,
    GUID,
    I_AUTOMODE,
    I_CUSTOMCOLOR,
    I_FOLDERCOMPACT,
    I_FOLDERDEPTH,
    I_FXEN,
    I_HEIGHTOVERRIDE,
    I_MCPH,
    I_MCPW,
    I_MCPX,
    I_MCPY,
    I_MIDIHWOUT,
    I_NCHAN,
    I_PANMODE,
    I_PERFFLAGS,
    I_PLAY_OFFSET_FLAG,
    I_RECARM,
    I_RECINPUT,
    I_RECMODE,
    I_RECMON,
    I_RECMONITEMS,
    I_SELECTED,
    I_SOLO,
    I_TCPH,
    I_TCPY,
    I_WNDH,
    IP_TRACKNUMBER,
    P_ENV(EnvChunkName),
    P_EXT(&'static CStr),
    P_ICON,
    P_MCP_LAYOUT,
    P_NAME,
    P_PARTRACK,
    P_PROJECT,
    P_TCP_LAYOUT,
    Custom(&'static CStr),
}

impl From<MediaTrackInfoKey> for Cow<'static, CStr> {
    fn from(value: MediaTrackInfoKey) -> Self {
        use MediaTrackInfoKey::*;
        match value {
            B_FREEMODE => c_str!("B_FREEMODE").into(),
            B_HEIGHTLOCK => c_str!("B_HEIGHTLOCK").into(),
            B_MAINSEND => c_str!("B_MAINSEND").into(),
            B_MUTE => c_str!("B_MUTE").into(),
            B_PHASE => c_str!("B_PHASE").into(),
            B_SHOWINMIXER => c_str!("B_SHOWINMIXER").into(),
            B_SHOWINTCP => c_str!("B_SHOWINTCP").into(),
            C_BEATATTACHMODE => c_str!("C_BEATATTACHMODE").into(),
            C_MAINSEND_OFFS => c_str!("C_MAINSEND_OFFS").into(),
            D_DUALPANL => c_str!("D_DUALPANL").into(),
            D_DUALPANR => c_str!("D_DUALPANR").into(),
            D_PAN => c_str!("D_PAN").into(),
            D_PANLAW => c_str!("D_PANLAW").into(),
            D_PLAY_OFFSET => c_str!("D_PLAY_OFFSET").into(),
            D_VOL => c_str!("D_VOL").into(),
            D_WIDTH => c_str!("D_WIDTH").into(),
            F_MCP_FXSEND_SCALE => c_str!("F_MCP_FXSEND_SCALE").into(),
            F_MCP_SENDRGN_SCALE => c_str!("F_MCP_SENDRGN_SCALE").into(),
            GUID => c_str!("GUID").into(),
            I_AUTOMODE => c_str!("I_AUTOMODE").into(),
            I_CUSTOMCOLOR => c_str!("I_CUSTOMCOLOR").into(),
            I_FOLDERCOMPACT => c_str!("I_FOLDERCOMPACT").into(),
            I_FOLDERDEPTH => c_str!("I_FOLDERDEPTH").into(),
            I_FXEN => c_str!("I_FXEN").into(),
            I_HEIGHTOVERRIDE => c_str!("I_HEIGHTOVERRIDE").into(),
            I_MCPH => c_str!("I_MCPH").into(),
            I_MCPW => c_str!("I_MCPW").into(),
            I_MCPX => c_str!("I_MCPX").into(),
            I_MCPY => c_str!("I_MCPY").into(),
            I_MIDIHWOUT => c_str!("I_MIDIHWOUT").into(),
            I_NCHAN => c_str!("I_NCHAN").into(),
            I_PANMODE => c_str!("I_PANMODE").into(),
            I_PERFFLAGS => c_str!("I_PERFFLAGS").into(),
            I_PLAY_OFFSET_FLAG => c_str!("I_PLAY_OFFSET_FLAG").into(),
            I_RECARM => c_str!("I_RECARM").into(),
            I_RECINPUT => c_str!("I_RECINPUT").into(),
            I_RECMODE => c_str!("I_RECMODE").into(),
            I_RECMON => c_str!("I_RECMON").into(),
            I_RECMONITEMS => c_str!("I_RECMONITEMS").into(),
            I_SELECTED => c_str!("I_SELECTED").into(),
            I_SOLO => c_str!("I_SOLO").into(),
            I_TCPH => c_str!("I_TCPH").into(),
            I_TCPY => c_str!("I_TCPY").into(),
            I_WNDH => c_str!("I_WNDH").into(),
            IP_TRACKNUMBER => c_str!("IP_TRACKNUMBER").into(),
            P_ENV(env_chunk_name) => {
                let cow: Cow<CStr> = env_chunk_name.into();
                concat_c_strs(c_str!("P_ENV:<"), cow.as_ref()).into()
            }
            P_EXT(extension_specific_key) => {
                concat_c_strs(c_str!("P_EXT:"), extension_specific_key).into()
            }
            P_ICON => c_str!("P_ICON").into(),
            P_MCP_LAYOUT => c_str!("P_MCP_LAYOUT").into(),
            P_NAME => c_str!("P_NAME").into(),
            P_PARTRACK => c_str!("P_PARTRACK").into(),
            P_PROJECT => c_str!("P_PROJECT").into(),
            P_TCP_LAYOUT => c_str!("P_TCP_LAYOUT").into(),
            Custom(key) => key.into(),
        }
    }
}

fn concat_c_strs(first: &CStr, second: &CStr) -> CString {
    CString::new([first.to_bytes(), second.to_bytes()].concat()).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialize() {
        use MediaTrackInfoKey::*;
        assert_eq!(Cow::from(B_MUTE).as_ref(), c_str!("B_MUTE"));
        assert_eq!(
            Cow::from(P_ENV(EnvChunkName::VOLENV)).as_ref(),
            c_str!("P_ENV:<VOLENV")
        );
        assert_eq!(
            Cow::from(P_ENV(EnvChunkName::Custom(c_str!("MYENV")))).as_ref(),
            c_str!("P_ENV:<MYENV")
        );
        assert_eq!(
            Cow::from(P_EXT(c_str!("SWS_FOO"))).as_ref(),
            c_str!("P_EXT:SWS_FOO")
        );
        assert_eq!(Cow::from(Custom(c_str!("BLA"))).as_ref(), c_str!("BLA"));
    }
}
