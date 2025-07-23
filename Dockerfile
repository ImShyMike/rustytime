FROM rust:1.86-alpine AS builder

WORKDIR /app
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    postgresql-dev

RUN rustup target add x86_64-unknown-linux-musl

COPY . .
WORKDIR /app/rustytime

ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PQ_LIB_DIR=/usr/lib

ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN cargo fetch --target x86_64-unknown-linux-musl
RUN cargo build --release --no-default-features --target x86_64-unknown-linux-musl

FROM alpine:latest
RUN apk add --no-cache \
    libpq \
    ca-certificates \
    libgcc
COPY --from=builder /app/rustytime/target/x86_64-unknown-linux-musl/release/rustytime /usr/local/bin/rustytime
COPY --from=builder /app/rustytime/templates /usr/local/bin/templates
EXPOSE 3000

CMD ["rustytime"]
