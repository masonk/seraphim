FROM rust:1.32

# ARG HOST_UID
# ARG HOST_GID

# RUN useradd seraphim -s /bin/bash
# RUN usermod -u ${HOST_UID} seraphim
# RUN usermod -g ${HOST_GID} seraphim

# RUN mkdir /cargo
# RUN chown seraphim:seraphim /cargo

# USER seraphim

COPY ./play/bashrc ${HOME}/.bashrc

RUN rustup install nightly
RUN rustup default nightly


