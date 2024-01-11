FROM rust:alpine as builder

ENV RUSTFLAGS="-C target-feature=-crt-static"

WORKDIR /app

COPY . /app

RUN apk --no-cache add pkgconfig musl-dev openssl-dev clang-dev build-base make \
    && rm -rf /var/cache/apk/*

RUN cargo build --release

FROM alpine:latest

ARG BOT_VERSION=unknown

LABEL org.opencontainers.image.authors="mediclab"
LABEL version=$BOT_VERSION
LABEL description="Bot for posting photos with exif"

COPY --from=builder /app/target/release/beautiful_inno_bot /usr/local/bin/beautiful_inno_bot

RUN apk --no-cache add ca-certificates openssl libgcc libstdc++ \
    && rm -rf /var/cache/apk/*

CMD ["beautiful_inno_bot"]