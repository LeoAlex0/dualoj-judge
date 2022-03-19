#!/bin/sh

docker cp .cert/rootCA.pem \
minikube:/usr/local/share/ca-certificates/custom.crt

docker exec -it minikube update-ca-certificates
docker restart minikube