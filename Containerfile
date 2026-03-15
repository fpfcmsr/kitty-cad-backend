# Multi-stage build for kitty-cad-backend
# Stage 1: Build
FROM registry.fedoraproject.org/fedora:43 AS builder

# Install build dependencies
# opencascade-rs bundles its own OCCT via occt-sys, but needs cmake for build scripts
# clang is needed for cxx-build FFI generation
RUN dnf install -y \
    gcc gcc-c++ make cmake \
    clang clang-devel \
    rust cargo \
    pkg-config \
    fontconfig-devel \
    && dnf clean all

WORKDIR /build

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.toml
COPY crates/protocol/Cargo.toml crates/protocol/Cargo.toml
COPY crates/engine/Cargo.toml crates/engine/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml

# Create dummy source files for dependency build
RUN mkdir -p crates/protocol/src crates/engine/src crates/server/src && \
    echo "pub mod modeling_cmd; pub mod responses; pub mod ws_messages;" > crates/protocol/src/lib.rs && \
    touch crates/protocol/src/modeling_cmd.rs crates/protocol/src/responses.rs crates/protocol/src/ws_messages.rs && \
    echo "" > crates/engine/src/lib.rs && \
    echo "fn main() {}" > crates/server/src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release 2>/dev/null || true

# Copy actual source
COPY crates/ crates/

# Touch source files to invalidate cache for our code only
RUN find crates -name "*.rs" -exec touch {} +

# Build the actual project
RUN cargo build --release

# Stage 2: Runtime
# occt-sys statically links OCCT, so we only need basic runtime libs
FROM registry.fedoraproject.org/fedora:43

RUN dnf install -y \
    fontconfig \
    && dnf clean all

COPY --from=builder /build/target/release/server /usr/local/bin/kitty-cad-backend

ENV PORT=3001
EXPOSE 3001

CMD ["kitty-cad-backend"]
