#!/bin/sh -xe

ADDR="grpc://workstation:443"
CPU_LIMIT="300"
TIME_LIMIT="5"
MEMORY_LIMIT="64"

SOLVER_ID=$(cargo run --bin=client -- --addr "${ADDR}" --tls-ca-cert=".cert/client/ca.pem" upload "examples/A+B Problem/$2" --exclude "**/target" --brief)
JUDGER_ID=$(cargo run --bin=client -- --addr "${ADDR}" --tls-ca-cert=".cert/client/ca.pem" upload "examples/A+B Problem/$1" --exclude "**/target" --brief)
cargo run --bin=client -- --addr "${ADDR}" --tls-ca-cert=".cert/client/ca.pem" build "${SOLVER_ID}"
cargo run --bin=client -- --addr "${ADDR}" --tls-ca-cert=".cert/client/ca.pem" build "${JUDGER_ID}"

cargo run --bin=client -- --addr "${ADDR}" --tls-ca-cert=".cert/client/ca.pem" judge --cpu-limit="${CPU_LIMIT}" \
    --time-limit="${TIME_LIMIT}" \
    --mem-limit="${MEMORY_LIMIT}" \
    "${JUDGER_ID}" "${SOLVER_ID}"
