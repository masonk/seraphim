FROM nvidia/cuda:10.0-cudnn7-devel-ubuntu16.04
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
ENV TENSORFLOW_VERSION r1.13
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
ENV TF_BUILD_BRANCH r1.13
ENV TF_NEED_ROCM 0
ENV TF_NEED_OPENCL_SYCL 0
ENV TMP /tmp/tensorflow
RUN echo "startup --batch" >>/etc/bazel.bazelrc

ENV LD_LIBRARY_PATH="/usr/local/lib:/usr/local/cuda/lib64:/usr/local/cuda/lib64/stubs:${LD_LIBRARY_PATH}"

RUN find / | grep  libcuda.so.1
WORKDIR /tensorflow
RUN yes '' | ./configure
# https://github.com/gunan/tensorflow-docker/blob/master/gpu-devel/Dockerfile.ubuntu#L64
# Is this ln -s is necessary when building? Doesn't seem to help, and the actual libcuda ... https://github.com/tensorflow/tensorflow/issues/14573#issuecomment-362424509
# Similarly, we need to workaround sandboxing issues: https://github.com/bazelbuild/bazel/issues/418
# RUN ln -s /usr/local/cuda/lib64/stubs/libcuda.so /usr/local/cuda/lib64/stubs/libcuda.so.1
# RUN bazel build \
#  --verbose_failures \
#  --spawn_strategy=standalone \
#     --genrule_strategy=standalone \
#     --action_env=LD_LIBRARY_PATH=${LD_LIBRARY_PATH} \
#     //tensorflow:libtensorflow.so

# RUN bazel build --action_env=LD_LIBRARY_PATH=${LD_LIBRARY_PATH} -c opt //tensorflow:libtensorflow.so

# RUN cp bazel-bin/tensorflow/libtensorflow.so /usr/local/lib

