FROM rust:1.85.0 AS wasm-builder
WORKDIR /build

RUN cargo install wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown

COPY ./wasm ./wasm
COPY ./dto ./dto

RUN cargo build --target=wasm32-unknown-unknown --manifest-path=./wasm/Cargo.toml --release
RUN wasm-bindgen --target=web ./wasm/target/wasm32-unknown-unknown/release/wasm.wasm --out-dir=pkg

FROM rust:1.85.0 AS bin-builder
WORKDIR /build

COPY ./dto ./dto
COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN cargo install --path .

FROM ubuntu:24.04 AS runner
WORKDIR /app

COPY ./Rocket.toml ./
COPY ./public ./public
COPY --from=bin-builder /build/target/release/verification-licences .
COPY --from=wasm-builder /build/pkg ./public/pkg

EXPOSE 8000

ENTRYPOINT ["/app/verification-licences"]
