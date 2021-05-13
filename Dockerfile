FROM docker.io/clux/muslrust:nightly-2021-04-23 as build

RUN rustup component add rustfmt
RUN apt update && apt-get install -y protobuf-compiler

WORKDIR /workspace

COPY Cargo.toml .
COPY Cargo.lock .
COPY build.rs ./build.rs
COPY src/ ./src
COPY proto/ ./proto

RUN \
    --mount=type=cache,target=/root/.cargo/registry\
    --mount=type=cache,target=target\
    cargo install --path=. --root=/

FROM docker.io/library/alpine:3.13

RUN apk add --no-cache libgcc

COPY --from=build /bin/server /bin/client /bin/
COPY --from=docker.io/moby/buildkit:v0.8.3 /usr/bin/buildctl /bin

ENV RUST_LOG info
CMD [ "server" ]
