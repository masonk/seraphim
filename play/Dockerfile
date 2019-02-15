FROM rust:1.32

ARG HOST_UID
ARG HOST_GID
ARG WHO

RUN groupadd ${WHO} -g ${HOST_GID}
RUN useradd -d /home/${WHO} -ms /bin/bash -g ${WHO} ${WHO} 
RUN usermod -u ${HOST_UID} ${WHO}
RUN usermod -g ${HOST_GID} ${WHO}

RUN mkdir /bash
RUN chown -R ${WHO}:${WHO} /bash

RUN mkdir /data
RUN chown -R ${WHO}:${WHO} /data

RUN mkdir /rust
RUN mkdir /rust/rustup
RUN mkdir /rust/cargo
RUN chown -R ${WHO}:${WHO} /rust

ENV CARGO_HOME /rust/cargo
ENV RUSTUP_HOME /rust/rustup

USER ${WHO}

RUN rustup install nightly
RUN rustup default nightly


