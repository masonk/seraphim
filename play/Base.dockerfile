FROM seraphim_play

RUN set -eux; \
    apt-get update && apt-get install -y --no-install-recommends \
    gdbserver \
    rust-lldb \
    lldb-4.0