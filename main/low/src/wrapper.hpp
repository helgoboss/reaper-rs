// This file aggregates all header files that serve as input for generating `bindings.rs`.
#include "../lib/reaper/reaper_plugin.h"
namespace reaper_functions {
  #include "../lib/reaper/reaper_plugin_functions.h"
  #include "../lib/reaper/more_reaper_plugin_functions.h"
}
#undef _WDL_SWELL_H_API_DEFINED_
#undef SWELL_API_DEFINE
#define SWELL_PROVIDED_BY_APP
namespace swell_functions {
#include "../lib/WDL/WDL/swell/swell-functions.h"
}
#include "control_surface.hpp"
#include "control_surface.hpp"
#include "midi.hpp"