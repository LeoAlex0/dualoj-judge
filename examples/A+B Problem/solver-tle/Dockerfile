FROM docker.io/library/alpine:3.13 AS build

WORKDIR /workdir
RUN apk add gcc musl-dev

COPY main.c main.c
RUN gcc ./main.c -o /bin/main

FROM docker.io/library/alpine:3.13
COPY --from=build /bin/main /bin/main
CMD [ "/bin/main" ]
