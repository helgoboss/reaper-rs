#pragma once

#include "../../lib/reaper/reaper_plugin.h"

namespace reaper_rs_midi {
  // This is implemented in C++ and called from Rust
  extern "C" MIDI_event_t* MIDI_eventlist_EnumItems(MIDI_eventlist* self, int* bpos);
  extern "C" MIDI_eventlist* midi_Input_GetReadBuf(midi_Input* self);
}