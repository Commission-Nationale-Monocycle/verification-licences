FROM rust:1.85.0 AS wasm-builder
WORKDIR /build

RUN cargo install wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown

COPY ./wasm ./wasm
COPY ./dto ./dto

RUN cargo build --target=wasm32-unknown-unknown --manifest-path=./wasm/Cargo.toml --release
RUN wasm-bindgen --target=web ./wasm/target/wasm32-unknown-unknown/release/wasm.wasm --out-dir=pkg

FROM node:22.14.0 as css-builder
WORKDIR build

RUN npm install tailwindcss @tailwindcss/cli flowbite

COPY ./public ./public
# Wasm folder should be copied, in case it creates element with styles never used elsewhere
COPY ./wasm ./wasm
COPY ./package.json .
COPY ./package-lock.json .
COPY ./tailwind.config.js .

RUN npx @tailwindcss/cli -i public/styles/styles.css -o public/static/styles.css --minify

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
COPY --from=css-builder /build/public/static/styles.css ./public/static/

EXPOSE 8000

ENTRYPOINT ["/app/verification-licences"]
