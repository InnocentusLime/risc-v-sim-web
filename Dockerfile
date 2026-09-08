FROM krinkin/rv64-toolchain:latest

RUN apt-get update && apt-get install -y curl pkg-config libssl-dev && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
ENV PATH="/root/.cargo/bin:$PATH"

COPY . /app

WORKDIR /app/risc-v-sim
RUN stat Cargo.toml
RUN cargo build --release
RUN cp target/release/risc-v-sim /usr/local/bin/
RUN risc-v-sim --help

WORKDIR /app
RUN cargo build --features "noop_authorization" --release

RUN mkdir -p /app/submission

EXPOSE 3000

ENTRYPOINT ./target/release/risc-v-sim-web \
    --simulator-binary ${SIMULATOR_BINARY} \
    --as-binary ${AS_BINARY} \
    --ld-binary ${LD_BINARY} \
    --codesize-max ${CODESIZE_MAX} \
    --ticks-max ${TICKS_MAX} \
    --submissions-folder ${SUBMISSIONS_FOLDER} \
