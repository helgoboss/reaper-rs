#include "surface.h"

namespace reaper_rs_surface {
  // This surface just delegates to the free functions implemented in Rust
  class ReaperRsControlSurface : public IReaperControlSurface {
  public:
    virtual const char* GetTypeString() {
      return ::GetTypeString();
    }
    virtual const char* GetDescString() {
      return ::GetDescString();
    }
    virtual const char* GetConfigString() {
      return ::GetConfigString();
    }
    virtual void CloseNoReset() {
      ::CloseNoReset();
    }
    virtual void Run() {
      ::Run();
    }
    virtual void SetTrackListChange() {
      ::SetTrackListChange();
    }
    virtual void SetSurfaceVolume(MediaTrack* trackid, double volume) {
      ::SetSurfaceVolume(trackid, volume);
    }
    virtual void SetSurfacePan(MediaTrack* trackid, double pan) {
      ::SetSurfacePan(trackid, pan);
    }
    virtual void SetSurfaceMute(MediaTrack* trackid, bool mute) {
      ::SetSurfaceMute(trackid, mute);
    }
    virtual void SetSurfaceSelected(MediaTrack* trackid, bool selected) {
      ::SetSurfaceSelected(trackid, selected);
    }
    virtual void SetSurfaceSolo(MediaTrack* trackid, bool solo) {
      ::SetSurfaceSolo(trackid, solo);
    }
    virtual void SetSurfaceRecArm(MediaTrack* trackid, bool recarm) {
      ::SetSurfaceRecArm(trackid, recarm);
    }
    virtual void SetPlayState(bool play, bool pause, bool rec) {
      ::SetPlayState(play, pause, rec);
    }
    virtual void SetRepeatState(bool rep) {
      ::SetRepeatState(rep);
    }
    virtual void SetTrackTitle(MediaTrack* trackid, const char* title) {
      ::SetTrackTitle(trackid, title);
    }
    virtual bool GetTouchState(MediaTrack* trackid, int isPan) {
      return ::GetTouchState(trackid, isPan);
    }
    virtual void SetAutoMode(int mode) {
      ::SetAutoMode(mode);
    }
    virtual void ResetCachedVolPanStates() {
      ::ResetCachedVolPanStates();
    }
    virtual void OnTrackSelection(MediaTrack* trackid) {
      ::OnTrackSelection(trackid);
    }
    virtual bool IsKeyDown(int key) {
      return ::IsKeyDown(key);
    }
    virtual int Extended(int call, void* parm1, void* parm2, void* parm3) {
      return ::Extended(call, parm1, parm2, parm3);
    }
  };

  void* get_surface() {
    static ReaperRsControlSurface SURFACE;
    return (void*) &SURFACE;
  }
}