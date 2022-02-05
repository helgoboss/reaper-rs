#pragma once

#include "../lib/reaper/reaper_plugin.h"

// All the functions in this namespace are called from Rust and implemented in C++. The implementation simply delegates
// to the respective method of the `self` object. This glue code is necessary because Rust can't call  C++ pure virtual
// functions directly.
namespace reaper_resample {
  // This function is called from Rust and implemented in C++. It destroys the given C++ REAPER_Resample_Interface object.
  extern "C" void delete_reaper_resample_interface(REAPER_Resample_Interface* resample_interface);

  extern "C" void REAPER_Resample_Interface_SetRates(REAPER_Resample_Interface* self, double rate_in, double rate_out);
  extern "C" void REAPER_Resample_Interface_Reset(REAPER_Resample_Interface* self);
  extern "C" double REAPER_Resample_Interface_GetCurrentLatency(REAPER_Resample_Interface* self);
  extern "C" int REAPER_Resample_Interface_ResamplePrepare(REAPER_Resample_Interface* self, int out_samples, int nch, ReaSample **inbuffer);
  extern "C" int REAPER_Resample_Interface_ResampleOut(REAPER_Resample_Interface* self, ReaSample *out, int nsamples_in, int nsamples_out, int nch);
  extern "C" int REAPER_Resample_Interface_Extended(REAPER_Resample_Interface* self, int call, void *parm1, void *parm2, void *parm3);
}