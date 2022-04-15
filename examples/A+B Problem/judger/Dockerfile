FROM docker.io/library/rust:alpine3.13 AS build

# Tonic depends rustfmt to format generated source file.
COPY script /script
RUN [[ -f /script/setup-mirror.sh ]] && . /script/setup-mirror.sh || echo "no setup-mirror.sh, skipped"; \
    apk add protoc musl-dev \
    && rustup component add rustfmt

WORKDIR /workdir

COPY .cargo ./.cargo
COPY src ./src
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY build.rs ./build.rs
COPY proto ./proto

RUN cargo install --path . --root /

FROM docker.io/library/alpine:3.13
COPY --from=build /bin/a-plus-b-judger /bin
CMD [ "a-plus-b-judger" ]
