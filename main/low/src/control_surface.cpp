#include "control_surface.hpp"

namespace reaper_control_surface {
  // C++ -> Rust

  // This surface just delegates to the free functions implemented in Rust. See header file for an explanation.
  class CppToRustControlSurface : public IReaperControlSurface {
  private:
    // This pointer points to a Box in Rust which holds an IReaperControlSurface trait implementation.
    void* callback_target_;
  public:
    CppToRustControlSurface(void* callback_target) : callback_target_(callback_target) {
    }

    virtual const char* GetTypeString() {
      return ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_GetTypeString(this->callback_target_);
    }
    virtual const char* GetDescString() {
      return ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_GetDescString(this->callback_target_);
    }
    virtual const char* GetConfigString() {
      return ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_GetConfigString(this->callback_target_);
    }
    virtual void CloseNoReset() {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_CloseNoReset(this->callback_target_);
    }
    virtual void Run() {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_Run(this->callback_target_);
    }
    virtual void SetTrackListChange() {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetTrackListChange(this->callback_target_);
    }
    virtual void SetSurfaceVolume(MediaTrack* trackid, double volume) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetSurfaceVolume(this->callback_target_, trackid, volume);
    }
    virtual void SetSurfacePan(MediaTrack* trackid, double pan) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetSurfacePan(this->callback_target_, trackid, pan);
    }
    virtual void SetSurfaceMute(MediaTrack* trackid, bool mute) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetSurfaceMute(this->callback_target_, trackid, mute);
    }
    virtual void SetSurfaceSelected(MediaTrack* trackid, bool selected) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetSurfaceSelected(this->callback_target_, trackid, selected);
    }
    virtual void SetSurfaceSolo(MediaTrack* trackid, bool solo) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetSurfaceSolo(this->callback_target_, trackid, solo);
    }
    virtual void SetSurfaceRecArm(MediaTrack* trackid, bool recarm) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetSurfaceRecArm(this->callback_target_, trackid, recarm);
    }
    virtual void SetPlayState(bool play, bool pause, bool rec) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetPlayState(this->callback_target_, play, pause, rec);
    }
    virtual void SetRepeatState(bool rep) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetRepeatState(this->callback_target_, rep);
    }
    virtual void SetTrackTitle(MediaTrack* trackid, const char* title) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetTrackTitle(this->callback_target_, trackid, title);
    }
    virtual bool GetTouchState(MediaTrack* trackid, int isPan) {
      return ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_GetTouchState(this->callback_target_, trackid, isPan);
    }
    virtual void SetAutoMode(int mode) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_SetAutoMode(this->callback_target_, mode);
    }
    virtual void ResetCachedVolPanStates() {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_ResetCachedVolPanStates(this->callback_target_);
    }
    virtual void OnTrackSelection(MediaTrack* trackid) {
      ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_OnTrackSelection(this->callback_target_, trackid);
    }
    virtual bool IsKeyDown(int key) {
      return ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_IsKeyDown(this->callback_target_, key);
    }
    virtual int Extended(int call, void* parm1, void* parm2, void* parm3) {
      return ::reaper_control_surface::cpp_to_rust_IReaperControlSurface_Extended(this->callback_target_, call, parm1, parm2, parm3);
    }
  };

  IReaperControlSurface* create_cpp_to_rust_control_surface(void* callback_target) {
    return new CppToRustControlSurface(callback_target);
  }

  void delete_control_surface(IReaperControlSurface* surface) {
    delete surface;
  }
}