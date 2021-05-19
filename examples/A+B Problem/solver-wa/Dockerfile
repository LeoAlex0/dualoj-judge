FROM docker.io/library/haskell:9.0.1-stretch AS build

WORKDIR /workdir

COPY main.hs main.hs
RUN ghc ./main.hs -static -o /bin/main

FROM docker.io/library/debian:stable-20210511-slim

COPY --from=build /bin/main /bin/main
CMD [ "/bin/main" ]
