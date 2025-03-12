# A tool to check memberships for the French Society of Unicycling (CNM)

[![Bin](https://github.com/maxence-cornaton/verification-licences/actions/workflows/bin.yml/badge.svg)](https://github.com/maxence-cornaton/verification-licences/actions/workflows/bin.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
---

## Introduction

At the CNM, we've had issues checking manually every and each person who'd like to take in an event. This involved a
long and tedious process. This app strives to simplify the process.

## Getting started

The following tools are required:

- [Rust](https://www.rust-lang.org/): version 1.85+ (supporting Rust Edition 2024)
- [wasm-bindgen-cli](https://github.com/rustwasm/wasm-bindgen): required to build the WASM and JS libs from Rust code
- `wasm32-unknown-unknown` toolchain: install using `rustup target add wasm32-unknown-unknown`
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/): required to run WASM tests
- [Docker](https://www.docker.com/): used to package the app in an easy-to-share image
- [Node.js](https://nodejs.org/en): required to install the tools to build the CSS file
- [Tailwind-CLI](https://tailwindcss.com/docs/installation/tailwind-cli): used to build the CSS file
- [Flowbite](https://flowbite.com/docs/getting-started/introduction/#install-using-npm): CSS library based on Tailwind,
  required to build the CSS file

Once everything's installed, you can try and compile the app:

1. On Windows, run `.\build-wasm.bat` to build the WASM lib. It should create a new `./public/static/pkg` folder. If you use another OS, please adapt the script - it should
   not be too hard.
2. On windows, run `.\build-css.bat` to build the CSS file. It should appear in the `./public/static/` folder. If you use another OS, please adapt the script - it should
   not be too hard.
3. Build and run the app in demo mode with `cargo run --features demo`.
4. If that's the first time you run the app, you'll have to populate the memberships. You can do so with cURL or any
   other tool: `curl --request GET --url http://127.0.0.1:8000/api/memberships`.
5. Once the app is started and populated, go to http://127.0.0.1:8000/check-memberships. You should be able to check
   memberships.

You'll need Fileo credentials to do so.

## File Structure

The project is structured as follows:

- [dto](https://github.com/maxence-cornaton/verification-licences/tree/main/dto): a library with all shared DTOs.
- [public](https://github.com/maxence-cornaton/verification-licences/tree/main/public): the client-side assets (images,
  templates).
- [src](https://github.com/maxence-cornaton/verification-licences/tree/main/src): the main app location. Includes the
  server, the data retrieval logic and the validation logic.
- [wasm](https://github.com/maxence-cornaton/verification-licences/tree/main/wasm): the client-side code, written in
  Rust and compiled into WASM.

Besides, throughout your journey into the app, you'll encounter some generated folders:

- _data/_: location for the downloaded memberships. This acts as the database.
- _demo_data/_: similarly to `data`, this is the location for the demo memberships. This is populated when running in
  demo mode.
- _public/static/pkg/_: this is the location for the generated WASM and JS libs.

## Running the tests

Running the tests in the 3 packages is fairly simple:

1. To run tests against the main app, run `cargo test` at the root of the project
2. To run tests against the `dto` crate, run the same command in the `dto` package
3. To run tests against the `wasm` crate, run `wasm-pack test --headless --firefox` in the `wasm` crate.

## Args

To run the app in production, you'll need to pass the following args while starting the app:

| Name                   | Description                                                                                                                             | Type   | Required | Default                        |
|------------------------|-----------------------------------------------------------------------------------------------------------------------------------------|--------|----------|--------------------------------|
| --email-sender-name    | The name email recipients should see                                                                                                    | String | Yes      | None                           |
| --email-sender-address | The address that should be used to send the emails.<br/>⚠ If it doesn't fit with the SMTP login, the SMTP server may reject the emails. | String | Yes      | None                           |
| --reply-to             | Which address the recipients should reply to                                                                                            | String | No       | `--email-sender-address` value |
| --smtp-server          | The SMTP server to use to send emails                                                                                                   | String | No       | smtp.gmail.com                 |
| --smtp-port            | The SMTP port the SMTP is listening on                                                                                                  | u16    | No       | 587                            |
| --smtp-login           | The login used to access the SMTP server                                                                                                | String | Yes      | None                           |
| --smtp-password        | The password used to access the SMTP server                                                                                             | String | Yes      | None                           |

E.g.:

```shell
cargo run -- \
  --email-sender-name=<sender-name> \
  --email-sender-address=<email-sender-address> \
  --reply-to=<reply-to> \
  --smtp-server=<smtp-server> \
  --smtp-port=<smtp-port> \
  --smtp-login=<smtp-login> \
  --smtp-password=<smtp-password>
```
