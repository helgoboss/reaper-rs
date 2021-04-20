/// Executed whenever Cargo builds reaper-rs
fn main() {
    #[cfg(target_family = "unix")]
    #[cfg(feature = "generate-stage-one")]
    codegen::stage_one::generate_bindings();

    #[cfg(feature = "generate-stage-two")]
    codegen::stage_two::generate_reaper_and_swell();

    #[cfg(target_family = "unix")]
    compile_swell_dialog_generator_support();

    compile_glue_code();
}

/// This makes SWELL dialogs via "swell-dlggen.h" possible (on C++ side only, via cc crate).
/// See the C++ source file for a detailled explanation.
#[cfg(target_family = "unix")]
fn compile_swell_dialog_generator_support() {
    let modstub_file = if cfg!(target_os = "macos") {
        "src/swell-modstub-custom.mm"
    } else {
        "src/swell-modstub-generic-custom.cpp"
    };
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .warnings(false)
        .define("SWELL_PROVIDED_BY_APP", None)
        .file(modstub_file);
    if cfg!(target_os = "macos") {
        build.cpp_set_stdlib("c++");
    }
    build.compile("swell");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=AppKit");
}

/// Compiles C++ glue code. This is necessary to interact with those parts of the REAPER C++ API
/// that use pure virtual interface classes and therefore the C++ ABI.
fn compile_glue_code() {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .warnings(false)
        .file("src/control_surface.cpp")
        .file("src/pcm_source.cpp")
        .file("src/midi.cpp");
    if cfg!(target_os = "macos") {
        build.cpp_set_stdlib("c++");
    }
    build.compile("glue");
}

#[cfg(any(feature = "generate-stage-one", feature = "generate-stage-two"))]
mod codegen {
    #[cfg(target_family = "unix")]
    #[cfg(feature = "generate-stage-one")]
    pub mod stage_one {
        use bindgen::callbacks::{IntKind, ParseCallbacks};

        #[derive(Debug)]
        struct CustomParseCallbacks;

        impl ParseCallbacks for CustomParseCallbacks {
            fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
                // Signed because directly associated with IReaperControlSurface::Extended()
                // `call` parameter, which is signed.
                if name.starts_with("CSURF_EXT_")
                    // Signed because ShowWindow() expects signed. winapi-rs conforms.
                    || name.starts_with("SW_")
                    // Signed because reaper_plugin_info_t::caller_version is signed.
                    || name == "REAPER_PLUGIN_VERSION"
                {
                    return Some(IntKind::I32);
                }
                // The following constants were interpreted as signed integers before but I changed
                // them to unsigned:
                // - VK_: Although declared as signed in winapi-rs, I think it should be unsigned
                //   because MapVirtualKey function takes an unsigned integer and none of the VK_
                //   constants are defined as negative integer.
                // - SWP_: Unsigned because SetWindowPos() expects unsigned. winapi-rs conforms.
                //
                // The following flags stay u32 although the APIs expect i32:
                // - UNDO_STATE_* (used as bitmask, UNDO_STATE_ALL doesn't fit into i32)
                None
            }

            fn include_file(&self, filename: &str) {
                bindgen::CargoCallbacks.include_file(filename);
            }
        }

