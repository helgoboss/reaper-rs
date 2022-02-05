#include "resample.hpp"

namespace reaper_resample {
  void delete_reaper_resample_interface(REAPER_Resample_Interface* resample_interface) {
    delete resample_interface;
  }

  void REAPER_Resample_Interface_SetRates(REAPER_Resample_Interface* self, double rate_in, double rate_out) {
    self->SetRates(rate_in, rate_out);
  }
  void REAPER_Resample_Interface_Reset(REAPER_Resample_Interface* self) {
    self->Reset();
  }
  double REAPER_Resample_Interface_GetCurrentLatency(REAPER_Resample_Interface* self) {
    return self->GetCurrentLatency();
  }
  int REAPER_Resample_Interface_ResamplePrepare(REAPER_Resample_Interface* self, int out_samples, int nch, ReaSample** inbuffer) {
    return self->ResamplePrepare(out_samples, nch, inbuffer);
  }
  int REAPER_Resample_Interface_ResampleOut(REAPER_Resample_Interface* self, ReaSample* out, int nsamples_in, int nsamples_out, int nch) {
    return self->ResampleOut(out, nsamples_in, nsamples_out, nch);
  }
  int REAPER_Resample_Interface_Extended(REAPER_Resample_Interface* self, int call, void* parm1, void* parm2, void* parm3) {
    return self->Extended(call, parm1, parm2, parm3);
  }
}