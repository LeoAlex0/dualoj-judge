#!/bin/sh -xe

kubectl run -it --rm --image localhost/dualoj-judge:demo k8s-dualoj-judge-env ash
