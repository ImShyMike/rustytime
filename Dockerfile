FROM rust:1.92-alpine AS builder

# Install system dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    postgresql-dev \
    bash \
    netcat-openbsd \
    git

# Set environment variables
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PQ_LIB_DIR=/usr/lib
ENV RUSTFLAGS="-C target-feature=-crt-static"

ARG BUILD_JOBS=4
ENV CARGO_BUILD_JOBS=${BUILD_JOBS}

WORKDIR /app

# Copy only Cargo files first for dependency caching
COPY rustytime/Cargo.toml rustytime/Cargo.lock ./rustytime/

# Create dummy source files to compile dependencies (for caching)
RUN mkdir -p rustytime/src && \
    echo "fn main() {}" > rustytime/src/main.rs && \
    echo "pub fn lib_main() {}" > rustytime/src/lib.rs

WORKDIR /app/rustytime

# Fetch and build dependencies (native compilation)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/rustytime/target \
    cargo fetch

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/rustytime/target \
    cargo build --profile good --no-default-features

# Remove dummy source
RUN rm -rf src/

# Copy actual source code
COPY rustytime/src ./src
COPY rustytime/build.rs ./build.rs
COPY rustytime/migrations ./migrations
COPY .git ../.git

# Build with actual source and copy binary out of cache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/rustytime/target \
    touch src/main.rs && \
    cargo build --profile good --no-default-features && \
    mkdir -p /tmp/target && \
    cp /app/rustytime/target/good/rustytime /tmp/target/rustytime

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache \
    libpq \
    ca-certificates \
    libgcc

COPY --from=builder /tmp/target/rustytime /usr/local/bin/rustytime

EXPOSE 3000
CMD ["rustytime"]