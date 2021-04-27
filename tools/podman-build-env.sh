#!/bin/sh -xe

podman run -it --rm -v "$(pwd)":/workdir docker.io/clux/muslrust:nightly-2021-04-23
