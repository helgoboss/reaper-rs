#pragma once

#include "../lib/reaper/reaper_plugin.h"

namespace reaper_pcm_source {
  // This function is called from Rust and implemented in C++. It instantiates a C++ PCM_source and returns
  // its address to Rust.
  extern "C" PCM_source* add_pcm_source(void* callback_target);

  // This function is called from Rust and implemented in C++. It destroys the given C++ PCM_source object.
  extern "C" void remove_pcm_source(PCM_source* source);

  // All of the following functions are called from C++ and implemented in Rust.
  extern "C" double GetLength(void* callback_target);
  extern "C" PCM_source* Duplicate(void* callback_target);
  extern "C" const char* GetType(void* callback_target);
  extern "C" const char* GetFileName(void* callback_target);
  extern "C" PCM_source* GetSource(void* callback_target);
  extern "C" bool IsAvailable(void* callback_target);
  extern "C" void SetAvailable(void* callback_target, bool avail);
  extern "C" bool SetFileName(void* callback_target, const char *newfn);
  extern "C" void SetSource(void* callback_target, PCM_source *src);
  extern "C" int GetNumChannels(void* callback_target);
  extern "C" double GetSampleRate(void* callback_target);
  extern "C" double GetLengthBeats(void* callback_target);
  extern "C" int GetBitsPerSample(void* callback_target);
  extern "C" double GetPreferredPosition(void* callback_target);
  extern "C" int PropertiesWindow(void* callback_target, HWND hwndParent);
  extern "C" void GetSamples(void* callback_target, PCM_source_transfer_t *block);
  extern "C" void GetPeakInfo(void* callback_target, PCM_source_peaktransfer_t *block);
  extern "C" void SaveState(void* callback_target, ProjectStateContext *ctx);
  extern "C" int LoadState(void* callback_target, const char *firstline, ProjectStateContext *ctx);
  extern "C" void Peaks_Clear(void* callback_target, bool deleteFile);
  extern "C" int PeaksBuild_Begin(void* callback_target);
  extern "C" int PeaksBuild_Run(void* callback_target);
  extern "C" void PeaksBuild_Finish(void* callback_target);
  extern "C" int rust_PCM_source_Extended(void* callback_target, int call, void *parm1, void *parm2, void *parm3);

  // All the following functions are called from Rust and implemented in C++. The implementation simply delegates
  // to the respective method of the `self` object. This glue code is necessary because Rust can't call  C++ pure 
  // virtual functions directly.
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