# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:latest as cargo-build

RUN apt-get update

RUN apt-get install musl-tools -y

RUN rustup target add aarch64-unknown-linux-musl

WORKDIR /usr/src/rong

COPY Cargo.toml Cargo.toml

RUN mkdir src/

# ENV CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc

# RUN ln -s /usr/local/opt/musl-cross/bin/x86_64-linux-musl-gcc /usr/local/bin/musl-gcc && echo "$CC_x86_64_unknown_linux_musl"

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/rongbotprime*

COPY . .

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

RUN addgroup -g 1000 rong

RUN adduser -D -s /bin/sh -u 1000 -G rong rong

WORKDIR /home/rong/bin/

COPY --from=cargo-build /usr/src/rong/target/x86_64-unknown-linux-musl/release/rong_bot_prime .

RUN chown rong:rong rong_bot_prime

USER rong

CMD ["./rong_bot_prime"]
