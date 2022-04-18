# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:latest as cargo-build

RUN apt-get update

RUN apt-get install musl-tools -y

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/rong

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
