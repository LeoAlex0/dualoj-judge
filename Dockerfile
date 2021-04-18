FROM docker.io/library/rust:alpine3.13 as build
# docker.io/library/rust:alpine has a bug which will cause Segment Fault when running.

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.tuna.tsinghua.edu.cn/g' /etc/apk/repositories
RUN apk add openssl-dev musl-dev
WORKDIR /workspace

RUN rustup override set nightly

COPY Cargo.toml .
COPY src/ ./src

RUN mkdir /root/.cargo
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry\
  --mount=type=cache,target=target\
  echo -e "\n\
[source.crates-io]\n\
replace-with = 'tuna'\n\
\n\
[source.tuna]\n\
registry = \"https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git\"\n\
" | tee /usr/local/cargo/config && \
  cargo install --bin=server --path=. --root=/

FROM docker.io/library/alpine:3.13

RUN apk add --no-cache libgcc

COPY --from=build /bin/server /bin/

ENV RUST_LOG info
CMD [ "server" ]
