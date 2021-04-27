# DualOJ-Judge

DualOJ-Judge is judger part of DualOJ, basically written with Rust language.

This component **DO NOT** need expose.

It consists of several parts:

- [ ] Builder
- [ ] Judge runner
- [ ] Client (for debugging, use proxy to connect)
- [ ] Write the press release

ALL Components are communicated with HTTP.

## Builder

Build tester's image & solver's image

Some Option should be tested:

- [ ] kaniko
  - LICENSE: Apache-2.0
  - kaniko is not an officially supported Google product
- [ ] buildah
- [ ] BuildKit

## Judge runner

1. Give a poster uri & post api key to post test result.

## File server

accept all directory save & archive.

## Internal registry

Some Option should be tested:

- [ ] harbor/harbor
- [ ] docker.io/library/registry

## Known BUGS

- [ ] `docker build -f ./Cargo.toml .` failed, caused by `denzp/cargo-wharf` [issue#32](https://github.com/denzp/cargo-wharf/issues/32)
