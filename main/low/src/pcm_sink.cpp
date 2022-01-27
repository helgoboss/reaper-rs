#include "pcm_sink.hpp"

namespace reaper_pcm_sink {
  // Rust -> C++
  void rust_to_cpp_PCM_sink_GetOutputInfoString(PCM_sink* self, char* buf, int buflen) {
    self->GetOutputInfoString(buf, buflen);
  }
  double rust_to_cpp_PCM_sink_GetStartTime(PCM_sink* self) {
    return self->GetStartTime();
  }
  void rust_to_cpp_PCM_sink_SetStartTime(PCM_sink* self, double st) {
    self->SetStartTime(st);
  }
  const char* rust_to_cpp_PCM_sink_GetFileName(PCM_sink* self) {
    return self->GetFileName();
  }
  int rust_to_cpp_PCM_sink_GetNumChannels(PCM_sink* self) {
    return self->GetNumChannels();
  }
  double rust_to_cpp_PCM_sink_GetLength(PCM_sink* self) {
    return self->GetLength();
  }
  long long int rust_to_cpp_PCM_sink_GetFileSize(PCM_sink* self) {
    return self->GetFileSize();
  }
  void rust_to_cpp_PCM_sink_WriteMIDI(PCM_sink* self,
      MIDI_eventlist* events,
      int len,
      double samplerate) {
    self->WriteMIDI(events, len, samplerate);
  }
  void rust_to_cpp_PCM_sink_WriteDoubles(PCM_sink* self,
      ReaSample** samples,
      int len,
      int nch,
      int offset,
      int spacing) {
    self->WriteDoubles(samples, len, nch, offset, spacing);
  }
  bool rust_to_cpp_PCM_sink_WantMIDI(PCM_sink* self) {
    return self->WantMIDI();
  }
  int rust_to_cpp_PCM_sink_GetLastSecondPeaks(PCM_sink* self, int sz, ReaSample* buf) {
    return self->GetLastSecondPeaks(sz, buf);
  }
  void rust_to_cpp_PCM_sink_GetPeakInfo(PCM_sink* self, PCM_source_peaktransfer_t* block) {
    self->GetPeakInfo(block);
  }
  int rust_to_cpp_PCM_sink_Extended(PCM_sink* self, int call, void* parm1, void* parm2, void* parm3) {
    return self->Extended(call, parm1, parm2, parm3);
  }

  // C++ -> Rust

  // This source just delegates to the free functions implemented in Rust. See header file for an explanation.
  class CppToRustPcmSink : public PCM_sink {
  private:
    // This pointer points to a Box in Rust which holds a PCM_sink trait implementation.
    void* callback_target_;
  public:
    CppToRustPcmSink(void* callback_target) : callback_target_(callback_target) {
    }
    virtual void GetOutputInfoString(char* buf, int buflen) {
      return ::reaper_pcm_sink::cpp_to_rust_PCM_sink_GetOutputInfoString(this->callback_target_, buf, buflen);
    }
    virtual const char* GetFileName() {
      return ::reaper_pcm_sink::cpp_to_rust_PCM_sink_GetFileName(this->callback_target_);
    }
    virtual int GetNumChannels() {
      return ::reaper_pcm_sink::cpp_to_rust_PCM_sink_GetNumChannels(this->callback_target_);
    }
    virtual double GetLength() {
      return ::reaper_pcm_sink::cpp_to_rust_PCM_sink_GetLength(this->callback_target_);
    }
    virtual long long int GetFileSize() {
      return ::reaper_pcm_sink::cpp_to_rust_PCM_sink_GetFileSize(this->callback_target_);
    }
    virtual void WriteMIDI(MIDI_eventlist* events, int len, double samplerate) {
      ::reaper_pcm_sink::cpp_to_rust_PCM_sink_WriteMIDI(this->callback_target_, events, len, samplerate);
    }
    virtual void WriteDoubles(ReaSample** samples, int len, int nch, int offset, int spacing) {
      ::reaper_pcm_sink::cpp_to_rust_PCM_sink_WriteDoubles(this->callback_target_, samples, len, nch, offset, spacing);
    }
    virtual double GetStartTime() {
      return ::reaper_pcm_sink::cpp_to_rust_PCM_sink_GetStartTime(this->callback_target_);
    }
    virtual void SetStartTime(double st) {
      ::reaper_pcm_sink::cpp_to_rust_PCM_sink_SetStartTime(this->callback_target_, st);
    }
    virtual bool WantMIDI() {
      return ::reaper_pcm_sink::cpp_to_rust_PCM_sink_WantMIDI(this->callback_target_);
    }
    virtual int GetLastSecondPeaks(int sz, ReaSample* buf) {
      return ::reaper_pcm_sink::cpp_to_rust_PCM_sink_GetLastSecondPeaks(this->callback_target_, sz, buf);
    }
    virtual void GetPeakInfo(PCM_source_peaktransfer_t* block) {
      ::reaper_pcm_sink::cpp_to_rust_PCM_sink_GetPeakInfo(this->callback_target_, block);
    }
    virtual int Extended(int call, void* parm1, void* parm2, void* parm3) {
      return ::reaper_pcm_sink::cpp_to_rust_PCM_sink_Extended(this->callback_target_, call, parm1, parm2, parm3);
    }
  };


  PCM_sink* create_cpp_to_rust_pcm_sink(void* callback_target) {
    return new CppToRustPcmSink(callback_target);
  }

  void delete_pcm_sink(PCM_sink* sink) {
    delete sink;
  }
}