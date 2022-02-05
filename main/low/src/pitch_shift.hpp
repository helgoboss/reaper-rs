#pragma once

#include "../lib/reaper/reaper_plugin.h"

// All the functions in this namespace are called from Rust and implemented in C++. The implementation simply delegates
// to the respective method of the `self` object. This glue code is necessary because Rust can't call  C++ pure virtual
// functions directly.
namespace reaper_pitch_shift {
  // This function is called from Rust and implemented in C++. It destroys the given C++ IReaperPitchShift object.
  extern "C" void delete_reaper_pitch_shift(IReaperPitchShift* pitch_shift);

  extern "C" void IReaperPitchShift_set_srate(IReaperPitchShift* self, double srate);
  extern "C" void IReaperPitchShift_set_nch(IReaperPitchShift* self, int nch);
  extern "C" void IReaperPitchShift_set_shift(IReaperPitchShift* self, double shift);
  extern "C" void IReaperPitchShift_set_formant_shift(IReaperPitchShift* self, double shift);
  extern "C" void IReaperPitchShift_set_tempo(IReaperPitchShift* self, double tempo);
  extern "C" void IReaperPitchShift_Reset(IReaperPitchShift* self);
  extern "C" ReaSample *IReaperPitchShift_GetBuffer(IReaperPitchShift* self, int size);
  extern "C" void IReaperPitchShift_BufferDone(IReaperPitchShift* self, int input_filled);
  extern "C" void IReaperPitchShift_FlushSamples(IReaperPitchShift* self);
  extern "C" bool IReaperPitchShift_IsReset(IReaperPitchShift* self);
  extern "C" int IReaperPitchShift_GetSamples(IReaperPitchShift* self, int requested_output, ReaSample *buffer);
  extern "C" void IReaperPitchShift_SetQualityParameter(IReaperPitchShift* self, int parm);
  extern "C" int IReaperPitchShift_Extended(IReaperPitchShift* self, int call, void *parm1, void *parm2, void *parm3);
}