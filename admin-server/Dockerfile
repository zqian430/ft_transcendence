# ==============================================================================
# STEP 1: Build
# ==============================================================================
FROM rust:1.88-alpine AS builder

# musl-dev and openssl-dev are required to compile SQL/HTTPS clients in Rust
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /usr/src/app

# --- Dependency caching ---
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# --- Build the actual source code ---
COPY src ./src


RUN touch src/main.rs && cargo build --release

# ==============================================================================
# STEP 2: Runtime image
# ==============================================================================
FROM alpine:3.20

# curl for the healthcheck, ca-certificates for calling external APIs (42 OAuth, external LLM)
RUN apk update && apk add --no-cache curl ca-certificates libgcc

RUN adduser -D -u 10001 adminuser

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/admin-server /app/server

RUN chown -R adminuser:adminuser /app
USER adminuser

EXPOSE ${ADMIN_SERVER_PORT}

CMD ["/app/server"]