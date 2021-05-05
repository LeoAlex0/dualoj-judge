#!/bin/sh -xe

UUID=$(cargo run --bin=client -- --addr "grpc://localhost:443" --tls-ca-cert=".cert/client/ca.pem" upload --exclude="./**/target" --brief .)
cargo run --bin=client -- --addr "grpc://localhost:443" --tls-ca-cert=".cert/client/ca.pem" build ${UUID}
