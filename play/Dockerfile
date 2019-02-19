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

ENV PYTHON_LIB_PATH /usr/local/lib/python2.7/dist-packages
ENV PYTHONPATH /tensorflow/lib
ENV PYTHON_ARG /tensorflow/lib

ENV GCC_HOST_COMPILER_PATH /usr/bin/gcc
ENV CC_OPT_FLAGS="-march=native"
ENV CUDA_TOOLKIT_PATH /usr/local/cuda
ENV CUDNN_INSTALL_PATH /usr/lib/x86_64-linux-gnu
ENV TF_CUDA_COMPUTE_CAPABILITIES 7.0 
ENV TF_CUDNN_VERSION 7
ENV TF_CUDA_VERSION 10.0
ENV TF_NEED_GCP 0
ENV TF_ENABLE_XLA 0
ENV TF_NEED_OPENCL 0
ENV TF_NEED_CUDA 1
ENV TF_CUDA_VERSION 10.0.0
ENV TF_NEED_HDFS 0
ENV TF_NEED_TENSORRT 0
ENV TF_NCCL_VERSION 2
ENV TF_BUILD_BRANCH master
ENV TF_NEED_ROCM 0
ENV TF_NEED_OPENCL_SYCL 0
ENV TMP /tmp/tensorflow
RUN echo "startup --batch" >>/etc/bazel.bazelrc
RUN ln -s /usr/local/cuda-10.0/compat/libcuda.so.1 /usr/local/cuda/lib64/stubs/libcuda.so.1

ENV LD_LIBRARY_PATH="/usr/local/lib:/usr/local/cuda/lib64:/usr/local/cuda/lib64/stubs:${LD_LIBRARY_PATH}"
RUN find / | grep  libcuda.so.1

WORKDIR /tensorflow
RUN yes '' | ./configure
# https://github.com/gunan/tensorflow-docker/blob/master/gpu-devel/Dockerfile.ubuntu#L64
# This ln-s is necessary when building ... https://github.com/tensorflow/tensorflow/issues/14573#issuecomment-362424509
# Similarly, we need to workaround sandboxing issues:
#   https://github.com/bazelbuild/bazel/issues/418

RUN bazel build \
    -c opt \
    --verbose_failures \
    --spawn_strategy=standalone \
    --genrule_strategy=standalone \
    --action_env=LD_LIBRARY_PATH=${LD_LIBRARY_PATH} \
    //tensorflow:libtensorflow.so

RUN cp bazel-bin/tensorflow/libtensorflow.so /usr/local/lib

RUN set -eux; \
    ### Rust Nightly 1.32
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
    chmod a+x -R /rust; \
    chmod -R a+w ${RUSTUP_HOME} ; \
    chmod -R a+w ${CARGO_HOME} ; \
    rustup install nightly; \
    rustup default nightly; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    rustup install nightly; \
    rustup default nightly; \
    rustup show; \
    ### SERAPHIM ###
    groupadd ${WHO} -g ${HOST_GID}; \
    useradd -d /home/${WHO} -ms /bin/bash -g ${WHO} ${WHO}; \
    usermod -u ${HOST_UID} ${WHO}; \
    usermod -g ${HOST_GID} ${WHO}; \
    mkdir /bash; \
    chown -R ${WHO}:${WHO} /bash; \
    mkdir /data; \
    chown -R ${WHO}:${WHO} /data

USER ${WHO}
