FROM rust:1.85.0 AS builder
WORKDIR /build
RUN cargo install wasm-bindgen-cli

COPY . .
RUN cargo install --path .

FROM ubuntu:24.04 AS runner
WORKDIR /app
COPY --from=builder /build/target/release/verification-licences .
COPY --from=builder /build/public ./public
COPY --from=builder /build/Rocket.toml ./
ENTRYPOINT ["/app/verification-licences"]
