use cc;

fn main() {
    #[cfg(feature = "generate-from-headers")]
    codegen::generate_all();
    compile_glue();
}

fn compile_glue() {
    cc::Build::new()
        .cpp(true)
        .file("src/low_level/control_surface.cpp")
        .file("src/low_level/midi.cpp")
        .compile("glue");
}

#[cfg(feature = "generate-from-headers")]
mod codegen {
    /// Generates both `bindings.rs` and `reaper.rs`
    pub fn generate_all() {
        generate_bindings();
        generate_reaper();
    }

    /// Generates the low-level `bindings.rs` file from REAPER C++ headers
    fn generate_bindings() {
        println!("cargo:rerun-if-changed=src/low_level/bindgen.hpp");
        let bindings = bindgen::Builder::default()
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
            .raw_line("#![allow(dead_code)]")
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
            .whitelist_var("GetMaxMidiInputs")
            .whitelist_var("GetMidiInput")
            .whitelist_var("GetMidiOutput")
            .whitelist_var("GetMIDIInputName")
            .whitelist_var("GetMaxMidiOutputs")
            .whitelist_var("GetMIDIOutputName")
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
            .whitelist_var("genGuid")
            .whitelist_var("GetCurrentProjectInLoadSave")
            .whitelist_var("Undo_BeginBlock2")
            .whitelist_var("Undo_EndBlock2")
            .whitelist_var("Undo_CanUndo2")
            .whitelist_var("Undo_CanRedo2")
            .whitelist_var("Undo_DoUndo2")
            .whitelist_var("Undo_DoRedo2")
            .whitelist_var("MarkProjectDirty")
            .whitelist_var("IsProjectDirty")
            .whitelist_var("Master_GetTempo")
            .whitelist_var("SetCurrentBPM")
            .whitelist_var("Master_GetPlayRate")
            .whitelist_var("CSurf_OnPlayRateChange")
            .whitelist_var("ShowMessageBox")
            .whitelist_var("GetMainHwnd")
            .whitelist_var("StuffMIDIMessage")
            .whitelist_var("Audio_RegHardwareHook")
            .whitelist_var("TrackFX_AddByName")
            .whitelist_var("TrackFX_GetEnabled")
            .whitelist_var("TrackFX_SetEnabled")
            .whitelist_var("TrackFX_GetNumParams")
            .whitelist_var("TrackFX_GetFXName")
            .whitelist_var("TrackFX_GetInstrument")
            .whitelist_var("TrackFX_GetParamName")
            .whitelist_var("TrackFX_GetFormattedParamValue")
            .whitelist_var("TrackFX_FormatParamValueNormalized")
            .whitelist_var("TrackFX_SetParamNormalized")
            .whitelist_var("TrackFX_GetParameterStepSizes")
            .whitelist_var("TrackFX_GetParamEx")
            .whitelist_var("TrackFX_GetPreset")
            .whitelist_var("TrackFX_GetPresetIndex")
            .whitelist_var("TrackFX_SetPresetByIndex")
            .whitelist_var("TrackFX_NavigatePresets")
            .whitelist_var("GetLastTouchedFX")
            .whitelist_var("TrackFX_CopyToTrack")
            .whitelist_var("TrackFX_Delete")
            .whitelist_var("TrackFX_GetFloatingWindow")
            .whitelist_var("TrackFX_Show")
            .whitelist_var("TrackFX_GetOpen")
            .whitelist_var("GetFocusedFX")
            .whitelist_var("CSURF_EXT_.*")
            .whitelist_type("HINSTANCE")
            .whitelist_type("reaper_plugin_info_t")
            .whitelist_type("gaccel_register_t")
            .whitelist_type("audio_hook_register_t")
            .whitelist_type("KbdSectionInfo")
            .whitelist_type("GUID")
            .whitelist_function("GetActiveWindow")
            .whitelist_function("reaper_rs_control_surface::.*")
            .whitelist_function("reaper_rs_midi::.*")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("Unable to generate bindings");
        // Write the bindings to the bindings.rs file.
        let out_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("src/low_level/bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    /// Generates the low-level `reaper.rs` file from the previously generated `bindings.rs`
    fn generate_reaper() {
        use quote::ToTokens;
        use std::path::Path;
        use syn::{
            AngleBracketedGenericArguments, ForeignItem, ForeignItemStatic, GenericArgument, Ident,
            Item, ItemForeignMod, ItemMod, PathArguments, PathSegment, Type, TypeBareFn,
        };

        generate();

        fn generate() {
            use std::env;
            use std::fs::File;
            use std::io::Read;
            use std::process;

            let mut file =
                File::open("src/low_level/bindings.rs").expect("Unable to open bindings.rs");

            let mut src = String::new();
            file.read_to_string(&mut src).expect("Unable to read file");
            let file = syn::parse_file(&src).expect("Unable to parse file");
            let fn_ptrs: Vec<_> = filter_reaper_fn_ptr_items(&file)
                .into_iter()
                .map(map_to_reaper_fn_ptr)
                .collect();
            let idents: Vec<_> = fn_ptrs.iter().map(|p| p.ident.clone()).collect();
            let fn_types: Vec<TypeBareFn> = fn_ptrs.iter().map(|p| p.fn_type.clone()).collect();
            let result = quote::quote! {
                use super::bindings::root;

               #[derive(Default)]
                pub struct Reaper {
                    #(
                        pub #idents: Option<#fn_types>,
                    )*
                }
            };
            std::fs::write("src/low_level/reaper.rs", result.to_string())
                .expect("Unable to write file");
        }

        fn filter_reaper_fn_ptr_items(file: &syn::File) -> Vec<&ForeignItemStatic> {
            let (_, root_mod_items) = match file.items.as_slice() {
                [Item::Mod(ItemMod {
                    ident: id,
                    content: Some(c),
                    ..
                })] if id == "root" => c,
                _ => panic!("root mod not found"),
            };
            root_mod_items
                .iter()
                .filter_map(|item| match item {
                    Item::ForeignMod(ItemForeignMod { items, .. }) => match items.as_slice() {
                        [ForeignItem::Static(i)] => Some(i),
                        _ => None,
                    },
                    _ => None,
                })
                .collect()
        }

        fn map_to_reaper_fn_ptr(item: &ForeignItemStatic) -> ReaperFnPtr {
            let option_segment = match &*item.ty {
                Type::Path(p) => p
                    .path
                    .segments
                    .iter()
                    .find(|seg| seg.ident == "Option")
                    .expect("Option not found in fn ptr item"),
                _ => panic!("fn ptr item doesn't have path type"),
            };
            let bare_fn = match &option_segment.arguments {
                PathArguments::AngleBracketed(a) => {
                    let generic_arg = a.args.first().expect("Angle bracket must have arg");
                    match generic_arg {
                        GenericArgument::Type(Type::BareFn(bare_fn)) => bare_fn,
                        _ => panic!("Generic argument is not a BareFn"),
                    }
                }
                _ => panic!("Option type doesn't have angle bracket"),
            };
            ReaperFnPtr {
                ident: item.ident.clone(),
                fn_type: TypeBareFn {
                    abi: None,
                    unsafety: None,
                    ..bare_fn.clone()
                },
            }
        }

        struct ReaperFnPtr {
            ident: Ident,
            fn_type: TypeBareFn,
        }
    }
}
