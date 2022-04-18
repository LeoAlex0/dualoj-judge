FROM docker.io/library/gcc:11.2.0 AS build

WORKDIR /workdir

COPY main.c main.c
RUN gcc -static ./main.c -o /bin/main

FROM docker.io/library/alpine:3.13
COPY --from=build /bin/main /bin/main
CMD [ "/bin/main" ]
