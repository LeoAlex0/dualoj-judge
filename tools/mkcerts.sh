#!/bin/bash -xe

DIR=./.cert
BUILDKITD_SAN="buildkitd.dualoj.svc.cluster.local"
JUDGER_SAN="judger.dualoj.svc.cluster.local"
REGISTRY_SAN="registry.dualoj.svc.cluster.local"
INGRESS_SAN="localhost workstation 10.0.1.2"
CLIENT_SAN="localhost"

mkdir -p ${DIR}/{buildkitd,judger,registry,ingress,client}
(
  cd ${DIR}
  CAROOT=$(pwd) mkcert -cert-file buildkitd/cert.pem -key-file buildkitd/key.pem ${BUILDKITD_SAN}
  CAROOT=$(pwd) mkcert -client -cert-file judger/cert.pem -key-file judger/key.pem ${JUDGER_SAN}
  CAROOT=$(pwd) mkcert -cert-file ingress/cert.pem -key-file ingress/key.pem ${INGRESS_SAN}
  CAROOT=$(pwd) mkcert -cert-file registry/cert.pem -key-file registry/key.pem ${REGISTRY_SAN}
  CAROOT=$(pwd) mkcert -client -cert-file client/cert.pem -key-file client/key.pem ${CLIENT_SAN}

  ln -f rootCA.pem buildkitd/ca.pem
  ln -f rootCA.pem judger/ca.pem
  ln -f rootCA.pem client/ca.pem

  kubectl -n dualoj create secret generic buildkitd-certs --dry-run=client -o yaml --from-file=./buildkitd >../manifests/01-buildkitd-certs.yaml
  kubectl -n dualoj create secret generic judger-certs --dry-run=client -o yaml --from-file=./judger >../manifests/01-judger-certs.yaml
  kubectl -n dualoj create secret tls registry-ingress --dry-run=client -o yaml --cert=./registry/cert.pem --key=./registry/key.pem >../manifests/01-registry-ingress.yaml
  kubectl -n dualoj create secret tls ingress-tls --dry-run=client -o yaml --cert=./ingress/cert.pem --key=./ingress/key.pem >../manifests/01-ingress-tls.yaml
)

docker cp .cert/rootCA.pem \
kind-control-plane:/usr/local/share/ca-certificates/custom.crt

docker exec -it kind-control-plane update-ca-certificates
docker restart kind-control-plane
