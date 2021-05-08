# Build context: parent directory
FROM rust:slim-bullseye as create-build-env
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc g++ cmake python3 make
COPY . /pps
WORKDIR /pps
ENV PYTHONUNBUFFERED=1
RUN python3 ./build-env/make.py --out /build-env

FROM rust:slim as build-pps
COPY . /pps
WORKDIR /pps
RUN cargo install --path cli

FROM ubuntu:focal
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc g++ cmake python3 make
WORKDIR /usr/local/bin
COPY --from=build-pps /usr/local/cargo/bin/pps-cli pps-cli
COPY --from=create-build-env /build-env /opt/pps-build-env
ENV JJS_PATH=/opt/pps-build-env
