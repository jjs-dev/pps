# Build context: parent directory
FROM alpine:edge as create-build-env
RUN apk add gcc g++ cmake python3 make rust
RUN apk add cargo
COPY . /pps
WORKDIR /pps
ENV PYTHONUNBUFFERED=1
RUN python3 ./build-env/make.py --out /build-env

FROM alpine:edge as build-pps
RUN apk add rust
RUN apk add cargo
COPY . /pps
WORKDIR /pps
RUN cargo install --path cli

FROM alpine:edge
RUN apk add gcc g++ cmake python3 make
WORKDIR /usr/local/bin
COPY --from=build-pps /root/.cargo/bin/pps-cli pps-cli
COPY --from=create-build-env /build-env /opt/pps-build-env
ENV JJS_PATH=/opt/pps-build-env
