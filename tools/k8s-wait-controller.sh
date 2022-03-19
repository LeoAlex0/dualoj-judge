#!/bin/sh -xe

kubectl wait --namespace dualoj --for=condition=ready pod --selector "app=judger"
