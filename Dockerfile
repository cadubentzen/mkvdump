FROM rust AS builder

WORKDIR /build
RUN rustup target add $(uname -m)-unknown-linux-musl

ADD . .

# Run tests
RUN cargo test --release --target $(uname -m)-unknown-linux-musl

# Build release
RUN cargo build --release --target $(uname -m)-unknown-linux-musl
RUN cp target/$(uname -m)-unknown-linux-musl/release/mkvdump /usr/local/bin/mkvdump

FROM alpine

COPY --from=builder /usr/local/bin/mkvdump /usr/local/bin/

ENTRYPOINT ["mkvdump"]
