#!/bin/sh -xe

podman build -t dualoj-judge:demo .
podman save localhost/dualoj-judge:demo | sudo "$(which kind)" load image-archive /dev/stdin
