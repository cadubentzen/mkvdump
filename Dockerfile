FROM rust AS builder

WORKDIR /build
RUN rustup target add x86_64-unknown-linux-musl

ADD . .

# Run tests
RUN cargo test --release --target x86_64-unknown-linux-musl

# Build release
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/mkvdump /usr/local/bin/

ENTRYPOINT ["mkvdump"]