        /// Generates the `bindings.rs` file from REAPER C++ headers
        pub fn generate_bindings() {
            println!("cargo:rerun-if-changed=src/wrapper.hpp");
            let builder = bindgen::Builder::default()
                .header("src/wrapper.hpp")
                .opaque_type("timex")
                .derive_eq(true)
                .derive_partialeq(true)
                .derive_hash(true)
                .derive_default(true)
                .clang_arg("-xc++")
                .enable_cxx_namespaces()
                // If we activate layout tests, we would have to regenerate at each build because
                // tests will fail on Linux if generated on Windows and vice versa.
                .layout_tests(false)
                // Tell cargo to invalidate the built crate whenever any of the
                // included header files changed.
                .parse_callbacks(Box::new(CustomParseCallbacks))
                .raw_line("#![allow(clippy::all)]")
                .raw_line("#![allow(non_upper_case_globals)]")
                .raw_line("#![allow(non_camel_case_types)]")
                .raw_line("#![allow(non_snake_case)]")
                .raw_line("#![allow(dead_code)]")
                .module_raw_line("root", include_str!("src/manual_bindings.rs"))
                .whitelist_var("reaper_functions::.*")
                .whitelist_var("swell_functions::.*")
                .whitelist_var("SWELL_.*")
                .whitelist_var("CSURF_EXT_.*")
                .whitelist_var("PCM_SOURCE_EXT_.*")
                .whitelist_var("REAPER_PLUGIN_VERSION")
                .whitelist_var("UNDO_STATE_.*")
                .whitelist_var("VK_.*")
                .whitelist_var("BM_.*")
                .whitelist_var("BST_.*")
                .whitelist_var("SW_.*")
                .whitelist_var("SWP_.*")
                .whitelist_var("CB_.*")
                .whitelist_var("MB_.*")
                .whitelist_var("CBN_.*")
                .whitelist_var("WM_.*")
                .whitelist_var("SIF_.*")
                .whitelist_var("SB_.*")
                .whitelist_var("EN_.*")
                .whitelist_var("MIIM_.*")
                .whitelist_var("MF_.*")
                .whitelist_var("TB_.*")
                .whitelist_var("TBM_.*")
                .whitelist_var("TPM_.*")
                .whitelist_var("CF_.*")
                .whitelist_var("DT_.*")
                .whitelist_var("GMEM_.*")
                .whitelist_var("COLOR_.*")
                .whitelist_var("SRCCOPY")
                .whitelist_var("SRCCOPY_USEALPHACHAN")
                .whitelist_var("ID.*")
                .whitelist_var("DLL_PROCESS_ATTACH")
                .whitelist_var("DLL_PROCESS_DETACH")
                .whitelist_var("TRANSPARENT")
                .whitelist_var("OPAQUE")
                .whitelist_type("HINSTANCE")
                .whitelist_type("reaper_plugin_info_t")
                .whitelist_type("gaccel_register_t")
                .whitelist_type("audio_hook_register_t")
                .whitelist_type("KbdSectionInfo")
                .whitelist_type("GUID")
                .whitelist_type("LPSTR")
                .whitelist_type("SCROLLINFO")
                .whitelist_function("reaper_control_surface::.*")
                .whitelist_function("reaper_midi::.*")
                .whitelist_function("reaper_pcm_source::.*")
                .blacklist_type("preview_register_t");
            #[cfg(target_os = "macos")]
            let builder = builder.clang_arg("-stdlib=libc++");
            let bindings = builder.generate().expect("Unable to generate bindings");
            let out_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            bindings
                .write_to_file(out_path.join("src/bindings.rs"))
                .expect("Couldn't write bindings!");
        }
    }

    #[cfg(feature = "generate-stage-two")]
    pub mod stage_two {
        use proc_macro2::Span;
        use std::fs::File;
        use std::io::Read;
        use syn::punctuated::Punctuated;
        use syn::token::{And, Colon, Extern, Fn, Paren, Pub, SelfValue, Unsafe};
        use syn::{
            Abi, Block, Expr, ExprCall, ExprPath, FnArg, ForeignItem, ForeignItemFn,
            ForeignItemStatic, GenericArgument, Ident, ImplItem, ImplItemMethod, Item,
            ItemForeignMod, ItemMod, Pat, PatIdent, PatType, Path, PathArguments, PathSegment,
            Receiver, Signature, Type, TypeBareFn, VisPublic, Visibility,
        };

        /// For these functions exposed by REAPER the function pointers need to use
        /// `extern "system` instead of `extern "C"`, probably because they are forwarded
        /// directly to Win32 calls. This uses the "stdcall" calling convention when talking to
        /// REAPER 32-bit on Windows. If we don't do this: Crash.  
        static EXTERN_SYSTEM_ABI_REAPER_FUNCTIONS: phf::Set<&'static str> = phf::phf_set![
            "InitializeCoolSB",
            "UninitializeCoolSB",
            "CoolSB_SetMinThumbSize",
            "CoolSB_GetScrollInfo",
            "CoolSB_SetScrollInfo",
            "CoolSB_SetScrollPos",
            "CoolSB_SetScrollRange",
            "CoolSB_ShowScrollBar",
            "CoolSB_SetResizingThumb",
            "CoolSB_SetThemeIndex",
        ];

        /// Functions which exist both in SWELL and in the Windows API.
        ///
        /// For all the functions in this set, the generator will create `Swell` methods which are
        /// enabled on Windows only and just delegate to the Windows-native counterparts. This is
        /// convenient because then you just write your UI code using the `Swell` struct and it
        /// will work on Windows as well. It works because SWELL has been designed to imitate the
        /// Windows API exactly (well, more or less).
        ///
        /// This set doesn't include functions which have parameters where the character encoding
        /// matters. On Windows, they have either an `A` (ANSI encoding) or a `W` suffix (wide =
        /// UTF-16 encoding). Previously, we just delegated to the `A` functions
        /// here, but of course this didn't work at all with non-ANSI characters. However, simply
        /// delegating to the `W` functions was also not an option because first, the signature
        /// is different (`W` functions take strings as `*const u16` instead of `*const i8` because
        /// they are encoded as UTF-16). This is now implemented manually in `SwellImpl`.
        static WIN32_SWELL_FUNCTIONS: phf::Set<&'static str> = phf::phf_set![
            // # winuser.h
            "BeginPaint",
            "CheckDlgButton",
            "CheckMenuItem",
            "ClientToScreen",
            "CloseClipboard",
            "CreateIconIndirect",
            "CreatePopupMenu",
            "DeleteMenu",
            "DestroyMenu",
            "DestroyWindow",
            "DrawMenuBar",
            "EmptyClipboard",
            "EnableMenuItem",
            "EnableWindow",
            "EndDialog",
            "EndPaint",
            "EnumChildWindows",
            "EnumClipboardFormats",
            "EnumWindows",
            "GetAsyncKeyState",
            "GetCapture",
            "GetClientRect",
            "GetClipboardData",
            "GetCursorPos",
            "GetDC",
            "GetDlgItem",
            "GetDlgItemInt",
            "GetFocus",
            "GetForegroundWindow",
            "GetMenu",
            "GetMenuItemCount",
            "GetMenuItemID",
            "GetMessagePos",
            "GetParent",
            "GetSubMenu",
            "GetSysColor",
            "GetSystemMetrics",
            "GetWindow",
            "GetWindowDC",
            "GetWindowRect",
            "InvalidateRect",
            "IsChild",
            "IsDlgButtonChecked",
            "IsWindow",
            "IsWindowEnabled",
            "IsWindowVisible",
            "KillTimer",
            "OpenClipboard",
            "ReleaseCapture",
            "ReleaseDC",
            "ScreenToClient",
            "ScrollWindow",
            "SetCapture",
            "SetClipboardData",
            "SetDlgItemInt",
            "SetFocus",
            "SetForegroundWindow",
            "SetMenu",
            "SetParent",
            "SetTimer",
            "SetWindowPos",
            "ShowWindow",
            "TrackPopupMenu",
            "WindowFromPoint",
            // # wingdi.h
            "BitBlt",
            "StretchBlt",
            "CreateSolidBrush",
            "SetTextColor",
            "SetBkMode",
            "SetBkColor",
            // # winbase.h
            "GlobalAlloc",
            "GlobalLock",
            "GlobalUnlock",
        ];

        /// This is a list of types that influence if a generated method will be marked as unsafe
        /// or not.
        ///
        /// If a method has no parameters or only parameters of such types in this list, it will
        /// not be marked as unsafe.
        ///
        /// Before this list existed, all non-pointer function parameter were considered safe. But
        /// that was not reliable because it sometimes accidently interpreted a parameter as not
        /// being a pointer while in fact it was one: It was a type alias that resolved to a
        /// pointer e.g. HWND.
        static SAFE_PARAM_TYPES: phf::Set<&'static str> = phf::phf_set![
            "::std::os::raw::c_int",
            "::std::os::raw::c_uint",
            "::std::os::raw::c_char",
            "root::reaper_functions::LICE_pixel",
            "f32",
            "f64",
            "bool"
        ];

        /// This is an include list of additional REAPER functions that don't need to be marked
        /// unsafe.
        ///
        /// Previously this has been
        static ADDITIONAL_SAFE_REAPER_FUNCTIONS: phf::Set<&'static str> = phf::phf_set![];

        /// This is an include list of additional SWELL functions that don't need to be marked
        /// unsafe.
        static ADDITIONAL_SAFE_SWELL_FUNCTIONS: phf::Set<&'static str> = phf::phf_set![];

        /// Generates `reaper.rs` and `swell.rs` from the previously generated `bindings.rs`
        pub fn generate_reaper_and_swell() {
            let file = parse_file("src/bindings.rs");
            generate_reaper(&file);
            generate_swell(&file);
        }

        /// Generates the `reaper.rs` file from the previously generated `bindings.rs`
        fn generate_reaper(file: &syn::File) {
            let fn_ptrs = parse_fn_ptrs(file, "reaper_functions");
            let result = generate_reaper_token_stream(&fn_ptrs);
            std::fs::write("src/reaper.rs", result.to_string()).expect("Unable to write file");
        }

        /// Generates the `swell.rs` file from the previously generated `bindings.rs`
        fn generate_swell(file: &syn::File) {
            let fn_ptrs = parse_fn_ptrs(file, "swell_functions");
            let result = generate_swell_token_stream(&fn_ptrs);
            std::fs::write("src/swell.rs", result.to_string()).expect("Unable to write file");
        }

        /// Generates the token stream. All of this could also be done in a procedural macro but
        /// I prefer the code generation approach for now.
        fn generate_reaper_token_stream(fn_ptrs: &Vec<FnPtr>) -> proc_macro2::TokenStream {
            let Compartments {
                names,
                fn_ptr_signatures,
                methods,
            } = build_compartments(
                fn_ptrs,
                &EXTERN_SYSTEM_ABI_REAPER_FUNCTIONS,
                &ADDITIONAL_SAFE_REAPER_FUNCTIONS,
            );
            let total_fn_ptr_count = names.len() as u32;
            quote::quote! {
                //! This file is automatically generated by executing `cargo build --features generate`.
                //!
                //! **Make adjustments in `build.rs`, not in this file!**
                #![allow(clippy::many_single_char_names)]
                #![allow(clippy::too_many_arguments)]
                #![allow(clippy::type_complexity)]
                #![allow(non_upper_case_globals)]
                #![allow(non_camel_case_types)]
                #![allow(non_snake_case)]

                use crate::{bindings::root, PluginContext};

                /// This is the low-level API access point to all REAPER functions.
                ///
                /// In order to use it, you first must obtain an instance of this struct by invoking [`load()`].
                ///
                /// `Default::default()` will give you an instance which panics on each function call. It's
                /// intended to be used for example code only.
                ///
                /// # Panics
                ///
                /// Please note that it's possible that functions are *not available*. This can be the case if
                /// the user runs your plug-in in an older version of REAPER which doesn't have that function yet.
                /// The availability of a function can be checked by inspecting the respective function pointer
                /// option accessible via the [`pointers()`] method. The actual methods in this structs are just
                /// convenience methods which unwrap the function pointers and panic if they are not available.
                ///
                /// [`load()`]: #method.load
                /// [`pointers()`]: #method.pointers
                #[derive(Copy, Clone, Debug, Default)]
                pub struct Reaper {
                    pub(crate) pointers: ReaperFunctionPointers,
                    // The only reason why this can be None is that we want to support Default. We want Default
                    // in order to be able to create rustdoc example code in higher-level APIs without needing a
                    // proper plug-in context.
                    pub(crate) plugin_context: Option<PluginContext>,
                }

                impl Reaper {
                    /// Loads all available REAPER functions from the given plug-in context.
                    ///
                    /// Returns a low-level `Reaper` instance which allows you to call these functions.
                    pub fn load(plugin_context: PluginContext) -> Reaper {
                        let mut loaded_count = 0;
                        let mut pointers = unsafe {
                            ReaperFunctionPointers {
                                loaded_count: 0,
                                #(
                                    #names: std::mem::transmute(plugin_context.GetFunc(c_str_macro::c_str!(stringify!(#names)).as_ptr())),
                                )*
                            }
                        };
                        #(
                            if pointers.#names.is_some() {
                                loaded_count += 1;
                            }
                        )*
                        pointers.loaded_count = loaded_count;
                        Reaper {
                            pointers,
                            plugin_context: Some(plugin_context)
                        }
                    }

                    #(
                        #methods
                    )*
                }

                /// Container for the REAPER function pointers.
                #[derive(Copy, Clone, Default)]
                pub struct ReaperFunctionPointers {
                    pub(crate) loaded_count: u32,
                    #(
                        pub #names: Option<#fn_ptr_signatures>,
                    )*
                }

                impl ReaperFunctionPointers {
                    pub(crate) const TOTAL_COUNT: u32 = #total_fn_ptr_count;
                }
            }
        }

        /// Generates the token stream. All of this could also be done in a procedural macro but
        /// I prefer the code generation approach for now.
        fn generate_swell_token_stream(fn_ptrs: &Vec<FnPtr>) -> proc_macro2::TokenStream {
            let Compartments {
                names,
                fn_ptr_signatures,
                methods,
            } = build_compartments(fn_ptrs, &phf::phf_set![], &ADDITIONAL_SAFE_SWELL_FUNCTIONS);
            let windows_functions: Vec<_> = fn_ptrs
                .iter()
                .filter_map(|p| {
                    if !WIN32_SWELL_FUNCTIONS.contains(p.name.to_string().as_str()) {
                        return None;
                    }
                    // It's important to use `extern "system"` here, otherwise the linker will
                    // complain about missing externals on Windows 32-bit.
                    Some(generate_function(&p, p.name.clone(), "system"))
                })
                .collect();
            let windows_methods: Vec<_> = fn_ptrs
                .iter()
                .filter_map(|p| {
                    if !WIN32_SWELL_FUNCTIONS.contains(p.name.to_string().as_str()) {
                        return None;
                    }
                    Some(generate_method(
                        &p,
                        generate_swell_windows_method_body(&p, p.name.clone()),
                        &ADDITIONAL_SAFE_SWELL_FUNCTIONS,
                    ))
                })
                .collect();
            let total_fn_ptr_count = names.len() as u32;
            quote::quote! {
                //! This file is automatically generated by executing `cargo build --features generate`.
                //!
                //! **Make adjustments in `build.rs`, not in this file!**
                #![allow(clippy::many_single_char_names)]
                #![allow(clippy::too_many_arguments)]
                #![allow(clippy::type_complexity)]
                #![allow(non_upper_case_globals)]
                #![allow(non_camel_case_types)]
                #![allow(non_snake_case)]
                #![allow(unused_unsafe)]

                use crate::{bindings::root, PluginContext};

                /// This is the low-level API access point to all SWELL functions.
                ///
                /// SWELL is the Simple Windows Emulation Layer and is exposed by REAPER for Linux
                /// and Mac OS X.
                ///
                /// See [`Reaper`] for details how to use this struct (it's very similar).
                ///
                /// [`Reaper`]: struct.Reaper.html
                #[derive(Copy, Clone, Debug, Default)]
                pub struct Swell {
                    pub(crate) pointers: SwellFunctionPointers,
                    // The only reason why this can be None is that we want to support Default. We want Default
                    // in order to be able to create rustdoc example code in higher-level APIs without needing a
                    // proper plug-in context.
                    pub(crate) plugin_context: Option<PluginContext>,
                }

                impl Swell {
                    /// Loads all available SWELL functions from the given plug-in context.
                    ///
                    /// Returns a `Swell` instance which allows you to call these functions.
                    ///
                    /// On Windows, this function will not load any function pointers because
                    /// the methods in this struct delegate to the corresponding Windows functions.
                    ///
                    /// # Panics
                    ///
                    /// If this is Linux and the SWELL function provider is not available, this
                    /// function panics.
                    pub fn load(plugin_context: PluginContext) -> Swell {
                        #[cfg(target_family = "windows")]
                        {
                            Swell {
                                pointers: Default::default(),
                                plugin_context: Some(plugin_context)
                            }
                        }
                        #[cfg(target_family = "unix")]
                        {
                            let mut loaded_count = 0;
                            let get_func = plugin_context.swell_function_provider()
                                .expect("SWELL function provider not available");
                            let mut pointers = unsafe {
                                SwellFunctionPointers {
                                    loaded_count: 0,
                                    #(
                                        #names: std::mem::transmute(get_func(c_str_macro::c_str!(stringify!(#names)).as_ptr())),
                                    )*
                                }
                            };
                            #(
                                if pointers.#names.is_some() {
                                    loaded_count += 1;
                                }
                            )*
                            pointers.loaded_count = loaded_count;
                            Swell {
                                pointers,
                                plugin_context: Some(plugin_context)
                            }
                        }
                    }

                    #(
                        #[cfg(target_family = "unix")]
                        #methods
                    )*

                    #(
                        #[cfg(target_family = "windows")]
                        #windows_methods
                    )*
                }

                /// Container for the SWELL function pointers.
                #[derive(Copy, Clone, Default)]
                pub struct SwellFunctionPointers {
                    pub(crate) loaded_count: u32,
                    #(
                        pub #names: Option<#fn_ptr_signatures>,
                    )*
                }

                impl SwellFunctionPointers {
                    pub(crate) const TOTAL_COUNT: u32 = #total_fn_ptr_count;
                }

                #[cfg(target_family = "windows")]
                mod windows {
                    use crate::bindings::root;

                    #(
                        #windows_functions
                    )*
                }
            }
        }

        /// This also modifies some unsafety and ABI fields based on the given sets.
        fn build_compartments(
            fn_ptrs: &Vec<FnPtr>,
            extern_system_functions: &phf::Set<&'static str>,
            safe_functions: &phf::Set<&'static str>,
        ) -> Compartments {
            Compartments {
                names: fn_ptrs.iter().map(|p| p.name.clone()).collect(),
                fn_ptr_signatures: fn_ptrs
                    .iter()
                    .map(|p| {
                        let function_name = p.name.to_string();
                        let abi = if extern_system_functions.contains(function_name.as_str()) {
                            "system"
                        } else {
                            "C"
                        };
                        TypeBareFn {
                            abi: Some(Abi {
                                extern_token: Extern {
                                    span: Span::call_site(),
                                },
                                name: Some(syn::LitStr::new(abi, Span::call_site())),
                            }),
                            unsafety: if p.is_safe(safe_functions) {
                                None
                            } else {
                                p.signature.unsafety.clone()
                            },
                            ..p.signature.clone()
                        }
                    })
                    .collect(),
                methods: fn_ptrs
                    .iter()
                    .map(|p| generate_method(&p, generate_method_body(&p), safe_functions))
                    .collect(),
            }
        }

        fn parse_file(path: &str) -> syn::File {
            let mut rust_file = File::open(path).expect("Unable to open file to be parsed");
            let mut src = String::new();
            rust_file
                .read_to_string(&mut src)
                .expect("Unable to read file");
            syn::parse_file(&src).expect("Unable to parse file")
        }

        /// Parses the names and signatures of the function pointers from `bindings.rs`.
        fn parse_fn_ptrs(file: &syn::File, module_name: &str) -> Vec<FnPtr> {
            filter_fn_ptr_items(file, module_name)
                .into_iter()
                .map(map_to_fn_ptr)
                .collect()
        }

        /// Generates a method definition in the body of e.g. `impl Reaper`.
        fn generate_method(
            ptr: &FnPtr,
            body: Block,
            safe_functions: &phf::Set<&'static str>,
        ) -> ImplItem {
            let is_safe = ptr.is_safe(safe_functions);
            let attrs = if is_safe {
                vec![]
            } else {
                vec![
                    syn::parse_quote! {
                        /// # Safety
                    },
                    syn::parse_quote! {
                        ///
                    },
                    syn::parse_quote! {
                        /// REAPER can crash if you pass an invalid pointer.
                    },
                ]
            };
            ImplItem::Method(ImplItemMethod {
                attrs,
                vis: Visibility::Public(VisPublic {
                    pub_token: Pub {
                        span: Span::call_site(),
                    },
                }),
                defaultness: None,
                sig: extract_signature(ptr, ptr.name.clone(), !is_safe, true),
                block: body,
            })
        }

        /// Generates a "extern C" or "extern system" free function definition
        fn generate_function(ptr: &FnPtr, name: Ident, extern_type: &str) -> Item {
            Item::ForeignMod(ItemForeignMod {
                attrs: vec![],
                abi: Abi {
                    extern_token: Extern {
                        span: Span::call_site(),
                    },
                    name: Some(syn::LitStr::new(extern_type, Span::call_site())),
                },
                brace_token: syn::token::Brace {
                    span: Span::call_site(),
                },
                items: vec![ForeignItem::Fn(ForeignItemFn {
                    attrs: vec![],
                    vis: Visibility::Public(VisPublic {
                        pub_token: Pub {
                            span: Span::call_site(),
                        },
                    }),
                    sig: extract_signature(ptr, name, false, false),
                    semi_token: syn::token::Semi {
                        spans: [Span::call_site()],
                    },
                })],
            })
        }

        fn extract_signature(
            ptr: &FnPtr,
            name: Ident,
            make_unsafe: bool,
            as_method: bool,
        ) -> Signature {
            Signature {
                constness: None,
                asyncness: None,
                unsafety: if make_unsafe {
                    Some(Unsafe {
                        span: Span::call_site(),
                    })
                } else {
                    None
                },
                abi: None,
                fn_token: Fn {
                    span: Span::call_site(),
                },
                ident: name,
                generics: Default::default(),
                paren_token: Paren {
                    span: Span::call_site(),
                },
                inputs: {
                    let actual_args = ptr.signature.inputs.iter().map(|a| {
                        FnArg::Typed(PatType {
                            attrs: vec![],
                            pat: Box::new(Pat::Ident(PatIdent {
                                attrs: vec![],
                                by_ref: None,
                                mutability: None,
                                ident: a.name.clone().unwrap().0,
                                subpat: None,
                            })),
                            colon_token: Colon {
                                spans: [Span::call_site()],
                            },
                            ty: Box::new(a.ty.clone()),
                        })
                    });
                    if as_method {
                        let receiver = FnArg::Receiver(Receiver {
                            attrs: vec![],
                            reference: Some((
                                And {
                                    spans: [Span::call_site()],
                                },
                                None,
                            )),
                            mutability: None,
                            self_token: SelfValue {
                                span: Span::call_site(),
                            },
                        });
                        std::iter::once(receiver).chain(actual_args).collect()
                    } else {
                        actual_args.collect()
                    }
                },
                variadic: None,
                output: ptr.signature.output.clone(),
            }
        }

        /// Generates the body of a method in e.g. `impl Reaper`
        fn generate_method_body(ptr: &FnPtr) -> Block {
            let name = &ptr.name;
            let fn_ptr_call =
                generate_fn_ptr_call(&ptr.signature, Ident::new("f", Span::call_site()));
            syn::parse_quote! {
                {
                    match self.pointers.#name {
                        None => panic!(format!(
                            "Attempt to use a function that has not been loaded: {}",
                            stringify!(#name)
                        )),
                        Some(f) => #fn_ptr_call,
                    }
                }
            }
        }

        /// Generates the body of a Swell method that just delegates to the corresponding Windows
        /// function instead of calling the function pointer (which wouldn't exist because on
        /// Windows SWELL is neither available nor necessary).
        fn generate_swell_windows_method_body(ptr: &FnPtr, fn_name: Ident) -> Block {
            let fn_ptr_call = generate_fn_ptr_call(&ptr.signature, fn_name);
            syn::parse_quote! {
                {
                    unsafe { windows::#fn_ptr_call }
                }
            }
        }

        /// Generates the actual function pointer call in the body of a method in `impl Reaper`
        fn generate_fn_ptr_call(signature: &TypeBareFn, fn_name: Ident) -> Expr {
            Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: {
                            let mut p = Punctuated::new();
                            let ps = PathSegment {
                                ident: fn_name,
                                arguments: Default::default(),
                            };
                            p.push(ps);
                            p
                        },
                    },
                })),
                paren_token: Paren {
                    span: Span::call_site(),
                },
                args: signature
                    .inputs
                    .iter()
                    .map(|a| {
                        Expr::Path(ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: {
                                    let mut p = Punctuated::new();
                                    let ps = PathSegment {
                                        ident: a.name.clone().unwrap().0,
                                        arguments: Default::default(),
                                    };
                                    p.push(ps);
                                    p
                                },
                            },
                        })
                    })
                    .collect(),
            })
        }

        /// Extracts the items of the given `bindings.rs` syntax tree and sub module of root that
        /// contains function pointers.
        fn filter_fn_ptr_items<'a>(
            bindings_tree: &'a syn::File,
            module_name: &str,
        ) -> Vec<&'a ForeignItemStatic> {
            let (_, root_mod_items) = match bindings_tree.items.as_slice() {
                [Item::Mod(ItemMod {
                    ident: id,
                    content: Some(c),
                    ..
                })] if id == "root" => c,
                _ => panic!("root mod not found"),
            };
            let (_, fn_ptr_mod_items) = root_mod_items
                .iter()
                .find_map(|item| match item {
                    Item::Mod(ItemMod {
                        ident: id,
                        content: Some(c),
                        ..
                    }) if id == module_name => Some(c),
                    _ => None,
                })
                .expect("function pointer mod not found");
            fn_ptr_mod_items
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

        /// Converts a syntax tree item which represents a REAPER function pointer to our
        /// convenience struct `ReaperFnPtr`
        fn map_to_fn_ptr(item: &ForeignItemStatic) -> FnPtr {
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
            FnPtr {
                name: item.ident.clone(),
                signature: bare_fn.clone(),
            }
        }

        struct Compartments {
            pub names: Vec<Ident>,
            pub fn_ptr_signatures: Vec<TypeBareFn>,
            pub methods: Vec<ImplItem>,
        }

        /// Contains the name and signature of a relevant function pointer
        #[derive(Clone)]
        struct FnPtr {
            name: Ident,
            signature: TypeBareFn,
        }

        impl FnPtr {
            /// Returns if the function represented by this function pointer can be considered safe.
            ///
            /// Is used for omitting the `unsafe` wherever possible.
            fn is_safe(&self, additional_safe_functions: &phf::Set<&'static str>) -> bool {
                // If this function is explicitly mentioned on the given include list, consider it
                // as safe.
                if additional_safe_functions.contains(self.name.to_string().as_str()) {
                    return true;
                }
                // If all parameters of this function are on a whitelist of "safe" types, consider
                // it as safe.
                self.signature.inputs.iter().all(|a| match &a.ty {
                    Type::Path(p) => {
                        let quoted = quote::quote! { #p };
                        let path_without_spaces = quoted.to_string().replace(" ", "");
                        SAFE_PARAM_TYPES.contains(path_without_spaces.as_str())
                    }
                    _ => false,
                })
            }
        }
    }
}
