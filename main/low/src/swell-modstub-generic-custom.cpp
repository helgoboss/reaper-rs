// This is a stripped-down version of the official "swell-modstub-generic.cpp".
//
// The purpose is the same: To reuse the existing (and maybe customized) SWELL provided by REAPER for Linux instead
// of compiling SWELL from scratch. This is done by defining all SWELL functions as a bunch of global function pointers
// and provide one entry point function to be called by REAPER which uses the passed SWELL function provider to initialize
// all those function pointers.
//
// The difference to "swell-modstub-generic.cpp" is that in our case the entry point function is named
// "register_swell_function_provider_called_from_rust" and not "SWELL_dllMain". We do that because *reaper-rs* already
// defines an entry point named "SWELL_dllMain" in the macros "reaper_extension_plugin!" and "reaper_vst_plugin!". Why?
// Because *reaper-rs* is interested in the SWELL function as well. It has its own mechanism of exposing the SWELL
// function pointers provided by REAPER (the `Swell` struct). When REAPER calls the SWELL entry point function
// ("SWELL_dllMain"), *reaper-rs* does 2 things:
//
// 1. It delegates to the "register_swell_function_provider_called_from_rust" implemented in this C++ source file (so
//    the C++ SWELL function pointers will be initialized).
// 2. It gets hold of the SWELL function provider pointer (for its own purposes).
//
// Why would we want the global C++ SWELL function pointers to be initialized in the first place? After all we said that
// *reaper-rs* has its own way of exposing SWELL. The problem is that the SWELL functions exposed by *reaper-rs* are
// not enough for creating a complete UI. They are enough to modify existing window controls and stuff. But for the
// initial creation of a window, including buttons, text fields etc., it needs more.
//
// SWELL's only mechanism to do this is to use `CreateDialogParam()`. This function exists in the Win32 API, too. There
// it is combined with an old-school Windows dialog resource file (RC file) which can be created with a nice WYSIWYG
// editor like ResEdit. SWELL, on the other hand, can't work with the RC file directly. The RC file first needs to be
// converted (via a PHP script called "mac_resgen.php") and the result needs to be put into a C++ file that includes
// "swell-dlggen.h". This C++ file makes super heavy use of preprocessor macros, static variables, static functions ...
// very difficult to see through this. So I didn't port it to Rust. There's also no need for that. Plug-ins can easily
// use the cc crate to use the SWELL dialog generation stuff.

#define SWELL_API_DEFPARM(x)
#define SWELL_API_DEFINE(ret, func, parms) ret (*func) parms ;
extern "C" {
#include "../lib/WDL/WDL/swell/swell.h"
};

// only include this file in projects that are linked to libSwell.so

struct SWELL_CursorResourceIndex *SWELL_curmodule_cursorresource_head;
struct SWELL_DialogResourceIndex *SWELL_curmodule_dialogresource_head;
struct SWELL_MenuResourceIndex *SWELL_curmodule_menuresource_head;

// define the functions

static struct {
    const char *name;
    void **func;
} api_tab[] = {

#undef _WDL_SWELL_H_API_DEFINED_
#undef SWELL_API_DEFINE
#define SWELL_API_DEFINE(ret, func, parms) {#func, (void **)&func },

#include "../lib/WDL/WDL/swell/swell.h"

};

static int dummyFunc() { return 0; }

static int doinit(void *(*GetFunc)(const char *name)) {
    int errcnt = 0;
    for (int x = 0; x < sizeof(api_tab) / sizeof(api_tab[0]); x++) {
        *api_tab[x].func = GetFunc(api_tab[x].name);
        if (!*api_tab[x].func) {
            printf("SWELL API not found: %s\n", api_tab[x].name);
            errcnt++;
            *api_tab[x].func = (void *) &dummyFunc;
        }
    }
    return errcnt;
}

// reaper-rs change.
// This will be called by Rust (the important difference).
extern "C" __attribute__ ((visibility ("default"))) void register_swell_function_provider_called_from_rust(LPVOID _GetFunc) {
    if (_GetFunc) {  
        doinit((void *(*)(const char *)) _GetFunc);
    } 
}