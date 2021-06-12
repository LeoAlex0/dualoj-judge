#!/bin/sh -xe

ADDR=${ADDR:="$(minikube service -n dualoj judger-controller --url)"}
CPU_LIMIT="300"
TIME_LIMIT="5"
MEMORY_LIMIT="64"

if not [ -z "${DUALOJ_TLS_CA_FILE}" ] ;then
    set -- --tls-ca-cert="${DUALOJ_TLS_CA_FILE}"
    CA_ARG="$*"
fi

SOLVER_ID=$(cargo run --bin=client -- ${CA_ARG} --addr "${ADDR}" upbuild "examples/A+B Problem/$2" --exclude "**/target" --brief)
JUDGER_ID=$(cargo run --bin=client -- ${CA_ARG} --addr "${ADDR}" upbuild "examples/A+B Problem/$1" --exclude "**/target" --brief)

cargo run --bin=client -- ${CA_ARG} --addr "${ADDR}" judge --cpu-limit="${CPU_LIMIT}" \
    --time-limit="${TIME_LIMIT}" \
    --mem-limit="${MEMORY_LIMIT}" \
    "${JUDGER_ID}" "${SOLVER_ID}"
