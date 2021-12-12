FROM docker.io/clux/muslrust:1.57.0 as build

RUN rustup component add rustfmt
RUN apt update && apt-get install -y protobuf-compiler

WORKDIR /workspace

COPY Cargo.toml .
COPY Cargo.lock .

# A hack for cache dependencies
RUN mkdir -p src/bin && touch src/bin/server.rs && cargo fetch --locked && rm -rf src

COPY build.rs ./build.rs
COPY src/ ./src
COPY proto/ ./proto

RUN cargo install --path=. --root=/

FROM docker.io/library/alpine:3.15

RUN apk add --no-cache libgcc

COPY --from=build /bin/server /bin/client /bin/
COPY --from=docker.io/moby/buildkit:v0.8.3 /usr/bin/buildctl /bin

ENV RUST_LOG info
CMD [ "server" ]
