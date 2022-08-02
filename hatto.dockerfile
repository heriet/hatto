FROM rust:1.62.0-bullseye as builder

WORKDIR /work

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    python3-dev \
    lld \
    && apt-get -y clean \
    && rm -rf /var/lib/apt/lists/*

ADD . ./

RUN cargo build --release


FROM python:3.9.13-slim-bullseye

WORKDIR /work

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    python3-dev \
    && apt-get -y clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /work/target/release/hatto /usr/local/bin/hatto

ENTRYPOINT ["hatto"]
