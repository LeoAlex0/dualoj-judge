#!/bin/sh -xe

./scripts/kind-update.sh
kubectl run -it --rm --image dualoj-judge:demo k8s-dualoj-judge-env ash
