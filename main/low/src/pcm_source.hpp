#pragma once

#include "../lib/reaper/reaper_plugin.h"

// All the functions in this namespace are called from Rust and implemented in C++. The implementation simply delegates
// to the respective method of the `self` object. This glue code is necessary because Rust can't call  C++ pure virtual
// functions directly.
namespace reaper_pcm_source {
  extern "C" double PCM_source_GetLength(PCM_source* self);
  extern "C" PCM_source* PCM_source_Duplicate(PCM_source* self);
  extern "C" const char* PCM_source_GetType(PCM_source* self);
  extern "C" const char* PCM_source_GetFileName(PCM_source* self);
  extern "C" PCM_source* PCM_source_GetSource(PCM_source* self);
  extern "C" bool PCM_source_IsAvailable(PCM_source* self);
  extern "C" void PCM_source_SetAvailable(PCM_source* self, bool avail);
  extern "C" bool PCM_source_SetFileName(PCM_source* self, const char *newfn);
  extern "C" void PCM_source_SetSource(PCM_source* self, PCM_source *src);
  extern "C" int PCM_source_GetNumChannels(PCM_source* self);
  extern "C" double PCM_source_GetSampleRate(PCM_source* self);
  extern "C" double PCM_source_GetLengthBeats(PCM_source* self);
  extern "C" int PCM_source_GetBitsPerSample(PCM_source* self);
  extern "C" double PCM_source_GetPreferredPosition(PCM_source* self);
  extern "C" int PCM_source_PropertiesWindow(PCM_source* self, HWND hwndParent);
  extern "C" void PCM_source_GetSamples(PCM_source* self, PCM_source_transfer_t *block);
  extern "C" void PCM_source_GetPeakInfo(PCM_source* self, PCM_source_peaktransfer_t *block);
  extern "C" void PCM_source_SaveState(PCM_source* self, ProjectStateContext *ctx);
  extern "C" int PCM_source_LoadState(PCM_source* self, const char *firstline, ProjectStateContext *ctx);
  extern "C" void PCM_source_Peaks_Clear(PCM_source* self, bool deleteFile);
  extern "C" int PCM_source_PeaksBuild_Begin(PCM_source* self);
  extern "C" int PCM_source_PeaksBuild_Run(PCM_source* self);
  extern "C" void PCM_source_PeaksBuild_Finish(PCM_source* self);
  extern "C" int PCM_source_Extended(PCM_source* self, int call, void *parm1, void *parm2, void *parm3);
}