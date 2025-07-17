FROM rust:1.88.0-alpine AS chef
ENV CARGO_INCREMENTAL=0
RUN apk add --no-cache libc-dev cmake make
RUN cargo install cargo-chef --version 0.1.71 && \
    rm -r $CARGO_HOME/registry
WORKDIR /src

FROM chef AS planner
COPY Cargo.toml Cargo.lock /src/
COPY person /src/person
COPY wallet /src/wallet
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /src/recipe.json /src/recipe.json
RUN cargo chef cook --release \
    --package didemo-person \
    --package didemo-wallet
COPY Cargo.toml Cargo.lock /src/
COPY person /src/person
COPY wallet /src/wallet
ARG GIT_REVISION=unknown
ENV GIT_REVISION=${GIT_REVISION}
RUN cargo build --release \
    --package didemo-person \
    --package didemo-wallet

FROM alpine:3.22.0 AS final
ARG GIT_REVISION=unknown
LABEL revision=${GIT_REVISION}
COPY --from=builder /src/target/release/didemo-person /didemo-person
COPY --from=builder /src/target/release/didemo-wallet /didemo-wallet
ENTRYPOINT ["/didemo-person"]
