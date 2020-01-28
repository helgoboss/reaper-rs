#pragma once

#include "../../lib/reaper/reaper_plugin.h"

namespace reaper_rs_surface {
  // This is implemented in C++ and called from Rust
  extern "C" void* get_surface();

  // These is implemented in Rust and called from C++
  extern "C" const char* GetTypeString();
  extern "C" const char* GetDescString();
  extern "C" const char* GetConfigString();
  extern "C" void CloseNoReset();
  extern "C" void Run();
  extern "C" void SetTrackListChange();
  extern "C" void SetSurfaceVolume(MediaTrack* trackid, double volume);
  extern "C" void SetSurfacePan(MediaTrack* trackid, double pan);
  extern "C" void SetSurfaceMute(MediaTrack* trackid, bool mute);
  extern "C" void SetSurfaceSelected(MediaTrack* trackid, bool selected);
  extern "C" void SetSurfaceSolo(MediaTrack* trackid, bool solo);
  extern "C" void SetSurfaceRecArm(MediaTrack* trackid, bool recarm);
  extern "C" void SetPlayState(bool play, bool pause, bool rec);
  extern "C" void SetRepeatState(bool rep);
  extern "C" void SetTrackTitle(MediaTrack* trackid, const char* title);
  extern "C" bool GetTouchState(MediaTrack* trackid, int isPan);
  extern "C" void SetAutoMode(int mode);
  extern "C" void ResetCachedVolPanStates();
  extern "C" void OnTrackSelection(MediaTrack* trackid);
  extern "C" bool IsKeyDown(int key);
  extern "C" int Extended(int call, void* parm1, void* parm2, void* parm3);
}