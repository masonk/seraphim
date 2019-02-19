FROM nvidia/cuda:10.0-cudnn7-devel-ubuntu16.04
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
    make \
    # bazel needs python2 https://github.com/tensorflow/tensorflow/issues/15618
    python \
    # tf needs swig
    swig \
    unzip \
    wget \
    zlib1g-dev; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

# Tensorflow pukes when you try this with 0.22.0
ENV BAZEL_VERSION 0.21.0
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
# We can't build from the latest stable release (r1.13), because it tries to link libcuda and can't
# https://github.com/tensorflow/tensorflow/issues/25865
ENV TENSORFLOW_VERSION master
RUN git clone https://github.com/tensorflow/tensorflow.git && \
    cd tensorflow && \
    git checkout ${TENSORFLOW_VERSION}
