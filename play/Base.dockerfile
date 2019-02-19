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

# https://github.com/tensorflow/tensorflow/blob/master/tensorflow/tools/ci_build/Dockerfile.gpu
# https://github.com/tensorflow/tensorflow/blob/master/tensorflow/tools/ci_build/linux/libtensorflow.sh
# https://hub.docker.com/r/bentou/tensorflowgo/dockerfile
# https://hub.docker.com/r/bentou/ubuntuxenialbazel/dockerfile

# RUN set -eux; \
#     apt-get update && apt-get install -y --no-install-recommends \
#     #     autoconf \
#     #     automake \
#     build-essential \
#     ca-certificates \
#     #     curl \
#     g++ \
#     gcc \
#     #     
#     #     libc6-dev \
#     #     libcurl3-dev \
#     #     libfreetype6-dev \
#     #     libpng12-dev \
#     #     libzmq3-dev \
#     #     make \
#     #     netbase \
#     #     pkg-config \
#     python \
#     #     python-dev \
#     #     rsync \
#     #     software-properties-common \
#     #     unzip \
#     wget \
#     #     xz-utils \
#     #     zip \
#     zlib1g-dev; \
#     apt-get clean;  \
#     rm -rf /var/lib/apt/lists/*; 

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


# ### BAZEL - For building libtensorflow from source ###
# RUN set -eux; \
#     add-apt-repository ppa:webupd8team/java; \
#     apt-get update ; \
#     # accept the oracle license
#     echo oracle-java8-installer shared/accepted-oracle-license-v1-1 select true | /usr/bin/debconf-set-selections; \
#     apt-get install -y --no-install-recommends oracle-java8-installer; \
#     apt-get clean; \
#     rm -rf /var/lib/apt/lists/*

# RUN echo "startup --batch" >>/root/.bazelrc
# # Similarly, we need to workaround sandboxing issues:
# #   https://github.com/bazelbuild/bazel/issues/418
# RUN echo "build --spawn_strategy=standalone --genrule_strategy=standalone" \
#     >>/root/.bazelrc
# ENV BAZELRC /root/.bazelrc

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
ENV TENSORFLOW_VERSION r1.13
RUN git clone https://github.com/tensorflow/tensorflow.git && \
    cd tensorflow && \
    git checkout ${TENSORFLOW_VERSION}
