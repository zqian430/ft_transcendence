# syntax=docker/dockerfile:1
# ==============================================================================
# STEP 1: Build
# ==============================================================================
# Works for any Cargo layout (single crate or workspace, any depth of members):
# the whole context is copied and built, then the binary BIN_NAME is shipped.
FROM rust:1-alpine AS builder

# musl-dev and openssl-dev are required to compile SQL/HTTPS clients in Rust
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /usr/src/app

# Name of the binary to ship (the 'name' of the package in Cargo.toml).
ARG BIN_NAME="admin-server"

COPY . .

# --- Dependency caching ---
# BuildKit cache mounts keep the cargo registry and the target/ directory between
# builds, so only changed crates are recompiled. No dummy main.rs needed, so it
# does not depend on where the sources live.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/app/target \
    set -eu; \
    cargo build --release --locked; \
    bin="target/release/$BIN_NAME"; \
    if [ ! -f "$bin" ]; then \
        echo "ERROR: binary '$BIN_NAME' not found in target/release" >&2; \
        exit 1; \
    fi; \
    cp "$bin" /usr/local/bin/app

# ==============================================================================
# STEP 2: Runtime image
# ==============================================================================
FROM alpine:3.20

# curl for the healthcheck, ca-certificates for calling external APIs (42 OAuth, external LLM)
RUN apk add --no-cache curl ca-certificates libgcc

# Security: create a non-root user to run the server
RUN adduser -D -u 10001 adminuser

WORKDIR /app

# Copy only the compiled executable from step 1, owned by our restricted user
COPY --from=builder --chown=adminuser:adminuser /usr/local/bin/app /app/server

USER adminuser

# Provided by docker-compose (build.args, from .env)
ARG ADMIN_SERVER_PORT
EXPOSE ${ADMIN_SERVER_PORT}

CMD ["/app/server"]
