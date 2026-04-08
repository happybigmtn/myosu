FROM rust:1.94-bookworm AS builder-base

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        libssl-dev \
        pkg-config \
        protobuf-compiler \
        python3 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32v1-none wasm32-unknown-unknown \
    --toolchain stable-x86_64-unknown-linux-gnu

WORKDIR /work
COPY . .

FROM builder-base AS chain-builder

RUN cargo build --locked --release -p myosu-chain-runtime \
    && SKIP_WASM_BUILD=1 cargo build --locked --release \
        -p myosu-chain \
        --features fast-runtime \
    && strip /work/target/release/myosu-chain

FROM chain-builder AS operator-builder

RUN SKIP_WASM_BUILD=1 cargo build --locked --release \
        -p myosu-miner \
        --bin myosu-miner \
    && SKIP_WASM_BUILD=1 cargo build --locked --release \
        -p myosu-validator \
        --bin myosu-validator \
    && SKIP_WASM_BUILD=1 cargo build --locked --release \
        -p myosu-games-poker \
        --example bootstrap_artifacts \
    && strip /work/target/release/myosu-miner \
    && strip /work/target/release/myosu-validator \
    && /work/target/release/examples/bootstrap_artifacts \
        /work/target/bootstrap/poker/encoder \
        /work/target/bootstrap/poker/query.bin

FROM debian:bookworm-slim AS runtime-base

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bash \
        ca-certificates \
        curl \
        libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --home-dir /home/myosu --shell /bin/bash myosu \
    && mkdir -p /opt/myosu /var/lib/myosu \
    && chown -R myosu:myosu /opt/myosu /var/lib/myosu

USER myosu
WORKDIR /home/myosu

FROM runtime-base AS chain-runtime

COPY --from=chain-builder --chown=myosu:myosu \
    /work/target/release/myosu-chain \
    /usr/local/bin/myosu-chain
COPY --chown=myosu:myosu ops/docker/chain-devnet-entrypoint.sh \
    /usr/local/bin/chain-devnet-entrypoint.sh
RUN chmod +x /usr/local/bin/chain-devnet-entrypoint.sh

ENTRYPOINT ["/usr/local/bin/myosu-chain"]

FROM runtime-base AS miner-runtime

COPY --from=operator-builder --chown=myosu:myosu \
    /work/target/release/myosu-miner \
    /usr/local/bin/myosu-miner
COPY --from=operator-builder --chown=myosu:myosu \
    /work/target/bootstrap \
    /opt/myosu/bootstrap
COPY --chown=myosu:myosu ops/docker/wait-for-rpc.sh \
    /usr/local/bin/wait-for-rpc.sh
COPY --chown=myosu:myosu ops/docker/miner-devnet-entrypoint.sh \
    /usr/local/bin/miner-devnet-entrypoint.sh
RUN chmod +x /usr/local/bin/wait-for-rpc.sh /usr/local/bin/miner-devnet-entrypoint.sh

ENTRYPOINT ["/usr/local/bin/miner-devnet-entrypoint.sh"]

FROM runtime-base AS validator-runtime

COPY --from=operator-builder --chown=myosu:myosu \
    /work/target/release/myosu-validator \
    /usr/local/bin/myosu-validator
COPY --from=operator-builder --chown=myosu:myosu \
    /work/target/bootstrap \
    /opt/myosu/bootstrap
COPY --chown=myosu:myosu ops/docker/wait-for-rpc.sh \
    /usr/local/bin/wait-for-rpc.sh
COPY --chown=myosu:myosu ops/docker/validator-devnet-entrypoint.sh \
    /usr/local/bin/validator-devnet-entrypoint.sh
RUN chmod +x /usr/local/bin/wait-for-rpc.sh /usr/local/bin/validator-devnet-entrypoint.sh

ENTRYPOINT ["/usr/local/bin/validator-devnet-entrypoint.sh"]
