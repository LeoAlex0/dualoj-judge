#!/bin/sh -xe

kubectl run -it --rm --image dualoj-judge:demo k8s-dualoj-judge-env
