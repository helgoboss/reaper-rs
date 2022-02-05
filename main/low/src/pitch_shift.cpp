#include "pitch_shift.hpp"

namespace reaper_pitch_shift {
  void delete_reaper_pitch_shift(IReaperPitchShift* pitch_shift) {
    delete pitch_shift;
  }

  void IReaperPitchShift_set_srate(IReaperPitchShift* self, double srate) {
    self->set_srate(srate);
  }
  void IReaperPitchShift_set_nch(IReaperPitchShift* self, int nch) {
    self->set_nch(nch);
  }
  void IReaperPitchShift_set_shift(IReaperPitchShift* self, double shift) {
    self->set_shift(shift);
  }
  void IReaperPitchShift_set_formant_shift(IReaperPitchShift* self, double shift) {
    self->set_formant_shift(shift);
  }
  void IReaperPitchShift_set_tempo(IReaperPitchShift* self, double tempo) {
    self->set_tempo(tempo);
  }
  void IReaperPitchShift_Reset(IReaperPitchShift* self) {
    self->Reset();
  }
  ReaSample* IReaperPitchShift_GetBuffer(IReaperPitchShift* self, int size) {
    return self->GetBuffer(size);
  }
  void IReaperPitchShift_BufferDone(IReaperPitchShift* self, int input_filled) {
    self->BufferDone(input_filled);
  }
  void IReaperPitchShift_FlushSamples(IReaperPitchShift* self) {
    self->FlushSamples();
  }
  bool IReaperPitchShift_IsReset(IReaperPitchShift* self) {
    return self->IsReset();
  }
  int IReaperPitchShift_GetSamples(IReaperPitchShift* self, int requested_output, ReaSample* buffer) {
    return self->GetSamples(requested_output, buffer);
  }
  void IReaperPitchShift_SetQualityParameter(IReaperPitchShift* self, int parm) {
    self->SetQualityParameter(parm);
  }
  int IReaperPitchShift_Extended(IReaperPitchShift* self, int call, void* parm1, void* parm2, void* parm3) {
    return self->Extended(call, parm1, parm2, parm3);
  }
}