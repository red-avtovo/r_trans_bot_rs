FROM ekidd/rust-musl-builder:stable-openssl11 AS cargo-build

COPY Cargo.toml Cargo.toml

RUN mkdir src/ && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release --target=x86_64-unknown-linux-musl
RUN rm -f target/x86_64-unknown-linux-musl/release/deps/remote-transmission-bot*

COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl


FROM alpine:3.10

LABEL authors="red.avtovo@gmail.com"

COPY --from=cargo-build /home/rust/src/target/x86_64-unknown-linux-musl/release/remote-transmission-bot /opt/

ENV RUST_LOG="info,transmission_rpc=warn"

RUN apk add --no-cache ca-certificates && update-ca-certificates
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
WORKDIR /opt
CMD ["./remote-transmission-bot"]