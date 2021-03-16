FROM rust:1.50-alpine3.12 as builder


RUN apk add --no-cache musl-dev
# WORKDIR /usr/src/

WORKDIR /usr/src/retransmitter
COPY packages/ /usr/packages/
RUN cargo init --bin --name retransmitter
COPY services/retransmitter/Cargo.lock ./Cargo.lock
COPY services/retransmitter/Cargo.toml ./Cargo.toml
RUN cargo build --release
# RUN cargo build
RUN rm src/*.rs
# build and cache dependencies 
COPY services/retransmitter/src ./src
RUN rm ./target/release/deps/retransmitter*
RUN cargo build --release

FROM alpine:3.12
COPY --from=builder /usr/src/retransmitter/target/release/retransmitter .
EXPOSE 8080
ENTRYPOINT [ "./retransmitter"]
