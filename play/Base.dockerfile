FROM nvidia/cuda:9.0-cudnn7-devel-ubuntu16.04
# TF docker (https://github.com/tensorflow/tensorflow/blob/master/tensorflow/tools/dockerfiles/dockerfiles/devel-gpu.Dockerfile)
# uses cuda:10.0-base-ubuntu16.04, but we need full sources 
# because later in this docker we are going to compile Tensorflow from scratch
# using bazel.
ARG HOST_UID
ARG HOST_GID
ARG WHO
ARG SERAPHIM
# so we aren't prompted to set up the keyboard and other nonsense
ENV DEBIAN_FRONTEND noninteractive 
ENV CARGO_HOME /rust/cargo
ENV RUSTUP_HOME /rust/rustup
ENV PATH=/rust/cargo/bin:$PATH
ENV RUST_VERSION=1.32.0

RUN set -eux; \
    apt-get update && apt-get install -y --no-install-recommends \
    autoconf \
    automake \
    build-essential \
    ca-certificates \
    g++ \
    gcc \
    git \
    # these were needed by the mnistCUDNN demo
    libfreeimage3 \
    libfreeimage-dev \
    # These two packages allow a TF to be built that run on multiple GPUs
    # libnccl2 libnccl-dev \
    # Seraphim depends on libssl via the openssl-sys crate (which I think is a transitive dep of ctrlc)
    libssl-dev \ 
    make \
    # Rust libssl crate uses pkg-config to find the openssl system headers
    pkg-config \
    # bazel needs python2 https://github.com/tensorflow/tensorflow/issues/15618
    python \
    # tf needs swig
    swig \
    unzip \
    wget \
    zlib1g-dev; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

# This version has to correspond to the appropriate bazel version
# r1.12 can't be built with bazel 0.19 https://github.com/tensorflow/tensorflow/issues/23401#issuecomment-434681778
# It also can't be built with any higher version of bazel that I tried
# Note to self for the future: r1.13 can be built with bazel 0.21, but not 0.22
ENV BAZEL_VERSION 0.18.0
WORKDIR /
RUN mkdir /bazel && \
    cd /bazel && \
    wget https://github.com/bazelbuild/bazel/releases/download/$BAZEL_VERSION/bazel-$BAZEL_VERSION-installer-linux-x86_64.sh && \
    # curl -fSsL -o /bazel/LICENSE.txt https://raw.githubusercontent.com/bazelbuild/bazel/master/LICENSE.txt && \
    chmod +x bazel-*.sh && \
    ./bazel-$BAZEL_VERSION-installer-linux-x86_64.sh && \
    cd / && \
    rm -f /bazel/bazel-$BAZEL_VERSION-installer-linux-x86_64.sh

RUN echo $(g++ --version)
RUN echo $(bazel version)

### TENSORFLOW 
# rust-tensorflow works with r1.12
# https://github.com/tensorflow/tensorflow/issues/25865
ENV TENSORFLOW_VERSION r1.12
RUN git clone https://github.com/tensorflow/tensorflow.git && \
    cd tensorflow && \
    git checkout ${TENSORFLOW_VERSION}