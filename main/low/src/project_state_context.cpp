#include "project_state_context.hpp"

namespace reaper_project_state_context {
  // Rust -> C++
  void rust_to_cpp_ProjectStateContext_AddLine(ProjectStateContext* self, const char* line) {
    // TODO-high This can't work. Wait for variadics support in stable Rust.
//    self->AddLine(line);
  }
  int rust_to_cpp_ProjectStateContext_GetLine(ProjectStateContext* self, char* buf, int buflen) {
    return self->GetLine(buf, buflen);
  }
  INT64 rust_to_cpp_ProjectStateContext_GetOutputSize(ProjectStateContext* self) {
    return self->GetOutputSize();
  }
  int rust_to_cpp_ProjectStateContext_GetTempFlag(ProjectStateContext* self) {
    return self->GetTempFlag();
  }
  void rust_to_cpp_ProjectStateContext_SetTempFlag(ProjectStateContext* self, int flag) {
    self->SetTempFlag(flag);
  }

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
      ::reaper_project_state_context::cpp_to_rust_ProjectStateContext_AddLine(this->callback_target_, fmt);
    }
#else
    virtual void AddLine(const char* fmt, ...) {
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