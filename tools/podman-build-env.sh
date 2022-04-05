#!/bin/sh -xe

podman run -it --rm -v "$(pwd)":/workdir docker.io/library/rust:1.59.0-alpine
