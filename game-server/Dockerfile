
# STEP 1: Build

FROM rust:1-alpine AS builder

# Install the system tools required to compile Rust crates on Alpine
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /usr/src/app

ARG BIN_NAME="server"

COPY . .

# cache cargo dependencies with Docker native BuildKit tool. cached in host pc
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

# STEP 2: Runtime image

FROM alpine:3.20

# Install curl (required for the Docker healthcheck) and certificates
RUN apk add --no-cache curl ca-certificates libgcc

# Security: create a non-root user to run the server
RUN adduser -D -u 10001 gameuser

WORKDIR /app

# Copy only the compiled executable from step 1, owned by our restricted user
COPY --from=builder --chown=gameuser:gameuser /usr/local/bin/app /app/server

USER gameuser

ARG GAME_SERVER_PORT
EXPOSE ${GAME_SERVER_PORT}

CMD ["/app/server"]
