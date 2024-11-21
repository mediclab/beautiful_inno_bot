FROM rust:alpine

ENV RUSTFLAGS="-C target-feature=-crt-static"

WORKDIR /app

RUN apk --no-cache add pkgconfig musl-dev openssl-dev clang-dev build-base make libheif-dev libpq-dev libheif-tools \
    && rm -rf /var/cache/apk/*