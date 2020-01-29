#include "control_surface.hpp"

namespace reaper_rs_control_surface {
  // This surface just delegates to the free functions implemented in Rust
  class ReaperRsControlSurface : public IReaperControlSurface {
  private:
    void* callback_target_;
  public:
    ReaperRsControlSurface(void* callback_target) : callback_target_(callback_target) {
    }

    virtual const char* GetTypeString() {
      return ::reaper_rs_control_surface::GetTypeString(this->callback_target_);
    }
    virtual const char* GetDescString() {
      return ::reaper_rs_control_surface::GetDescString(this->callback_target_);
    }
    virtual const char* GetConfigString() {
      return ::reaper_rs_control_surface::GetConfigString(this->callback_target_);
    }
    virtual void CloseNoReset() {
      ::reaper_rs_control_surface::CloseNoReset(this->callback_target_);
    }
    virtual void Run() {
      ::reaper_rs_control_surface::Run(this->callback_target_);
    }
    virtual void SetTrackListChange() {
      ::reaper_rs_control_surface::SetTrackListChange(this->callback_target_);
    }
    virtual void SetSurfaceVolume(MediaTrack* trackid, double volume) {
      ::reaper_rs_control_surface::SetSurfaceVolume(this->callback_target_, trackid, volume);
    }
    virtual void SetSurfacePan(MediaTrack* trackid, double pan) {
      ::reaper_rs_control_surface::SetSurfacePan(this->callback_target_, trackid, pan);
    }
    virtual void SetSurfaceMute(MediaTrack* trackid, bool mute) {
      ::reaper_rs_control_surface::SetSurfaceMute(this->callback_target_, trackid, mute);
    }
    virtual void SetSurfaceSelected(MediaTrack* trackid, bool selected) {
      ::reaper_rs_control_surface::SetSurfaceSelected(this->callback_target_, trackid, selected);
    }
    virtual void SetSurfaceSolo(MediaTrack* trackid, bool solo) {
      ::reaper_rs_control_surface::SetSurfaceSolo(this->callback_target_, trackid, solo);
    }
    virtual void SetSurfaceRecArm(MediaTrack* trackid, bool recarm) {
      ::reaper_rs_control_surface::SetSurfaceRecArm(this->callback_target_, trackid, recarm);
    }
    virtual void SetPlayState(bool play, bool pause, bool rec) {
      ::reaper_rs_control_surface::SetPlayState(this->callback_target_, play, pause, rec);
    }
    virtual void SetRepeatState(bool rep) {
      ::reaper_rs_control_surface::SetRepeatState(this->callback_target_, rep);
    }
    virtual void SetTrackTitle(MediaTrack* trackid, const char* title) {
      ::reaper_rs_control_surface::SetTrackTitle(this->callback_target_, trackid, title);
    }
    virtual bool GetTouchState(MediaTrack* trackid, int isPan) {
      return ::reaper_rs_control_surface::GetTouchState(this->callback_target_, trackid, isPan);
    }
    virtual void SetAutoMode(int mode) {
      ::reaper_rs_control_surface::SetAutoMode(this->callback_target_, mode);
    }
    virtual void ResetCachedVolPanStates() {
      ::reaper_rs_control_surface::ResetCachedVolPanStates(this->callback_target_);
    }
    virtual void OnTrackSelection(MediaTrack* trackid) {
      ::reaper_rs_control_surface::OnTrackSelection(this->callback_target_, trackid);
    }
    virtual bool IsKeyDown(int key) {
      return ::reaper_rs_control_surface::IsKeyDown(this->callback_target_, key);
    }
    virtual int Extended(int call, void* parm1, void* parm2, void* parm3) {
      return ::reaper_rs_control_surface::Extended(this->callback_target_, call, parm1, parm2, parm3);
    }
  };

  void* create_control_surface(void* callback_target) {
    static ReaperRsControlSurface CONTROL_SURFACE(callback_target);
    return (void*) &CONTROL_SURFACE;
  }
}