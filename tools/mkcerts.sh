#!/bin/bash -xe

DIR=./.cert
BUILDKITD_SAN="buildkitd.dualoj.svc.local"
JUDGER_SAN="judger.dualoj.svc.local"
INGRESS_SAN="localhost 10.0.1.2"
CLIENT_SAN="localhost"

mkdir -p ${DIR}/{buildkitd,judger,ingress,client}
(
  cd ${DIR}
  CAROOT=$(pwd) mkcert -cert-file buildkitd/cert.pem -key-file buildkitd/key.pem ${BUILDKITD_SAN}
  CAROOT=$(pwd) mkcert -cert-file judger/cert.pem -key-file judger/key.pem ${JUDGER_SAN}
  CAROOT=$(pwd) mkcert -cert-file ingress/cert.pem -key-file ingress/key.pem ${INGRESS_SAN}
  CAROOT=$(pwd) mkcert -client -cert-file client/cert.pem -key-file client/key.pem ${CLIENT_SAN}

  ln -f rootCA.pem buildkitd/ca.pem
  ln -f rootCA.pem judger/ca.pem
  ln -f rootCA.pem client/ca.pem

  kubectl -n dualoj create secret generic buildkitd-certs --dry-run=client -o yaml --from-file=./buildkitd >../manifests/01-buildkitd-certs.yaml
  kubectl -n dualoj create secret generic judger-certs --dry-run=client -o yaml --from-file=./client >../manifests/01-judger-certs.yaml
  kubectl -n dualoj create secret tls ingress-tls --dry-run=client -o yaml --cert=./ingress/cert.pem --key=./ingress/key.pem >../manifests/01-ingress-tls.yaml
)
