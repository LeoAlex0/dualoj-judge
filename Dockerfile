FROM docker.io/library/alpine:latest as build
# docker.io/library/rust:alpine has a bug which will cause Segment Fault when running.

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.tuna.tsinghua.edu.cn/g' /etc/apk/repositories
RUN apk update && apk add cargo openssl-dev

COPY Cargo.toml .
COPY src/ ./src

RUN mkdir /root/.cargo
RUN \
  --mount=type=cache,target=/root/.cargo\
  --mount=type=cache,target=target\
  echo -e "\n\
[source.crates-io]\n\
replace-with = 'tuna'\n\
\n\
[source.tuna]\n\
registry = \"https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git\"\n\
" | tee /root/.cargo/config && \
  cargo install --bin=server --path=. --root=/


FROM docker.io/library/alpine:latest

RUN apk add --no-cache libgcc

COPY --from=build /bin/server /bin/

ENV RUST_LOG info
CMD [ "server" ]
