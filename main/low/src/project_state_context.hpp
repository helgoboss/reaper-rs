#pragma once

#include "../lib/reaper/reaper_plugin.h"

namespace reaper_project_state_context {
  // This function is called from Rust and implemented in C++. It instantiates a C++ ProjectStateContext and returns
  // its address to Rust.
  extern "C" ProjectStateContext* create_cpp_to_rust_project_state_context(void* callback_target);

  // This function is called from Rust and implemented in C++. It destroys the given C++ ProjectStateContext object.
  extern "C" void delete_project_state_context(ProjectStateContext* context);

  // All of the following functions are called from C++ and implemented in Rust.
  // TODO-high This can't work. Wait for variadics support in stable Rust.
  extern "C" void cpp_to_rust_ProjectStateContext_AddLine(void* callback_target, const char *line);
  extern "C" int cpp_to_rust_ProjectStateContext_GetLine(void* callback_target, char *buf, int buflen);
  extern "C" INT64 cpp_to_rust_ProjectStateContext_GetOutputSize(void* callback_target);
  extern "C" int cpp_to_rust_ProjectStateContext_GetTempFlag(void* callback_target);
  extern "C" void cpp_to_rust_ProjectStateContext_SetTempFlag(void* callback_target, int flag);
  
  // All the following functions are called from Rust and implemented in C++. The implementation simply delegates
  // to the respective method of the `self` object. This glue code is necessary because Rust can't call  C++ pure 
  // virtual functions directly.
  // TODO-high This can't work. Wait for variadics support in stable Rust.
  extern "C" void rust_to_cpp_ProjectStateContext_AddLine(ProjectStateContext* self, const char *line);
  extern "C" int rust_to_cpp_ProjectStateContext_GetLine(ProjectStateContext* self, char *buf, int buflen);
  extern "C" INT64 rust_to_cpp_ProjectStateContext_GetOutputSize(ProjectStateContext* self);
  extern "C" int rust_to_cpp_ProjectStateContext_GetTempFlag(ProjectStateContext* self);
  extern "C" void rust_to_cpp_ProjectStateContext_SetTempFlag(ProjectStateContext* self, int flag);
}