/// Executed whenever Cargo builds reaper-rs
fn main() {
    #[cfg(target_os = "linux")]
    #[cfg(feature = "generate")]
    codegen::generate_all();
    compile_glue_code();
}

/// Compiles C++ glue code. This is necessary to interact with those parts of the REAPER C++ API
/// that use pure virtual interface classes and therefore the C++ ABI.
fn compile_glue_code() {
    cc::Build::new()
        .cpp(true)
        .warnings(false)
        .file("src/control_surface.cpp")
        .file("src/midi.cpp")
        .compile("glue");
}

#[cfg(feature = "generate")]
mod codegen {
    /// Generates `bindings.rs`, `reaper.rs` and `swell.rs`
    pub fn generate_all() {
        phase_one::generate_bindings();
        phase_two::generate_all();
    }

    mod phase_one {
        use bindgen::callbacks::{IntKind, ParseCallbacks};

        #[derive(Debug)]
        struct CustomParseCallbacks;

        impl ParseCallbacks for CustomParseCallbacks {
            fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
                if name.starts_with("CSURF_EXT_")
                    || name.starts_with("VK_")
                    || name.starts_with("SW_")
                    || name == "REAPER_PLUGIN_VERSION"
                {
                    return Some(IntKind::I32);
                }
                // The following flags stay u32 although the APIs expect i32:
                // - UNDO_STATE_* (used as bitmask, UNDO_STATE_ALL doesn't fit into i32)
                None
            }
        }

