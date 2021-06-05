# DualOJ-Judge

DualOJ-Judge is the judge module of DualOJ (WIP), basically written in Rust language.

This component **DO NOT** need to be exposed.

## Builder support

Build `Judger` image & `Solver` image

Some Option should be tested:

- [ ] [kaniko](https://github.com/GoogleContainerTools/kaniko)
  - LICENSE: Apache-2.0
  - kaniko is not an officially supported Google product
- [ ] [buildah](https://github.com/containers/buildah)
- [x] [BuildKit](https://github.com/moby/buildkit)

## Internal registry support

Some Option should be tested:

- [ ] [harbor/harbor](https://github.com/goharbor/harbor)
- [x] [docker.io/library/registry](https://hub.docker.com/_/registry/)

## Known BUGS
