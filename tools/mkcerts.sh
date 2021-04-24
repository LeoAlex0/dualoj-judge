#!/bin/sh -xe

DIR=./.cert
SAN="localhost buildkitd.dualoj.svc.local"
SAN_CLIENT="judger.dualoj.svc.local"

mkdir -p ${DIR}/buildkitd ${DIR}/client
(
  echo ${SAN} | tr " " "\n" >SAN

  cd ${DIR}
  CAROOT=$(pwd) mkcert -cert-file buildkitd/cert.pem -key-file buildkitd/key.pem ${SAN}
  CAROOT=$(pwd) mkcert -client -cert-file client/cert.pem -key-file client/key.pem ${SAN_CLIENT}

  ln -f rootCA.pem buildkitd/ca.pem
  ln -f rootCA.pem client/ca.pem

  kubectl create secret generic buildkitd-certs --dry-run=client -o yaml --from-file=./buildkitd >../manifests/01-buildkitd-certs.yaml
  kubectl create secret generic judger-certs --dry-run=client -o yaml --from-file=./client > ../manifests/01-judger-certs.yaml
)
