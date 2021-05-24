#!/bin/sh

docker cp .cert/rootCA.pem \
kind-control-plane:/usr/local/share/ca-certificates/custom.crt

docker exec -it kind-control-plane update-ca-certificates
docker restart kind-control-plane
