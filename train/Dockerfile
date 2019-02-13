FROM nvcr.io/nvidia/tensorflow:19.01-py3
COPY src/tictactoe/train.py /seraphim
WORKDIR /seraphim
RUN chmod 755 /train.py
ENV SERAPHIM /seraphim
ENTRYPOINT [ "/train.py" ]
