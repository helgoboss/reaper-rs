#pragma once

#include "../lib/reaper/reaper_plugin.h"

namespace reaper_pcm_sink {
  // This function is called from Rust and implemented in C++. It instantiates a C++ PCM_sink and returns
  // its address to Rust.
  extern "C" PCM_sink* create_cpp_to_rust_pcm_sink(void* callback_target);

  // This function is called from Rust and implemented in C++. It destroys the given C++ PCM_sink object.
  extern "C" void delete_pcm_sink(PCM_sink* sink);

  // All of the following functions are called from C++ and implemented in Rust.
  extern "C" void cpp_to_rust_PCM_sink_GetOutputInfoString(void* callback_target, char* buf, int buflen);
  extern "C" double cpp_to_rust_PCM_sink_GetStartTime(void* callback_target);
  extern "C" void cpp_to_rust_PCM_sink_SetStartTime(void* callback_target, double st);
  extern "C" const char* cpp_to_rust_PCM_sink_GetFileName(void* callback_target);
  extern "C" int cpp_to_rust_PCM_sink_GetNumChannels(void* callback_target);
  extern "C" double cpp_to_rust_PCM_sink_GetLength(void* callback_target);
  extern "C" INT64 cpp_to_rust_PCM_sink_GetFileSize(void* callback_target);
  extern "C" void cpp_to_rust_PCM_sink_WriteMIDI(void* callback_target, MIDI_eventlist* events, int len, double samplerate);
  extern "C" void cpp_to_rust_PCM_sink_WriteDoubles(void* callback_target, ReaSample** samples, int len, int nch, int offset, int spacing);
  extern "C" bool cpp_to_rust_PCM_sink_WantMIDI(void* callback_target);
  extern "C" int cpp_to_rust_PCM_sink_GetLastSecondPeaks(void* callback_target, int sz, ReaSample* buf);
  extern "C" void cpp_to_rust_PCM_sink_GetPeakInfo(void* callback_target, PCM_source_peaktransfer_t* block);
  extern "C" int cpp_to_rust_PCM_sink_Extended(void* callback_target, int call, void* parm1, void* parm2, void* parm3);

  // All the following functions are called from Rust and implemented in C++. The implementation simply delegates
  // to the respective method of the `self` object. This glue code is necessary because Rust can't call  C++ pure 
  // virtual functions directly.

  extern "C" void rust_to_cpp_PCM_sink_GetOutputInfoString(PCM_sink* self, char* buf, int buflen);
  extern "C" double rust_to_cpp_PCM_sink_GetStartTime(PCM_sink* self);
  extern "C" void rust_to_cpp_PCM_sink_SetStartTime(PCM_sink* self, double st);
  extern "C" const char* rust_to_cpp_PCM_sink_GetFileName(PCM_sink* self);
  extern "C" int rust_to_cpp_PCM_sink_GetNumChannels(PCM_sink* self);
  extern "C" double rust_to_cpp_PCM_sink_GetLength(PCM_sink* self);
  extern "C" INT64 rust_to_cpp_PCM_sink_GetFileSize(PCM_sink* self);
  extern "C" void rust_to_cpp_PCM_sink_WriteMIDI(PCM_sink* self, MIDI_eventlist* events, int len, double samplerate);
  extern "C" void rust_to_cpp_PCM_sink_WriteDoubles(PCM_sink* self, ReaSample** samples, int len, int nch, int offset, int spacing);
  extern "C" bool rust_to_cpp_PCM_sink_WantMIDI(PCM_sink* self);
  extern "C" int rust_to_cpp_PCM_sink_GetLastSecondPeaks(PCM_sink* self, int sz, ReaSample* buf);
  extern "C" void rust_to_cpp_PCM_sink_GetPeakInfo(PCM_sink* self, PCM_source_peaktransfer_t* block);
  extern "C" int rust_to_cpp_PCM_sink_Extended(PCM_sink* self, int call, void* parm1, void* parm2, void* parm3);
}