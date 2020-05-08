// This file aggregates all header files that serve as input for generating `bindings.rs`.
#include "../lib/reaper/reaper_plugin.h"
namespace reaper_functions {
  #include "../lib/reaper/reaper_plugin_functions.h"
  #include "../lib/reaper/more_reaper_plugin_functions.h"
}
#include "../lib/WDL/WDL/swell/swell.h"
#include "control_surface.hpp"
#include "midi.hpp"