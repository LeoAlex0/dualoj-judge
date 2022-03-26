#!/bin/sh -xe

minikube image build -t localhost/dualoj-judge:demo .
kubectl -n dualoj delete --selector "app=judger" pods
