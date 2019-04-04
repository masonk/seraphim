FROM ubuntu:18.04
# using 18.04 to get a higher version of lldb
# lldb-4 (highest in 16.04) has a broken lldb-server


# This version has to correspond to the appropriate bazel version
# r1.12 can't be built with bazel 0.19 https://github.com/tensorflow/tensorflow/issues/23401#issuecomment-434681778
# It also can't be built with any higher version of bazel that I tried
# Note to self for the future: r1.13 can be built with bazel 0.21, but not 0.22
RUN set -eux; \
    apt-get update && apt-get install -y --no-install-recommends \
    autoconf \
    automake \
    build-essential \
    # wget over https
    ca-certificates \
    g++ \
    gcc \
    # pull tensorflow
    git \
    # convenience
    less \
    # these were needed by the mnistCUDNN demo
    libfreeimage3 \
    libfreeimage-dev \
    # These two packages allow a TF to be built that run on multiple GPUs
    # libnccl2 libnccl-dev \
    # for `perf`
    linux-tools-generic \
    # Seraphim depends on libssl via the openssl-sys crate (which I think is a transitive dep of ctrlc)
    libssl-dev \ 
    # debugger
    lldb-7 \
    gdbserver \
    make \
    # Rust libssl crate uses pkg-config to find the openssl system headers
    pkg-config \
    # rust wrapper around lldb that formats code nicely
    rust-lldb \
    # tf needs swig
    #just for my own convenience, I like this package
    tree \ 
    unzip \
    wget \
    zlib1g-dev; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

RUN ln -s /usr/bin/lldb-7 /usr/bin/lldb
RUN ln -s /usr/bin/lldb-server-7 /usr/bin/lldb-server

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
