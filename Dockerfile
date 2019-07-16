# Credit to https://alexbrand.dev/post/how-to-package-rust-applications-into-minimal-docker-containers/
FROM rustlang/rust:nightly AS build

WORKDIR /usr/src

ARG RUST_VERSION=nightly
ARG ARCHITECTURE=x86_64-unknown-linux-musl

RUN set -x \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y musl-tools \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN rustup target add x86_64-unknown-linux-musl --toolchain=nightly

RUN USER=root cargo new expense-report
WORKDIR /usr/src/expense-report
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN ls ./src
#RUN rm -f target/release/deps/expense-report*
RUN cargo install --target x86_64-unknown-linux-musl --path .
#RUN cargo install --path .

FROM alpine:latest
COPY --from=build /usr/local/cargo/bin/expense-report .
COPY static ./static
USER 1000
ENTRYPOINT ["./expense-report"]
