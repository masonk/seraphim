FROM nvidia/cuda:9.0-cudnn7-devel-ubuntu16.04
# TF docker (https://github.com/tensorflow/tensorflow/blob/master/tensorflow/tools/dockerfiles/dockerfiles/devel-gpu.Dockerfile)
# uses cuda:10.0-base-ubuntu16.04, but we need full sources 
# because later in this docker we are going to compile Tensorflow from scratch
# using bazel.
# so we aren't prompted to set up the keyboard and other nonsense
ENV DEBIAN_FRONTEND noninteractive 

RUN set -eux; \
    apt-get update && apt-get install -y --no-install-recommends \
    autoconf \
    automake \
    build-essential \
    ca-certificates \
    g++ \
    gcc \
    git \
    less \
    # these were needed by the mnistCUDNN demo
    libfreeimage3 \
    libfreeimage-dev \
    # These two packages allow a TF to be built that run on multiple GPUs
    # libnccl2 libnccl-dev \
    # Seraphim depends on libssl via the openssl-sys crate (which I think is a transitive dep of ctrlc)
    libssl-dev \ 
    linx-tools-generic \
    lldb-7 \
    gdbserver \
    make \
    # Rust libssl crate uses pkg-config to find the openssl system headers
    pkg-config \
    # bazel needs python2 https://github.com/tensorflow/tensorflow/issues/15618
    python \
    # tf needs swig
    swig \
    #just for my own convenience, I like this package
    tree \ 
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

ENV TF_NEED_CUDA 1


WORKDIR /tensorflow
RUN yes '' | ./configure
# https://github.com/gunan/tensorflow-docker/blob/master/gpu-devel/Dockerfile.ubuntu#L64
# This ln-s is necessary when building ... https://github.com/tensorflow/tensorflow/issues/14573#issuecomment-362424509
# Similarly, we need to workaround sandboxing issues:
#   https://github.com/bazelbuild/bazel/issues/418

# Context for this elaborate bazel invocation is here https://github.com/tensorflow/tensorflow/issues/25865
# TLDR: tf <=1.13 links to libcuda.so.1 at build time, but those headers aren't available in the nvidia docker
# at build time. (They are mounted in the running container, but not at build time). So we link to
# stubs to satisfy the build-time linker.
RUN export LD_LIBRARY_PATH=/usr/local/cuda/lib64/stubs:$LD_LIBRARY_PATH && \
    ln -s /usr/local/cuda/lib64/stubs/libcuda.so /usr/local/cuda/lib64/stubs/libcuda.so.1 && \
    bazel build \
    -c opt \
    --config="opt" \
    --copt=-mavx --copt=-mavx2 --copt=-mfma --copt=-mfpmath=both --copt=-msse4.2 \
    --action_env=LD_LIBRARY_PATH=${LD_LIBRARY_PATH} \
    --verbose_failures \
    --spawn_strategy=standalone \
    --genrule_strategy=standalone \
    //tensorflow:libtensorflow.so && \
    rm /usr/local/cuda/lib64/stubs/libcuda.so.1

RUN cp bazel-bin/tensorflow/libtensorflow.so /usr/local/lib

### Rust Nightly 
ENV CARGO_HOME /rust/cargo
ENV RUSTUP_HOME /rust/rustup
ENV PATH=/rust/cargo/bin:$PATH
ENV RUST_VERSION=1.32.0
ENV CARGO_TARGET_DIR=/target

RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
    amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='2d4ddf4e53915a23dda722608ed24e5c3f29ea1688da55aa4e98765fc6223f71' ;; \
    armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='be85f50dc70ee239c5bb6acb60080797841a1e7c45fbf6bae15d6bd4b37ce0e5' ;; \
    arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='454f00a86be75ab070149bac1f541a7b39e5d3383d6da96ad2b929867ed40167' ;; \
    i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='179e3b39f11037a708874e750081f7c0d3e1a6a4c431c2ecee2295acc7b696af' ;; \
    *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.16.0/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    mkdir /target; \
    chmod a+x -R /rust; \
    chmod -R a+w ${RUSTUP_HOME} ; \
    chmod -R a+w ${CARGO_HOME} ; \
    chmod -R a+rwx /target; \
    rustup install nightly; \
    rustup default nightly; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    rustup install nightly; \
    rustup default nightly; \
    rustup show; 

### SERAPHIM ###

ARG HOST_UID
ARG HOST_GID
ARG WHO
ARG SERAPHIM

RUN set -eux; \
    groupadd ${WHO} -g ${HOST_GID}; \
    useradd -d /home/${WHO} -ms /bin/bash -g ${WHO} ${WHO}; \
    usermod -u ${HOST_UID} ${WHO}; \
    usermod -g ${HOST_GID} ${WHO}; \
    mkdir /bash; \
    chown -R ${WHO}:${WHO} /bash; \
    mkdir /data; \
    chown -R ${WHO}:${WHO} /data
