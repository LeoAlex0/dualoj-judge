#!/bin/sh

sudo podman cp .cert/rootCA.pem \
minikube:/usr/local/share/ca-certificates/custom.crt

sudo podman exec -it minikube update-ca-certificates
sudo podman restart minikube
minikube stop && minikube start