        /// Generates the `bindings.rs` file from REAPER C++ headers
        pub fn generate_bindings() {
            println!("cargo:rerun-if-changed=src/wrapper.hpp");
            let bindings = bindgen::Builder::default()
                .header("src/wrapper.hpp")
                .opaque_type("timex")
                .derive_eq(true)
                .derive_partialeq(true)
                .derive_hash(true)
                .clang_arg("-xc++")
                .enable_cxx_namespaces()
                // If we activate layout tests, we would have to regenerate at each build because
                // tests will fail on Linux if generated on Windows and vice versa.
                .layout_tests(false)
                // Tell cargo to invalidate the built crate whenever any of the
                // included header files changed.
                .parse_callbacks(Box::new(bindgen::CargoCallbacks))
                .parse_callbacks(Box::new(CustomParseCallbacks))
                .raw_line("#![allow(clippy::all)]")
                .raw_line("#![allow(non_upper_case_globals)]")
                .raw_line("#![allow(non_camel_case_types)]")
                .raw_line("#![allow(non_snake_case)]")
                .raw_line("#![allow(dead_code)]")
                .whitelist_var("reaper_functions::.*")
                .whitelist_var("swell_functions::.*")
                .whitelist_var("SWELL_.*")
                .whitelist_var("CSURF_EXT_.*")
                .whitelist_var("REAPER_PLUGIN_VERSION")
                .whitelist_var("UNDO_STATE_.*")
                .whitelist_var("VK_.*")
                .whitelist_var("SW_.*")
                .whitelist_var("WM_.*")
                .whitelist_var("DLL_PROCESS_ATTACH")
                .whitelist_type("HINSTANCE")
                .whitelist_type("reaper_plugin_info_t")
                .whitelist_type("gaccel_register_t")
                .whitelist_type("audio_hook_register_t")
                .whitelist_type("KbdSectionInfo")
                .whitelist_type("GUID")
                .whitelist_function("reaper_control_surface::.*")
                .whitelist_function("reaper_midi::.*")
                .generate()
                .expect("Unable to generate bindings");
            let out_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            bindings
                .write_to_file(out_path.join("src/bindings.rs"))
                .expect("Couldn't write bindings!");
        }
    }

    mod phase_two {
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

        /// Functions which exist both in SWELL and in the Windows API.
        ///
        /// It's important that the signatures match.
        ///
        /// In some cases they might have a different name, e.g. Windows functions in which
        /// the character encoding matters, have a suffix. That's why this is a map:
        /// On the left side the SWELL function name, on the right side the one for Windows.
        /// Please note that only *real* functions are allowed on the right side, not function
        /// macros.
        static SWELL_WINDOWS_MAPPING: phf::Map<&'static str, &'static str> = phf::phf_map! {
            "DefWindowProc" => "DefWindowProcA",
            "DestroyWindow" => "DestroyWindow",
            "GetDlgItem" => "GetDlgItem",
            "ShowWindow" => "ShowWindow",
        };

        pub fn generate_all() {
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
            } = build_compartments(fn_ptrs);
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

                use crate::{bindings::root, ReaperPluginContext};
                use c_str_macro::c_str;

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
                    pub(crate) plugin_context: Option<ReaperPluginContext>,
                }

                impl Reaper {
                    /// Loads all available REAPER functions from the given plug-in context.
                    ///
                    /// Returns a low-level `Reaper` instance which allows you to call these functions.
                    pub fn load(plugin_context: ReaperPluginContext) -> Reaper {
                        let pointers = unsafe {
                            ReaperFunctionPointers {
                                #(
                                    #names: std::mem::transmute(plugin_context.GetFunc(c_str!(stringify!(#names)).as_ptr())),
                                )*
                            }
                        };
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
                    #(
                        pub #names: Option<#fn_ptr_signatures>,
                    )*
                }

                impl std::fmt::Debug for ReaperFunctionPointers {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        // TODO-low In future this could print "x of y functions loaded".
                        f.debug_struct("ReaperFunctionPointers")
                         .finish()
                    }
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
            } = build_compartments(fn_ptrs);
            let windows_functions: Vec<_> = fn_ptrs
                .iter()
                .filter_map(|p| {
                    let win_name = SWELL_WINDOWS_MAPPING.get(p.name.to_string().as_str())?;
                    Some(generate_function(
                        &p,
                        Ident::new(*win_name, Span::call_site()),
                    ))
                })
                .collect();
            let windows_methods: Vec<_> = fn_ptrs
                .iter()
                .filter_map(|p| {
                    let win_name = SWELL_WINDOWS_MAPPING.get(p.name.to_string().as_str())?;
                    Some(generate_method(
                        &p,
                        generate_swell_windows_method_body(
                            &p,
                            Ident::new(*win_name, Span::call_site()),
                        ),
                    ))
                })
                .collect();
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

                use crate::{bindings::root, ReaperPluginContext};

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
                    pub(crate) plugin_context: Option<ReaperPluginContext>,
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
                    pub fn load(plugin_context: ReaperPluginContext) -> Swell {
                        #[cfg(target_os = "windows")]
                        {
                            Swell {
                                pointers: Default::default(),
                                plugin_context: Some(plugin_context)
                            }
                        }
                        #[cfg(target_os = "linux")]
                        {
                            let get_func = plugin_context.swell_function_provider()
                                .expect("SWELL function provider not available");
                            let pointers = unsafe {
                                SwellFunctionPointers {
                                    #(
                                        #names: std::mem::transmute(get_func(c_str_macro::c_str!(stringify!(#names)).as_ptr())),
                                    )*
                                }
                            };
                            Swell {
                                pointers,
                                plugin_context: Some(plugin_context)
                            }
                        }
                    }

                    #(
                        #[cfg(target_os = "linux")]
                        #methods
                    )*

                    #(
                        #[cfg(target_os = "windows")]
                        #windows_methods
                    )*
                }

                /// Container for the SWELL function pointers.
                #[derive(Copy, Clone, Default)]
                pub struct SwellFunctionPointers {
                    #(
                        pub #names: Option<#fn_ptr_signatures>,
                    )*
                }

                impl std::fmt::Debug for SwellFunctionPointers {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        // TODO-low In future this could print "x of y functions loaded".
                        f.debug_struct("SwellFunctionPointers")
                         .finish()
                    }
                }

                #[cfg(target_os = "windows")]
                mod windows {
                    use crate::bindings::root;

                    #(
                        #windows_functions
                    )*
                }
            }
        }

        fn build_compartments(fn_ptrs: &Vec<FnPtr>) -> Compartments {
            Compartments {
                names: fn_ptrs.iter().map(|p| p.name.clone()).collect(),
                fn_ptr_signatures: fn_ptrs
                    .iter()
                    .map(|p| TypeBareFn {
                        unsafety: if p.has_pointer_args() {
                            p.signature.unsafety.clone()
                        } else {
                            None
                        },
                        ..p.signature.clone()
                    })
                    .collect(),
                methods: fn_ptrs
                    .iter()
                    .map(|p| generate_method(&p, generate_method_body(&p)))
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
        fn generate_method(ptr: &FnPtr, body: Block) -> ImplItem {
            let has_pointer_args = ptr.has_pointer_args();
            let attrs = if has_pointer_args {
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
            } else {
                vec![]
            };
            ImplItem::Method(ImplItemMethod {
                attrs,
                vis: Visibility::Public(VisPublic {
                    pub_token: Pub {
                        span: Span::call_site(),
                    },
                }),
                defaultness: None,
                sig: extract_signature(ptr, ptr.name.clone(), has_pointer_args, true),
                block: body,
            })
        }

        /// Generates a "extern C" free function definition
        fn generate_function(ptr: &FnPtr, name: Ident) -> Item {
            Item::ForeignMod(ItemForeignMod {
                attrs: vec![],
                abi: Abi {
                    extern_token: Extern {
                        span: Span::call_site(),
                    },
                    name: Some(syn::LitStr::new("C", Span::call_site())),
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
            fn has_pointer_args(&self) -> bool {
                self.signature.inputs.iter().any(|a| match a.ty {
                    Type::Ptr(_) => true,
                    _ => false,
                })
            }
        }
    }
}
