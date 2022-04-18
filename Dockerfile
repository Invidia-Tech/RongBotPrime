####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN update-ca-certificates

# Create appuser
ENV USER=rong
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

RUN cargo new --bin rongbotprime
WORKDIR /rongbotprime

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

# Cache builds of libraries for release
RUN cargo build --release

# Delete and move in the real source
RUN rm -rf src/

COPY ./src ./src
COPY sqlx-data.json sqlx-data.json
COPY ./migrations ./migrations

ENV SQLX_OFFLINE true

# build for release
RUN rm ./target/release/deps/rongbotprime*
RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM ubuntu:latest

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /rongbotprime

# Copy our build
COPY --from=builder /rongbotprime/target/release/rongbotprime ./

# DB Migrations
COPY ./migrations ./migrations

# Use an unprivileged user.
USER rong:rong

CMD ["/rongbotprime/rongbotprime"]
