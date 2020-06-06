#include "midi.hpp"

namespace reaper_midi {
  MIDI_event_t* MIDI_eventlist_EnumItems(MIDI_eventlist* self, int* bpos) {
    return self->EnumItems(bpos);
  }

  MIDI_eventlist* midi_Input_GetReadBuf(midi_Input* self) {
    return self->GetReadBuf();
  }

  void midi_Output_Send(midi_Output* self, unsigned char status, unsigned char d1, unsigned char d2, int frame_offset) {
    self->Send(status, d1, d2, frame_offset);
  }
}