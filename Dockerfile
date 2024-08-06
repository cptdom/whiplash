FROM rust:alpine AS builder

ENV APP_NAME=whiplash

WORKDIR /usr/src/${APP_NAME}

COPY Cargo.toml Cargo.lock ./
COPY src ./src

ARG CONFIG_PATH

COPY ${CONFIG_PATH} /config/

RUN apk update && apk add ca-certificates pkgconfig gcc musl-dev openssl-dev libc-dev build-base perl && apk cache clean
RUN cargo build --release
RUN cargo test --release

##############################################################################################
FROM alpine:latest

RUN apk update && apk add ca-certificates pkgconfig openssl && apk cache clean

ENV APP_NAME=whiplash

COPY --from=builder /config/config.yaml /config/config.yaml

COPY --from=builder /usr/src/${APP_NAME}/target/release/${APP_NAME} /usr/local/bin/${APP_NAME}

ENTRYPOINT ["/usr/local/bin/whiplash", "/config/config.yaml"]
