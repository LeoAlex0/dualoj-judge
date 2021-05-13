FROM docker.io/library/rust:alpine3.13 AS build

# Tonic depends rustfmt to format generated source file.
RUN rustup component add rustfmt
RUN apk add protoc musl-dev

WORKDIR /workdir

COPY src ./src
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY build.rs ./build.rs
COPY proto ./proto

RUN cargo build && cargo install --path . --root /

FROM docker.io/library/alpine:3.13
COPY --from=build /bin/a-plus-b-judger /bin
ENTRYPOINT [ "a-plus-b-judger" ]
