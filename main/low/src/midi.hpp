#pragma once

#include "../lib/reaper/reaper_plugin.h"

// All the functions in this namespace are called from Rust and implemented in C++. The implementation simply delegates
// to the respective method of the `self` object. This glue code is necessary because Rust can't call  C++ pure virtual
// functions directly.
namespace reaper_midi {
  extern "C" MIDI_event_t* MIDI_eventlist_EnumItems(MIDI_eventlist* self, int* bpos);
  extern "C" MIDI_eventlist* midi_Input_GetReadBuf(midi_Input* self);
  extern "C" void midi_Output_Send(midi_Output* self, unsigned char status, unsigned char d1, unsigned char d2, int frame_offset);
}