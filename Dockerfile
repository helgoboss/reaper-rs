FROM rust:1.69.0
RUN apt update && apt install -y llvm-dev libclang-dev clang