# adapted from ms devcontainers
FROM rust:1-bullseye

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
     && apt-get -y install --no-install-recommends neofetch curl htop nano python3 nodejs mesa-utils

RUN rustup default stable
RUN rustup component add rustfmt
RUN rustup component add clippy