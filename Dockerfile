FROM rust:alpine as builder

ENV RUSTFLAGS="-C target-feature=-crt-static"

WORKDIR /app

COPY . /app

RUN apk --no-cache add pkgconfig musl-dev openssl-dev clang-dev build-base make libheif-dev \
    && rm -rf /var/cache/apk/*

RUN cargo build --release

FROM alpine:latest

MAINTAINER mediclab

ARG BOT_VERSION=unknown
ENV BOT_VERSION=$BOT_VERSION

LABEL org.opencontainers.image.authors="mediclab"
LABEL version=$BOT_VERSION
LABEL description="Bot for posting photos with exif"

COPY --from=builder /app/target/release/beautiful_inno_bot /usr/local/bin/beautiful_inno_bot

RUN apk --no-cache add ca-certificates openssl libgcc libstdc++ libheif libheif-tools \
    && rm -rf /var/cache/apk/*

CMD ["beautiful_inno_bot"]