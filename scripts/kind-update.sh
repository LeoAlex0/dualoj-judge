#!/bin/sh -xe

DOCKER_BUILDKIT=1 docker build -t dualoj-judge:demo .
kind load docker-image dualoj-judge:demo
