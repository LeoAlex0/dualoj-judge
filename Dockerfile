FROM docker.io/clux/muslrust:nightly-2021-04-23 as build

WORKDIR /workspace

COPY Cargo.toml .
COPY Cargo.lock .
COPY build.rs ./build.rs
COPY src/ ./src
COPY proto/ ./proto

RUN rustup component add rustfmt
RUN apt update && apt-get install -y protobuf-compiler
RUN \
    --mount=type=cache,target=/root/.cargo/registry\
    --mount=type=cache,target=target\
    cargo install --bin=server --path=. --root=/

FROM docker.io/library/alpine:3.13

RUN apk add --no-cache libgcc

COPY --from=build /bin/server /bin/

ENV RUST_LOG info
CMD [ "server" ]
