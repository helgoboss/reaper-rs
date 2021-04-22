#pragma once

#include "../lib/reaper/reaper_plugin.h"

// Most functions in this namespace are called from C++ and implemented in Rust. They are callbacks invoked by REAPER.
// This glue code is necessary because Rust can't implement pure virtual functions directly.
namespace reaper_control_surface {
  // This function is called from Rust and implemented in C++. It instantiates a C++ IReaperControlSurface and returns
  // its address to Rust.
  extern "C" IReaperControlSurface* create_cpp_to_rust_control_surface(void* callback_target);

  // This function is called from Rust and implemented in C++. It destroys the given C++ IReaperControlSurface object.
  extern "C" void delete_control_surface(IReaperControlSurface* surface);

  // All of the following functions are called from C++ and implemented in Rust.
  extern "C" const char* cpp_to_rust_IReaperControlSurface_GetTypeString(void* callback_target);
  extern "C" const char* cpp_to_rust_IReaperControlSurface_GetDescString(void* callback_target);
  extern "C" const char* cpp_to_rust_IReaperControlSurface_GetConfigString(void* callback_target);
  extern "C" void cpp_to_rust_IReaperControlSurface_CloseNoReset(void* callback_target);
  extern "C" void cpp_to_rust_IReaperControlSurface_Run(void* callback_target);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetTrackListChange(void* callback_target);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetSurfaceVolume(void* callback_target, MediaTrack* trackid, double volume);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetSurfacePan(void* callback_target, MediaTrack* trackid, double pan);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetSurfaceMute(void* callback_target, MediaTrack* trackid, bool mute);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetSurfaceSelected(void* callback_target, MediaTrack* trackid, bool selected);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetSurfaceSolo(void* callback_target, MediaTrack* trackid, bool solo);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetSurfaceRecArm(void* callback_target, MediaTrack* trackid, bool recarm);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetPlayState(void* callback_target, bool play, bool pause, bool rec);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetRepeatState(void* callback_target, bool rep);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetTrackTitle(void* callback_target, MediaTrack* trackid, const char* title);
  extern "C" bool cpp_to_rust_IReaperControlSurface_GetTouchState(void* callback_target, MediaTrack* trackid, int isPan);
  extern "C" void cpp_to_rust_IReaperControlSurface_SetAutoMode(void* callback_target, int mode);
  extern "C" void cpp_to_rust_IReaperControlSurface_ResetCachedVolPanStates(void* callback_target);
  extern "C" void cpp_to_rust_IReaperControlSurface_OnTrackSelection(void* callback_target, MediaTrack* trackid);
  extern "C" bool cpp_to_rust_IReaperControlSurface_IsKeyDown(void* callback_target, int key);
  extern "C" int cpp_to_rust_IReaperControlSurface_Extended(void* callback_target, int call, void* parm1, void* parm2, void* parm3);
}