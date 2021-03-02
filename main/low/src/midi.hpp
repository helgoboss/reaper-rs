#pragma once

#include "../lib/reaper/reaper_plugin.h"

// All the functions in this namespace are called from Rust and implemented in C++. The implementation simply delegates
// to the respective method of the `self` object. This glue code is necessary because Rust can't call  C++ pure virtual
// functions directly.
namespace reaper_midi {
  // MIDI_eventlist methods
  extern "C" MIDI_event_t* MIDI_eventlist_EnumItems(MIDI_eventlist* self, int* bpos);
  extern "C" void MIDI_eventlist_AddItem(MIDI_eventlist* self, MIDI_event_t* evt);

  // midi_Input methods
  extern "C" MIDI_eventlist* midi_Input_GetReadBuf(midi_Input* self);

  // midi_Output methods
  extern "C" void midi_Output_Send(midi_Output* self, unsigned char status, unsigned char d1, unsigned char d2, int frame_offset);
  extern "C" void midi_Output_SendMsg(midi_Output* self, MIDI_event_t* msg, int frame_offset);
}