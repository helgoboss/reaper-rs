#pragma once

#include "../lib/reaper/reaper_plugin.h"

namespace reaper_project_state_context {
  // This function is called from Rust and implemented in C++. It instantiates a C++ ProjectStateContext and returns
  // its address to Rust.
  extern "C" ProjectStateContext* create_cpp_to_rust_project_state_context(void* callback_target);

  // This function is called from Rust and implemented in C++. It destroys the given C++ ProjectStateContext object.
  extern "C" void delete_project_state_context(ProjectStateContext* context);

  // All of the following functions are called from C++ and implemented in Rust.
  // Because stable Rust doesn't support variadics, we "render" the string on C++ side.
  extern "C" void cpp_to_rust_ProjectStateContext_AddLine(void* callback_target, const char *line);
  extern "C" int cpp_to_rust_ProjectStateContext_GetLine(void* callback_target, char *buf, int buflen);
  extern "C" INT64 cpp_to_rust_ProjectStateContext_GetOutputSize(void* callback_target);
  extern "C" int cpp_to_rust_ProjectStateContext_GetTempFlag(void* callback_target);
  extern "C" void cpp_to_rust_ProjectStateContext_SetTempFlag(void* callback_target, int flag);
}