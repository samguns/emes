FROM rust:1.85.0-slim

RUN sed -i "s@http://.*deb.debian.org@https://repo.huaweicloud.com@g" /etc/apt/sources.list.d/debian.sources

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y python3-full python3-dev python3-pip gcc vim libprotobuf-dev zlib1g zlib1g-dev libsm6 \
        libgl1 libglib2.0-0 cmake libssl-dev pkg-config ffmpeg \
        && apt-get clean \
        && rm -rf /var/lib/apt/lists/* \
        && rm /usr/lib/python3.11/EXTERNALLY-MANAGED

RUN pip config --user set global.index https://mirrors.huaweicloud.com/repository/pypi
RUN pip config --user set global.index-url https://mirrors.huaweicloud.com/repository/pypi/simple
RUN pip config --user set global.trusted-host mirrors.huaweicloud.com

RUN pip3 install cattr numpy spidev pendulum librosa

RUN mkdir .cargo > /dev/null 2>&1 || true && \
    ( \
    echo "[source.crates-io]"; \
    echo "replace-with = 'rsproxy-sparse'"; \
    echo ; \
    echo "[source.rsproxy-sparse]"; \
    echo "registry = \"sparse+https://rsproxy.cn/index/\""; \
    echo ; \
    echo "[build]"; \
    echo "jobs = 4"; \
    ) > .cargo/config.toml

CMD ["/bin/bash"]
