FROM rustlang/rust:nightly
RUN apt update && apt install -y llvm-dev libclang-dev clang