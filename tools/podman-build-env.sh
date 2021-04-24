#!/bin/sh -xe

podman run -it --rm -v "$(pwd)":/workdir docker.io/library/rust:alpine3.13
