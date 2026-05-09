# =============================================================================
# Stage 0 — cargo-chef installer
#
# A dedicated stage so the cargo-chef binary itself is cached independently
# from both the dependency cook and the final build.
# =============================================================================
FROM rust:1.95-slim-bookworm AS chef

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef --locked

WORKDIR /app

# =============================================================================
# Stage 1 — dependency planner
#
# Copies the full workspace and produces recipe.json, which is a fingerprint
# of all Cargo dependencies.  Only changes to Cargo.toml / Cargo.lock will
# invalidate the cache layers that follow.
# =============================================================================
FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 2 — dependency cache + binary compilation
#
# 1. Cook (download + compile) only dependencies using the recipe.
#    This layer is fully cached as long as the recipe does not change.
# 2. Copy the real source and compile the two release binaries.
# =============================================================================
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release --locked -p api \
 && cargo build --release --locked -p ingestion

# =============================================================================
# Stage 3 — api runtime
#
# Minimal Debian image.  Only the compiled binary and CA certificates are
# present.  The service runs as an unprivileged system user.
# =============================================================================
FROM debian:bookworm-slim AS api

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app \
    && useradd  --system --gid app --no-create-home app

COPY --from=builder --chown=app:app /app/target/release/api /usr/local/bin/api

USER app

EXPOSE 8080

# Docker-native health check (also works with plain `docker run`).
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -sf http://localhost:8080/currencies || exit 1

ENTRYPOINT ["/usr/local/bin/api"]

# =============================================================================
# Stage 4 — ingestion runtime
#
# CA certificates are required for outbound HTTPS calls to the rate provider.
# =============================================================================
FROM debian:bookworm-slim AS ingestion

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app \
    && useradd  --system --gid app --no-create-home app

COPY --from=builder --chown=app:app /app/target/release/ingestion /usr/local/bin/ingestion

USER app

ENTRYPOINT ["/usr/local/bin/ingestion"]
