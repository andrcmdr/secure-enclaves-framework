FROM public.ecr.aws/amazonlinux/amazonlinux:2023 as builder
# https://gallery.ecr.aws/amazonlinux/amazonlinux

ENV SHELL="/usr/bin/env bash"

RUN dnf upgrade -y
RUN dnf install -y git gcc pkgconfig openssl openssl-devel openssl-libs perl perl-FindBin
RUN dnf install -y time which hostname

ENV RUST_LOG="debug"
ENV RUST_BACKTRACE="full"

ENV CARGO_HOME="$HOME/rust" RUSTUP_HOME="$HOME/rustup" PATH="$PATH:$HOME/rust/bin"
RUN curl -fsSL https://sh.rustup.rs | bash -is -- -y --verbose --no-modify-path --default-toolchain stable --profile minimal
# RUN rustup -v toolchain install nightly --profile minimal
# RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app-builder

# COPY --link app.rs/ /app-builder/app.rs/

# CMD whoami; uname -a; date; pwd;
# CMD sleep infinity
CMD tail -f /dev/null
