FROM nvcr.io/nvidia/tensorflow:19.01-py3
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

USER ${WHO}
