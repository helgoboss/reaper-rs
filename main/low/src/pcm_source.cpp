#include "pcm_source.hpp"

namespace reaper_pcm_source {
  // Rust -> C++
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

  // C++ -> Rust

  // This source just delegates to the free functions implemented in Rust. See header file for an explanation.
  class ReaperRsPcmSource : public PCM_source {
  private:
    // This pointer points to a Box in Rust which holds a PCM_source trait implementation.
    void* callback_target_;
  public:
    ReaperRsPcmSource(void* callback_target) : callback_target_(callback_target) {
    }

    virtual PCM_source* Duplicate() {
      return ::reaper_pcm_source::Duplicate(this->callback_target_);
    }
    virtual bool IsAvailable() {
      return ::reaper_pcm_source::IsAvailable(this->callback_target_);
    }
    virtual const char* GetType() {
      return ::reaper_pcm_source::GetType(this->callback_target_);
    }
    virtual bool SetFileName(const char* newfn) {
      return ::reaper_pcm_source::SetFileName(this->callback_target_, newfn);
    }
    virtual int GetNumChannels() {
      return ::reaper_pcm_source::GetNumChannels(this->callback_target_);
    }
    virtual double GetSampleRate() {
      return ::reaper_pcm_source::GetSampleRate(this->callback_target_);
    }
    virtual double GetLength() {
      return ::reaper_pcm_source::GetLength(this->callback_target_);
    }
    virtual int PropertiesWindow(HWND hwndParent) {
      return ::reaper_pcm_source::PropertiesWindow(this->callback_target_, hwndParent);
    }
    virtual void GetSamples(PCM_source_transfer_t* block) {
      ::reaper_pcm_source::GetSamples(this->callback_target_, block);
    }
    virtual void GetPeakInfo(PCM_source_peaktransfer_t* block) {
      ::reaper_pcm_source::GetPeakInfo(this->callback_target_, block);
    }
    virtual void SaveState(ProjectStateContext* ctx) {
      ::reaper_pcm_source::SaveState(this->callback_target_, ctx);
    }
    virtual int LoadState(const char* firstline, ProjectStateContext* ctx) {
      return ::reaper_pcm_source::LoadState(this->callback_target_, firstline, ctx);
    }
    virtual void Peaks_Clear(bool deleteFile) {
      ::reaper_pcm_source::Peaks_Clear(this->callback_target_, deleteFile);
    }
    virtual int PeaksBuild_Begin() {
      return ::reaper_pcm_source::PeaksBuild_Begin(this->callback_target_);
    }
    virtual int PeaksBuild_Run() {
      return ::reaper_pcm_source::PeaksBuild_Run(this->callback_target_);
    }
    virtual void PeaksBuild_Finish() {
      ::reaper_pcm_source::PeaksBuild_Finish(this->callback_target_);
    }
    virtual void SetAvailable(bool avail) {
      ::reaper_pcm_source::SetAvailable(this->callback_target_, avail);
    }
    virtual const char* GetFileName() {
      return ::reaper_pcm_source::GetFileName(this->callback_target_);
    }
    virtual PCM_source* GetSource() {
      return ::reaper_pcm_source::GetSource(this->callback_target_);
    }
    virtual void SetSource(PCM_source* src) {
      ::reaper_pcm_source::SetSource(this->callback_target_, src);
    }
    virtual double GetLengthBeats() {
      return ::reaper_pcm_source::GetLengthBeats(this->callback_target_);
    }
    virtual int GetBitsPerSample() {
      return ::reaper_pcm_source::GetBitsPerSample(this->callback_target_);
    }
    virtual double GetPreferredPosition() {
      return ::reaper_pcm_source::GetPreferredPosition(this->callback_target_);
    }
    virtual int Extended(int call, void* parm1, void* parm2, void* parm3) {
      return ::reaper_pcm_source::rust_PCM_source_Extended(this->callback_target_, call, parm1, parm2, parm3);
    }
  };

  PCM_source* add_pcm_source(void* callback_target) {
    return new ReaperRsPcmSource(callback_target);
  }

  void remove_pcm_source(PCM_source* source) {
    delete source;
  }
}