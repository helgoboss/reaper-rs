#include "midi.hpp"

namespace reaper_midi {
  // MIDI_eventlist methods

  MIDI_event_t* MIDI_eventlist_EnumItems(MIDI_eventlist* self, int* bpos) {
    return self->EnumItems(bpos);
  }

  void MIDI_eventlist_AddItem(MIDI_eventlist* self, MIDI_event_t* evt) {
    self->AddItem(evt);
  }

  // midi_Input methods

  MIDI_eventlist* midi_Input_GetReadBuf(midi_Input* self) {
    return self->GetReadBuf();
  }

  // midi_Output methods

  void midi_Output_Send(midi_Output* self, unsigned char status, unsigned char d1, unsigned char d2, int frame_offset) {
    self->Send(status, d1, d2, frame_offset);
  }

  void midi_Output_SendMsg(midi_Output* self, MIDI_event_t* msg, int frame_offset) {
    self->SendMsg(msg, frame_offset);
  }
}