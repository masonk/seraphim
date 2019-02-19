FROM nvcr.io/nvidia/tensorflow:19.01-py3
ARG HOST_UID
ARG HOST_GID
ARG WHO

# RUN set -eux; \
#     apt-get update; \
#     apt-get install -y --no-install-recommends \
#     autoconf \
#     automake \
#     bzip2 \
#     dpkg-dev \
#     file \
#     g++ \
#     gcc \
#     imagemagick \
#     libbz2-dev \
#     libc6-dev \
#     libcurl4-openssl-dev \
#     libdb-dev \
#     libevent-dev \
#     libffi-dev \
#     libgdbm-dev \
#     libgeoip-dev \
#     libglib2.0-dev \
#     libjpeg-dev \
#     libkrb5-dev \
#     liblzma-dev \
#     libmagickcore-dev \
#     libmagickwand-dev \
#     libncurses5-dev \
#     libncursesw5-dev \
#     libpng-dev \
#     libpq-dev \
#     libreadline-dev \
#     libsqlite3-dev \
#     libssl-dev \
#     libtool \
#     libwebp-dev \
#     libxml2-dev \
#     libxslt-dev \
#     libyaml-dev \
#     make \
#     patch \
#     unzip \
#     xz-utils \
#     zlib1g-dev \
#     \
#     # https://lists.debian.org/debian-devel-announce/2016/09/msg00000.html
#     $( \
#     # if we use just "apt-cache show" here, it returns zero because "Can't select versions from package 'libmysqlclient-dev' as it is purely virtual", hence the pipe to grep
#     if apt-cache show 'default-libmysqlclient-dev' 2>/dev/null | grep -q '^Version:'; then \
#     echo 'default-libmysqlclient-dev'; \
#     else \
#     echo 'libmysqlclient-dev'; \
#     fi \
#     ) \
#     ; \
#     rm -rf /var/lib/apt/lists/*

ENV CARGO_HOME /rust/cargo
ENV RUSTUP_HOME /rust/rustup
ENV PATH=/rust/cargo/bin:$PATH
ENV RUST_VERSION=1.32.0

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
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup install nightly; \
    rustup default nightly; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    groupadd ${WHO} -g ${HOST_GID}; \
    useradd -d /home/${WHO} -ms /bin/bash -g ${WHO} ${WHO}; \
    usermod -u ${HOST_UID} ${WHO}; \
    usermod -g ${HOST_GID} ${WHO}; \
    mkdir /bash; \
    chown -R ${WHO}:${WHO} /bash; \
    mkdir /data; \
    chown -R ${WHO}:${WHO} /data

USER ${WHO}
