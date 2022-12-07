# Credit to https://alexbrand.dev/post/how-to-package-rust-applications-into-minimal-docker-containers/
FROM rust:1.64 AS build

WORKDIR /usr/src

ARG RUST_VERSION=nightly
ARG ARCHITECTURE=x86_64-unknown-linux-musl

RUN set -x \
    && apt update \
    && DEBIAN_FRONTEND=noninteractive apt install -y musl-tools \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN rustup target add x86_64-unknown-linux-musl
RUN USER=root cargo new expense-report
WORKDIR /usr/src/expense-report
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY src ./src
RUN ls ./src
#RUN cargo install --target x86_64-unknown-linux-musl --path .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=build /usr/local/cargo/bin/expense-report .
COPY static ./static
RUN apt-get update && apt-get install -y libsqlite3-0
#USER 1000
ENTRYPOINT ["./expense-report"]
CMD ["/db_dir", "finances.gnucash"]
