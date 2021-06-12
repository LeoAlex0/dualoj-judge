#!/bin/sh -xe

eval "$(minikube -p minikube docker-env)"
DOCKER_BUILDKIT=1 docker build -t localhost/dualoj-judge:demo .
kubectl -n dualoj delete --selector "app=judger" pods
