FROM rust:latest

RUN rustup target add wasm32-unknown-unknown

RUN cargo install candid-extractor
RUN cargo install cargo-audit

RUN DFX_VERSION=0.15.1 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
ENV PATH="${PATH}:/root/bin"