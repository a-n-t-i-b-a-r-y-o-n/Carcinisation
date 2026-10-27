ARG DEPENDENCIES=""

# ===============================

FROM docker.io/arm64v8/rust:trixie AS base

# Remove python from the image for space reasons
RUN apt remove -y python3.13-minimal

# Upgrade dependencies
RUN apt update && apt upgrade -y

# ===============================

FROM base AS build-convergence

WORKDIR /

# Install protoc
RUN apt install -y protobuf-compiler
RUN apt autoremove -y

# Clone workspace repo
COPY . /carcinisation
WORKDIR /carcinisation

# Build the plugin packager
RUN cargo build -r --package convergence --bin convergence

# Publish to output directory
RUN mkdir -p /output
RUN mv -v /carcinisation/target/release/convergence /output/