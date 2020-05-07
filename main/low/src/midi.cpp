#include "midi.hpp"

namespace reaper_midi {
  MIDI_event_t* MIDI_eventlist_EnumItems(MIDI_eventlist* self, int* bpos) {
    return self->EnumItems(bpos);
  }

  MIDI_eventlist* midi_Input_GetReadBuf(midi_Input* self) {
    return self->GetReadBuf();
  }
}