#!/bin/sh -xe

ADDR="grpc://workstation:443"

SOLVER_ID=$(cargo run --bin=client -- --addr "${ADDR}" --tls-ca-cert=".cert/client/ca.pem" upload "examples/A+B Problem/$2" --exclude "**/target" --brief)
JUDGER_ID=$(cargo run --bin=client -- --addr "${ADDR}" --tls-ca-cert=".cert/client/ca.pem" upload "examples/A+B Problem/$1" --exclude "**/target" --brief)
cargo run --bin=client -- --addr "${ADDR}" --tls-ca-cert=".cert/client/ca.pem" build "${SOLVER_ID}"
cargo run --bin=client -- --addr "${ADDR}" --tls-ca-cert=".cert/client/ca.pem" build "${JUDGER_ID}"

cargo run --bin=client -- --addr "${ADDR}" --tls-ca-cert=".cert/client/ca.pem" judge --cpu-limit=300 "${JUDGER_ID}" "${SOLVER_ID}"
