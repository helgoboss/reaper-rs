#include "pcm_source.hpp"

namespace reaper_pcm_source {
  double PCM_source_GetLength(PCM_source* self) {
    return self->GetLength();
  }
  PCM_source* PCM_source_Duplicate(PCM_source* self) {
    return self->Duplicate();
  }
  const char* PCM_source_GetType(PCM_source* self) {
    return self->GetType();
  }
  const char* PCM_source_GetFileName(PCM_source* self) {
    return self->GetFileName();
  }
  PCM_source* PCM_source_GetSource(PCM_source* self) {
    return self->GetSource();
  }
  bool PCM_source_IsAvailable(PCM_source* self) {
    return self->IsAvailable();
  }
  void PCM_source_SetAvailable(PCM_source* self, bool avail) {
    self->SetAvailable(avail);
  }
  bool PCM_source_SetFileName(PCM_source* self, const char* newfn) {
    return self->SetFileName(newfn);
  }
  void PCM_source_SetSource(PCM_source* self, PCM_source* src) {
    self->SetSource(src);
  }
  int PCM_source_GetNumChannels(PCM_source* self) {
    return self->GetNumChannels();
  }
  double PCM_source_GetSampleRate(PCM_source* self) {
    return self->GetSampleRate();
  }
  double PCM_source_GetLengthBeats(PCM_source* self) {
    return self->GetLengthBeats();
  }
  int PCM_source_GetBitsPerSample(PCM_source* self) {
    return self->GetBitsPerSample();
  }
  double PCM_source_GetPreferredPosition(PCM_source* self) {
    return self->GetPreferredPosition();
  }
  int PCM_source_PropertiesWindow(PCM_source* self, HWND hwndParent) {
    return self->PropertiesWindow(hwndParent);
  }
  void PCM_source_GetSamples(PCM_source* self, PCM_source_transfer_t* block) {
    self->GetSamples(block);
  }
  void PCM_source_GetPeakInfo(PCM_source* self, PCM_source_peaktransfer_t* block) {
    self->GetPeakInfo(block);
  }
  void PCM_source_SaveState(PCM_source* self, ProjectStateContext* ctx) {
    self->SaveState(ctx);
  }
  int PCM_source_LoadState(PCM_source* self, const char* firstline, ProjectStateContext* ctx) {
    return self->LoadState(firstline, ctx);
  }
  void PCM_source_Peaks_Clear(PCM_source* self, bool deleteFile) {
    self->Peaks_Clear(deleteFile);
  }
  int PCM_source_PeaksBuild_Begin(PCM_source* self) {
    return self->PeaksBuild_Begin();
  }
  int PCM_source_PeaksBuild_Run(PCM_source* self) {
    return self->PeaksBuild_Run();
  }
  void PCM_source_PeaksBuild_Finish(PCM_source* self) {
    self->PeaksBuild_Finish();
  }
  int PCM_source_Extended(PCM_source* self, int call, void* parm1, void* parm2, void* parm3) {
    return self->Extended(call, parm1, parm2, parm3);
  }
}