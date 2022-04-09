FROM docker.io/library/rust:1.59.0-alpine as build

# environment
COPY script /script
COPY .cargo $HOME/.cargo
RUN [[ -f /script/setup-mirror.sh ]] && . /script/setup-mirror.sh || echo "no setup-mirror.sh, skipped"; \
    apk add g++ openssl-dev cmake make \
    && rustup component add rustfmt

WORKDIR /workspace

# pre-compile for cache dependency
COPY Cargo.toml .
COPY Cargo.lock .
RUN  mkdir -p src/bin && echo "fn main() {}" > src/bin/server.rs \
    && cargo fetch --locked \
    && rm -r src

# build
COPY build.rs ./build.rs
COPY src/ ./src
COPY proto/ ./proto

RUN RUSTFLAGS='-C target-feature=-crt-static' cargo install --path=. --root=/

FROM docker.io/library/alpine:3.15

COPY script /script
RUN [[ -f /script/setup-mirror.sh ]] && . /script/setup-mirror.sh || echo "no setup-mirror.sh, skipped"; \
    apk add --no-cache libgcc

COPY --from=build /bin/server /bin/client /bin/
COPY --from=docker.io/moby/buildkit:v0.8.3 /usr/bin/buildctl /bin

ENV RUST_LOG info
CMD [ "server" ]
