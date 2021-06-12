#!/bin/bash -xe

DIR=./.cert
BUILDKITD_SAN="buildkitd.dualoj.svc.cluster.local"
JUDGER_SAN="judger.dualoj.svc.cluster.local"

mkdir -p ${DIR}/{buildkitd,judger}
(
  cd ${DIR}
  CAROOT=$(pwd) mkcert -cert-file buildkitd/cert.pem -key-file buildkitd/key.pem ${BUILDKITD_SAN}
  CAROOT=$(pwd) mkcert -client -cert-file judger/cert.pem -key-file judger/key.pem ${JUDGER_SAN}

  ln -f rootCA.pem buildkitd/ca.pem
  ln -f rootCA.pem judger/ca.pem

  kubectl -n dualoj create secret generic buildkitd-certs --dry-run=client -o yaml --from-file=./buildkitd >../manifests/01-buildkitd-certs.yaml
  kubectl -n dualoj create secret generic judger-certs --dry-run=client -o yaml --from-file=./judger >../manifests/01-judger-certs.yaml
)
