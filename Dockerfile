FROM rust:1.86-alpine AS builder

# Install system dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    postgresql-dev

# Add target
RUN rustup target add x86_64-unknown-linux-musl

# Set environment variables
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PQ_LIB_DIR=/usr/lib
ENV RUSTFLAGS="-C target-feature=-crt-static"

WORKDIR /app

# Copy only Cargo files first for dependency caching
COPY rustytime/Cargo.toml rustytime/Cargo.lock ./rustytime/

# Create a dummy main.rs to compile dependencies (for caching)
RUN mkdir -p rustytime/src && \
    echo "fn main() {}" > rustytime/src/main.rs

WORKDIR /app/rustytime

# Fetch and build dependencies
RUN cargo fetch --target x86_64-unknown-linux-musl
RUN cargo build --release --no-default-features --target x86_64-unknown-linux-musl

# Remove dummy source
RUN rm -rf src/

# Copy actual source code
COPY rustytime/src ./src
COPY rustytime/migrations ./migrations
COPY rustytime/templates ./templates

# Build with actual source
RUN touch src/main.rs && \
    cargo build --release --no-default-features --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache \
    libpq \
    ca-certificates \
    libgcc

COPY --from=builder /app/rustytime/target/x86_64-unknown-linux-musl/release/rustytime /usr/local/bin/rustytime

EXPOSE 3000

CMD ["rustytime"]
