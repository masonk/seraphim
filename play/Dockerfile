FROM baseplay

# COPY play/libcudnn7_7.4.2.24-1+cuda10.0_amd64.deb play/libcudnn7_7.4.2.24-1+cuda10.0_amd64.deb
# COPY play/libcudnn7-dev_7.4.2.24-1+cuda10.0_amd64.deb play/libcudnn7-dev_7.4.2.24-1+cuda10.0_amd64.deb
# COPY play/libcudnn7-doc_7.4.2.24-1+cuda10.0_amd64.deb play/libcudnn7-doc_7.4.2.24-1+cuda10.0_amd64.deb

# RUN dpkg -i play/libcudnn7_7.4.2.24-1+cuda10.0_amd64.deb; \
#     dpkg -i play/libcudnn7-dev_7.4.2.24-1+cuda10.0_amd64.deb; \
#     dpkg -i play/libcudnn7-doc_7.4.2.24-1+cuda10.0_amd64.deb

# RUN set -eux; \
#     apt-get update && apt-get install -y --no-install-recommends \
#     pkg-config 

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


# WORKDIR /tensorflow
# RUN yes '' | ./configure
# RUN bazel build -c opt //tensorflow:libtensorflow.so

# RUN cp bazel-bin/tensorflow/libtensorflow.so /usr/local/lib

# RUN set -eux; \
#     ### Rust Nightly 1.32
#     dpkgArch="$(dpkg --print-architecture)"; \
#     case "${dpkgArch##*-}" in \
#     amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='2d4ddf4e53915a23dda722608ed24e5c3f29ea1688da55aa4e98765fc6223f71' ;; \
#     armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='be85f50dc70ee239c5bb6acb60080797841a1e7c45fbf6bae15d6bd4b37ce0e5' ;; \
#     arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='454f00a86be75ab070149bac1f541a7b39e5d3383d6da96ad2b929867ed40167' ;; \
#     i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='179e3b39f11037a708874e750081f7c0d3e1a6a4c431c2ecee2295acc7b696af' ;; \
#     *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
#     esac; \
#     url="https://static.rust-lang.org/rustup/archive/1.16.0/${rustArch}/rustup-init"; \
#     wget "$url"; \
#     echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
#     chmod +x rustup-init; \
#     ./rustup-init -y --no-modify-path --default-toolchain $RUST_VERSION; \
#     rm rustup-init; \
#     chmod a+x -R /rust; \
#     chmod -R a+w ${RUSTUP_HOME} ; \
#     chmod -R a+w ${CARGO_HOME} ; \
#     rustup install nightly; \
#     rustup default nightly; \
#     rustup --version; \
#     cargo --version; \
#     rustc --version; \
#     rustup install nightly; \
#     rustup default nightly; \
#     rustup show; \
#     ### SERAPHIM ###
#     groupadd ${WHO} -g ${HOST_GID}; \
#     useradd -d /home/${WHO} -ms /bin/bash -g ${WHO} ${WHO}; \
#     usermod -u ${HOST_UID} ${WHO}; \
#     usermod -g ${HOST_GID} ${WHO}; \
#     mkdir /bash; \
#     chown -R ${WHO}:${WHO} /bash; \
#     mkdir /data; \
#     chown -R ${WHO}:${WHO} /data

USER ${WHO}
