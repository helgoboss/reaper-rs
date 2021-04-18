#include "pcm_source.hpp"

namespace reaper_pcm_source {
  double PCM_source_GetLength(PCM_source* self) {
    return self->GetLength();
  }
}