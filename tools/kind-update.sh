#!/bin/sh -xe

DOCKER_BUILDKIT=1 docker build -f Cargo.toml -t localhost/dualoj-judge:demo .
"$(which kind)" load docker-image localhost/dualoj-judge:demo