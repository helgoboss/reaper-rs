// This file aggregates all header files that serve as input for generating `bindings.rs` with bindgen.

// Make all types relevant for REAPER plug-ins visible to bindgen.
#include "../lib/reaper/reaper_plugin.h"

// Make all REAPER functions (function pointers to be exact) visible under dedicated namespace, so we can easily include
// them with bindgen. None of the function pointers will actually be used in the real plug-in. They just serve as
// input for stage two of the generation process: The generation of `reaper.rs`.
namespace reaper_functions {
  #include "../lib/reaper/reaper_plugin_functions.h"
  #include "../lib/reaper/more_reaper_plugin_functions.h"
  #include "../lib/reaper/coolscroll_reaper_plugin_functions.h"
}

// Make all SWELL functions visible under dedicated namespace, so we can easily include them with bindgen.
// Above inclusion of "reaper_plugin.h" already included SWELL as well. So we need to undefine some stuff to be able
// to include it again - under a different namespace and with SWELL_PROVIDED_BY_APP defined, which makes the SWELL
// functions end up as function pointers - exactly like the REAPER functions. They also serve as input for stage two
// of the generation process: The generation of `swell.rs`.
#undef _WDL_SWELL_H_API_DEFINED_
#undef SWELL_API_DEFINE
#define SWELL_PROVIDED_BY_APP
namespace swell_functions {
#include "../lib/WDL/WDL/swell/swell-functions.h"

  // We pick macOS-specific functions by hand.
  extern "C" bool (*SWELL_osx_is_dark_mode)(int mode);
}

// Make C++ glue code functions visible to bindgen. They will be used in the real application.
#include "control_surface.hpp"
#include "midi.hpp"
#include "pcm_source.hpp"