#pragma once

#include "../lib/reaper/reaper_plugin.h"

// Most functions in this namespace are called from C++ and implemented in Rust. They are callbacks invoked by REAPER.
// This glue code is necessary because Rust can't implement pure virtual functions directly.
namespace reaper_control_surface {
  // This function is called from Rust and implemented in C++. It instantiates a C++ IReaperControlSurface and returns
  // its address to Rust.
  extern "C" IReaperControlSurface* add_control_surface(void* callback_target);

  // This function is called from Rust and implemented in C++. It destroys the given C++ IReaperControlSurface object.
  extern "C" void remove_control_surface(IReaperControlSurface* surface);

  // All of the following functions are called from C++ and implemented in Rust.
  extern "C" const char* GetTypeString(void* callback_target);
  extern "C" const char* GetDescString(void* callback_target);
  extern "C" const char* GetConfigString(void* callback_target);
  extern "C" void CloseNoReset(void* callback_target);
  extern "C" void Run(void* callback_target);
  extern "C" void SetTrackListChange(void* callback_target);
  extern "C" void SetSurfaceVolume(void* callback_target, MediaTrack* trackid, double volume);
  extern "C" void SetSurfacePan(void* callback_target, MediaTrack* trackid, double pan);
  extern "C" void SetSurfaceMute(void* callback_target, MediaTrack* trackid, bool mute);
  extern "C" void SetSurfaceSelected(void* callback_target, MediaTrack* trackid, bool selected);
  extern "C" void SetSurfaceSolo(void* callback_target, MediaTrack* trackid, bool solo);
  extern "C" void SetSurfaceRecArm(void* callback_target, MediaTrack* trackid, bool recarm);
  extern "C" void SetPlayState(void* callback_target, bool play, bool pause, bool rec);
  extern "C" void SetRepeatState(void* callback_target, bool rep);
  extern "C" void SetTrackTitle(void* callback_target, MediaTrack* trackid, const char* title);
  extern "C" bool GetTouchState(void* callback_target, MediaTrack* trackid, int isPan);
  extern "C" void SetAutoMode(void* callback_target, int mode);
  extern "C" void ResetCachedVolPanStates(void* callback_target);
  extern "C" void OnTrackSelection(void* callback_target, MediaTrack* trackid);
  extern "C" bool IsKeyDown(void* callback_target, int key);
  extern "C" int Extended(void* callback_target, int call, void* parm1, void* parm2, void* parm3);
}