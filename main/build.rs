use cc;

fn main() {
    #[cfg(not(windows))]
        generate_bindings();
    compile_glue();
}

fn compile_glue() {
    cc::Build::new()
        .cpp(true)
        .file("src/low_level/control_surface.cpp")
        .compile("glue");
}

fn generate_bindings() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/low_level/bindgen.hpp");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/low_level/bindgen.hpp")
        .opaque_type("timex")
        .derive_eq(true)
        .derive_partialeq(true)
        .derive_hash(true)
        .clang_arg("-xc++")
        .enable_cxx_namespaces()
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .whitelist_var("EnumProjects")
        .whitelist_var("GetTrack")
        .whitelist_var("ValidatePtr2")
        .whitelist_var("GetSetMediaTrackInfo")
        .whitelist_var("ShowConsoleMsg")
        .whitelist_var("REAPER_PLUGIN_VERSION")
        .whitelist_var("plugin_register")
        .whitelist_var("SectionFromUniqueID")
        .whitelist_var("NamedCommandLookup")
        .whitelist_var("KBD_OnMainActionEx")
        .whitelist_var("GetMainHwnd")
        .whitelist_var("ClearConsole")
        .whitelist_var("CountTracks")
        .whitelist_var("InsertTrackAtIndex")
        .whitelist_var("TrackList_UpdateAllExternalSurfaces")
        .whitelist_var("GetMediaTrackInfo_Value")
        .whitelist_var("GetAppVersion")
        .whitelist_var("GetTrackEnvelopeByName")
        .whitelist_var("GetTrackAutomationMode")
        .whitelist_var("GetGlobalAutomationOverride")
        .whitelist_var("TrackFX_GetRecCount")
        .whitelist_var("TrackFX_GetCount")
        .whitelist_var("TrackFX_GetFXGUID")
        .whitelist_var("TrackFX_GetParamNormalized")
        .whitelist_var("GetMasterTrack")
        .whitelist_var("guidToString")
        .whitelist_var("stringToGuid")
        .whitelist_var("CSurf_OnInputMonitorChangeEx")
        .whitelist_var("SetMediaTrackInfo_Value")
        .whitelist_var("DB2SLIDER")
        .whitelist_var("SLIDER2DB")
        .whitelist_var("GetTrackUIVolPan")
        .whitelist_var("CSurf_OnVolumeChangeEx")
        .whitelist_var("CSurf_SetSurfaceVolume")
        .whitelist_var("CSurf_OnPanChangeEx")
        .whitelist_var("CSurf_SetSurfacePan")
        .whitelist_var("CountSelectedTracks2")
        .whitelist_var("SetTrackSelected")
        .whitelist_var("GetSelectedTrack2")
        .whitelist_var("SetOnlyTrackSelected")
        .whitelist_var("GetTrackStateChunk")
        .whitelist_var("CSurf_OnRecArmChangeEx")
        .whitelist_var("SetTrackStateChunk")
        .whitelist_var("DeleteTrack")
        .whitelist_var("GetTrackNumSends")
        .whitelist_var("GetSetTrackSendInfo")
        .whitelist_var("CreateTrackSend")
        .whitelist_var("CSurf_OnSendVolumeChange")
        .whitelist_var("CSurf_OnSendPanChange")
        .whitelist_var("GetTrackSendUIVolPan")
        .whitelist_var("kbd_getTextFromCmd")
        .whitelist_var("GetToggleCommandState2")
        .whitelist_var("ReverseNamedCommandLookup")
        .whitelist_var("Main_OnCommandEx")
        .whitelist_var("CSurf_SetSurfaceMute")
        .whitelist_var("CSurf_SetSurfaceSolo")
        .whitelist_var("CSURF_EXT_.*")
        .whitelist_type("HINSTANCE")
        .whitelist_type("reaper_plugin_info_t")
        .whitelist_type("gaccel_register_t")
        .whitelist_type("KbdSectionInfo")
        .whitelist_type("GUID")
        .whitelist_function("reaper_rs_control_surface::.*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed. TODO Do as soon as available
//        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the bindings.rs file.
    let out_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("src/low_level/bindings.rs"))
        .expect("Couldn't write bindings!");
}