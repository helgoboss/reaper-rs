FROM rust:1.98.1
RUN apt update && apt install -y llvm-dev libclang-dev clang