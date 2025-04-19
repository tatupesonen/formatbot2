ARG BACKEND
ARG RUST_VERSION=1.8.6

# ------------ Base builder for Rust ------------
FROM rust:${RUST_VERSION}-alpine AS base
ARG BACKEND

# Install Rust build deps
RUN apk add --no-cache musl-dev openssl-dev pkgconfig build-base

WORKDIR /app
COPY . .
RUN cargo build --bin=formatter --release --features=$BACKEND

# ------------ TypeScript runtime ------------
FROM rust:${RUST_VERSION}-alpine AS typescript-runtime
RUN apk add --no-cache nodejs npm libgcc
RUN npm install -g prettier
WORKDIR /app
COPY --from=base /app/target/release/formatter /app/formatter

# ------------ Rust runtime ------------
FROM rust:${RUST_VERSION}-alpine AS rust-runtime
RUN rustup component add rustfmt
RUN apk add --no-cache libgcc
WORKDIR /app
COPY --from=base /app/target/release/formatter /app/formatter

# ------------ PHP runtime (with php-cs-fixer) ------------
FROM php:8.2-cli-alpine AS php-runtime
# Install PHP dependencies + curl for downloading cs-fixer
RUN apk add --no-cache curl

# Download PHP-CS-Fixer
RUN curl -L https://cs.symfony.com/download/php-cs-fixer-v3.phar -o /usr/local/bin/php-cs-fixer && \
    chmod +x /usr/local/bin/php-cs-fixer

WORKDIR /app
COPY --from=base /app/target/release/formatter /app/formatter

# ------------ Final image (selected at build time) ------------
ARG BACKEND
FROM ${BACKEND}-runtime

ENTRYPOINT ["/app/formatter"]
