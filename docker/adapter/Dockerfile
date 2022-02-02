# Clones the `ic` repository and builds the `ic-btc-adapter`.
FROM ubuntu:20.04 as builder

ARG DEBIAN_FRONTEND=noninteractive
ARG rust_version=1.55
ARG ic_revision=99116f8e872b8765aa609f91eb8c9394914c483d

ENV TZ=UTC

WORKDIR /opt

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone && \
    apt -yq update && \
    apt -yqq install --no-install-recommends curl ca-certificates \
        build-essential pkg-config libssl-dev llvm-dev liblmdb-dev clang cmake \
        git

ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:$PATH

RUN curl --fail https://sh.rustup.rs -sSf \
        | sh -s -- -y --default-toolchain ${rust_version}-x86_64-unknown-linux-gnu --no-modify-path && \
    rustup default ${rust_version}-x86_64-unknown-linux-gnu

ENV PATH=/cargo/bin:$PATH

RUN git clone https://github.com/dfinity/ic.git && \
    cd ic/rs && \
    git checkout ${ic_revision} && \
    cargo build -p ic-btc-adapter --release

# The actual image the user will interact with.
FROM ubuntu:20.04 as release

ARG DEBIAN_FRONTEND=noninteractive
ARG USER=app
ARG UID=3000
ARG GID=3000
ARG BINPATH=/opt/ic/rs/target/release/ic-btc-adapter

WORKDIR /app

RUN groupadd -g $GID -o $USER
RUN useradd -m -u $UID -g $GID -o -s /bin/bash $USER

COPY docker/adapter/regtest.json /etc/regtest.json

COPY --chown=${USER}:${USER} \
        --from=builder [ \
        "${BINPATH}", \
        "./"]

ENV PATH="/app:${PATH}"

CMD ["ic-btc-adapter", "/etc/regtest.json"]
