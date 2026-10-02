# Multi-stage build: compile with the full toolchain, ship a small runtime
# image containing one statically-linked-ish binary and no build tools.

FROM rust:1.95-slim-bookworm AS builder
WORKDIR /build

# cmake, clang, libclang and g++ build BoringSSL, which `wreq` uses to
# present the same TLS handshake as Chrome to the stores' bot protection.
# git is needed too: boring-sys2's build script applies its patches with it.
RUN apt-get update -qq \
 && apt-get install -y --no-install-recommends pkg-config cmake clang libclang-dev g++ perl make git \
 && rm -rf /var/lib/apt/lists/*

# Dependencies first, in their own layer, so editing source does not re-fetch
# and rebuild 300 crates on every image build.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
 && echo 'fn main() {}' > src/main.rs \
 && echo '' > src/lib.rs \
 && cargo build --release --locked \
 && rm -rf src

# `sqlx::migrate!` reads these at compile time, so they must be present
# before the real build.
COPY migrations ./migrations
COPY src ./src

# Touch the entrypoints so cargo rebuilds them rather than reusing the stubs.
RUN touch src/main.rs src/lib.rs \
 && cargo build --release --locked

FROM debian:bookworm-slim AS runtime

# ca-certificates: the process fetches Clerk's JWKS over HTTPS. (The store
# client carries its own root store and needs nothing extra.)
RUN apt-get update -qq \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --create-home --shell /usr/sbin/nologin grocery \
 && mkdir -p /var/log/grocery \
 && chown grocery:grocery /var/log/grocery

COPY --from=builder /build/target/release/grocery-backend /usr/local/bin/grocery-backend

USER grocery
EXPOSE 8000

# Migrations are embedded in the binary, so the container needs nothing on
# disk and cannot run a schema its code was not built against.
CMD ["grocery-backend"]
