#!/bin/sh -xe

for i in ./manifests/*.yml 
do
    kubectl apply -f ${i}
done
