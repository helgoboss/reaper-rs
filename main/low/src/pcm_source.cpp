#include "pcm_source.hpp"

namespace reaper_pcm_source {
  // Rust -> C++
  double rust_to_cpp_PCM_source_GetLength(PCM_source* self) {
    return self->GetLength();
  }
  PCM_source* rust_to_cpp_PCM_source_Duplicate(PCM_source* self) {
    return self->Duplicate();
  }
  const char* rust_to_cpp_PCM_source_GetType(PCM_source* self) {
    return self->GetType();
  }
  const char* rust_to_cpp_PCM_source_GetFileName(PCM_source* self) {
    return self->GetFileName();
  }
  PCM_source* rust_to_cpp_PCM_source_GetSource(PCM_source* self) {
    return self->GetSource();
  }
  bool rust_to_cpp_PCM_source_IsAvailable(PCM_source* self) {
    return self->IsAvailable();
  }
  void rust_to_cpp_PCM_source_SetAvailable(PCM_source* self, bool avail) {
    self->SetAvailable(avail);
  }
  bool rust_to_cpp_PCM_source_SetFileName(PCM_source* self, const char* newfn) {
    return self->SetFileName(newfn);
  }
  void rust_to_cpp_PCM_source_SetSource(PCM_source* self, PCM_source* src) {
    self->SetSource(src);
  }
  int rust_to_cpp_PCM_source_GetNumChannels(PCM_source* self) {
    return self->GetNumChannels();
  }
  double rust_to_cpp_PCM_source_GetSampleRate(PCM_source* self) {
    return self->GetSampleRate();
  }
  double rust_to_cpp_PCM_source_GetLengthBeats(PCM_source* self) {
    return self->GetLengthBeats();
  }
  int rust_to_cpp_PCM_source_GetBitsPerSample(PCM_source* self) {
    return self->GetBitsPerSample();
  }
  double rust_to_cpp_PCM_source_GetPreferredPosition(PCM_source* self) {
    return self->GetPreferredPosition();
  }
  int rust_to_cpp_PCM_source_PropertiesWindow(PCM_source* self, HWND hwndParent) {
    return self->PropertiesWindow(hwndParent);
  }
  void rust_to_cpp_PCM_source_GetSamples(PCM_source* self, PCM_source_transfer_t* block) {
    self->GetSamples(block);
  }
  void rust_to_cpp_PCM_source_GetPeakInfo(PCM_source* self, PCM_source_peaktransfer_t* block) {
    self->GetPeakInfo(block);
  }
  void rust_to_cpp_PCM_source_SaveState(PCM_source* self, ProjectStateContext* ctx) {
    self->SaveState(ctx);
  }
  int rust_to_cpp_PCM_source_LoadState(PCM_source* self, const char* firstline, ProjectStateContext* ctx) {
    return self->LoadState(firstline, ctx);
  }
  void rust_to_cpp_PCM_source_Peaks_Clear(PCM_source* self, bool deleteFile) {
    self->Peaks_Clear(deleteFile);
  }
  int rust_to_cpp_PCM_source_PeaksBuild_Begin(PCM_source* self) {
    return self->PeaksBuild_Begin();
  }
  int rust_to_cpp_PCM_source_PeaksBuild_Run(PCM_source* self) {
    return self->PeaksBuild_Run();
  }
  void rust_to_cpp_PCM_source_PeaksBuild_Finish(PCM_source* self) {
    self->PeaksBuild_Finish();
  }
  int rust_to_cpp_PCM_source_Extended(PCM_source* self, int call, void* parm1, void* parm2, void* parm3) {
    return self->Extended(call, parm1, parm2, parm3);
  }

  // C++ -> Rust

  // This source just delegates to the free functions implemented in Rust. See header file for an explanation.
  class CppToRustPcmSource : public PCM_source {
  private:
    // This pointer points to a Box in Rust which holds a PCM_source trait implementation.
    void* callback_target_;
  public:
    CppToRustPcmSource(void* callback_target) : callback_target_(callback_target) {
    }

    virtual PCM_source* Duplicate() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_Duplicate(this->callback_target_);
    }
    virtual bool IsAvailable() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_IsAvailable(this->callback_target_);
    }
    virtual const char* GetType() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_GetType(this->callback_target_);
    }
    virtual bool SetFileName(const char* newfn) {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_SetFileName(this->callback_target_, newfn);
    }
    virtual int GetNumChannels() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_GetNumChannels(this->callback_target_);
    }
    virtual double GetSampleRate() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_GetSampleRate(this->callback_target_);
    }
    virtual double GetLength() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_GetLength(this->callback_target_);
    }
    virtual int PropertiesWindow(HWND hwndParent) {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_PropertiesWindow(this->callback_target_, hwndParent);
    }
    virtual void GetSamples(PCM_source_transfer_t* block) {
      ::reaper_pcm_source::cpp_to_rust_PCM_source_GetSamples(this->callback_target_, block);
    }
    virtual void GetPeakInfo(PCM_source_peaktransfer_t* block) {
      ::reaper_pcm_source::cpp_to_rust_PCM_source_GetPeakInfo(this->callback_target_, block);
    }
    virtual void SaveState(ProjectStateContext* ctx) {
      ::reaper_pcm_source::cpp_to_rust_PCM_source_SaveState(this->callback_target_, ctx);
    }
    virtual int LoadState(const char* firstline, ProjectStateContext* ctx) {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_LoadState(this->callback_target_, firstline, ctx);
    }
    virtual void Peaks_Clear(bool deleteFile) {
      ::reaper_pcm_source::cpp_to_rust_PCM_source_Peaks_Clear(this->callback_target_, deleteFile);
    }
    virtual int PeaksBuild_Begin() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_PeaksBuild_Begin(this->callback_target_);
    }
    virtual int PeaksBuild_Run() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_PeaksBuild_Run(this->callback_target_);
    }
    virtual void PeaksBuild_Finish() {
      ::reaper_pcm_source::cpp_to_rust_PCM_source_PeaksBuild_Finish(this->callback_target_);
    }
    virtual void SetAvailable(bool avail) {
      ::reaper_pcm_source::cpp_to_rust_PCM_source_SetAvailable(this->callback_target_, avail);
    }
    virtual const char* GetFileName() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_GetFileName(this->callback_target_);
    }
    virtual PCM_source* GetSource() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_GetSource(this->callback_target_);
    }
    virtual void SetSource(PCM_source* src) {
      ::reaper_pcm_source::cpp_to_rust_PCM_source_SetSource(this->callback_target_, src);
    }
    virtual double GetLengthBeats() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_GetLengthBeats(this->callback_target_);
    }
    virtual int GetBitsPerSample() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_GetBitsPerSample(this->callback_target_);
    }
    virtual double GetPreferredPosition() {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_GetPreferredPosition(this->callback_target_);
    }
    virtual int Extended(int call, void* parm1, void* parm2, void* parm3) {
      return ::reaper_pcm_source::cpp_to_rust_PCM_source_Extended(this->callback_target_, call, parm1, parm2, parm3);
    }
  };

  PCM_source* create_cpp_to_rust_pcm_source(void* callback_target) {
    return new CppToRustPcmSource(callback_target);
  }

  void delete_pcm_source(PCM_source* source) {
    delete source;
  }
}