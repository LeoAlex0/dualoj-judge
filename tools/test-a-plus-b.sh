#!/bin/sh -xe

SOLVER_ID=$(cargo run --bin=client -- --addr "grpc://localhost:443" --tls-ca-cert=".cert/client/ca.pem" upload "examples/A+B Problem/solver-bash" --exclude "**/target" --brief)
JUDGER_ID=$(cargo run --bin=client -- --addr "grpc://localhost:443" --tls-ca-cert=".cert/client/ca.pem" upload "examples/A+B Problem/judger" --exclude "**/target" --brief)
cargo run --bin=client -- --addr "grpc://localhost:443" --tls-ca-cert=".cert/client/ca.pem" build "${SOLVER_ID}"
cargo run --bin=client -- --addr "grpc://localhost:443" --tls-ca-cert=".cert/client/ca.pem" build "${JUDGER_ID}"

cargo run --bin=client -- --addr "grpc://localhost:443" --tls-ca-cert=".cert/client/ca.pem" judge "${JUDGER_ID}" "${SOLVER_ID}"
