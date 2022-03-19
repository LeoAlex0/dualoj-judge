# DualOJ-Judge

[![123](https://github.com/LeoAlex0/dualoj-judge/actions/workflows/rust.yml/badge.svg)](https://github.com/LeoAlex0/dualoj-judge/actions/workflows/rust.yml)

DualOJ-Judge is the judge module of DualOJ (WIP), basically written in Rust language.

WARNING:

* This component **DO NOT** need to be exposed.
* This project are still Work In Progress.

## Introduction & final goal

This module can make DualOJ able to let problem-solvers & judgers to use `Dockerfile`
customize environment and dependencies.

This means it is no longer necessary to use some extreme methods
to optimize performance in most case, or copy & paste code for reusing.

## Quick develop guide

### Dependencies

* `kubernetes cluster`, can use kubectl to manage, with an `ingress-controller`.
  * recommended to use [minikube](https://github.com/kubernetes/minikube) with `ingress` addon to develop.
* `OpenSSL` or something can be used to generate Self-signed SSL Certifications.
  * recommended to use [mkcert](https://github.com/FiloSottile/mkcert).

And next steps will consider you using a **recommended** configuration.

### Start a local minikube cluster

It is recommended to assign at least 4 CPU cores and 8 GB of memory: `minikube start --cpus=4 --memory=8192`

### Generate self-signed certification

Cause for using [BuildKit](https://github.com/moby/buildkit) securely, **must**
generate a self-signed certification first.

You can simply do `tools/mkcerts.sh` for this step.

### Upload certificate to container

Run `tools/minikube-upcerts.sh`.

### Build & Load image

For `minikube` & `docker` user, you can easily do `tools/minikube-update.sh` for this step.

### Apply manifests

Just `kubectl apply -f ./manifests`.

### Update pod

So if you complete a new feature or just for test, you can use `tools/minikube-update.sh`
to build and update using changed new sources.

(For `minikube` & `docker` only)

### Interface debug

If you need to invoke some commands, you can use `cargo run --bin=client` to run a client.

And you may need to add `--tls-ca-cert=".cert/client/ca.pem"` flag to trust CA Certificate generated above.

## Builder support

Build `Judger` image & `Solver` image

Some Option should be tested:

* [ ] [kaniko](https://github.com/GoogleContainerTools/kaniko)
  * LICENSE: Apache-2.0
  * kaniko is not an officially supported Google product
* [ ] [buildah](https://github.com/containers/buildah)
* [x] [BuildKit](https://github.com/moby/buildkit)

## Internal registry support

Some Option should be tested:

* [ ] [harbor/harbor](https://github.com/goharbor/harbor)
* [x] [docker.io/library/registry](https://hub.docker.com/_/registry/)

## Known BUGS
