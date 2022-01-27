#include "project_state_context.hpp"

namespace reaper_project_state_context {
  // C++ -> Rust

  // This source just delegates to the free functions implemented in Rust. See header file for an explanation.
  class CppToRustProjectStateContext : public ProjectStateContext {
  private:
    // This pointer points to a Box in Rust which holds a ProjectStateContext trait implementation.
    void* callback_target_;
  public:
    CppToRustProjectStateContext(void* callback_target) : callback_target_(callback_target) {
    }

#ifdef __GNUC__
    virtual void  __attribute__ ((format (printf,2,3))) AddLine(const char *fmt, ...) {
      // TODO-high Render fmt and variadics to temporary string.
      ::reaper_project_state_context::cpp_to_rust_ProjectStateContext_AddLine(this->callback_target_, fmt);
    }
#else
    virtual void AddLine(const char* fmt, ...) {
      // TODO-high Render fmt and variadics to temporary string.
      ::reaper_project_state_context::cpp_to_rust_ProjectStateContext_AddLine(this->callback_target_, fmt);
    }
#endif
    virtual int GetLine(char* buf, int buflen) {
      return ::reaper_project_state_context::cpp_to_rust_ProjectStateContext_GetLine(this->callback_target_, buf, buflen);
    }
    virtual long long int GetOutputSize() {
      return ::reaper_project_state_context::cpp_to_rust_ProjectStateContext_GetOutputSize(this->callback_target_);
    }
    virtual int GetTempFlag() {
      return ::reaper_project_state_context::cpp_to_rust_ProjectStateContext_GetTempFlag(this->callback_target_);
    }
    virtual void SetTempFlag(int flag) {
      ::reaper_project_state_context::cpp_to_rust_ProjectStateContext_SetTempFlag(this->callback_target_, flag);
    }
  };

  ProjectStateContext* create_cpp_to_rust_project_state_context(void* callback_target) {
    return new CppToRustProjectStateContext(callback_target);
  }

  void delete_project_state_context(ProjectStateContext* context) {
    delete context;
  }
}