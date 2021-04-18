#pragma once

#include "../lib/reaper/reaper_plugin.h"

// All the functions in this namespace are called from Rust and implemented in C++. The implementation simply delegates
// to the respective method of the `self` object. This glue code is necessary because Rust can't call  C++ pure virtual
// functions directly.
namespace reaper_pcm_source {
  extern "C" double PCM_source_GetLength(PCM_source* self);
}