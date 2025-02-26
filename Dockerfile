FROM rust:1.85.0 AS builder
WORKDIR /build
COPY . .
RUN cargo install --path .

FROM ubuntu:24.04 AS runner
WORKDIR /app
COPY --from=builder /build/target/release/verification-licences .
ENTRYPOINT ["/app/verification-licences"]
