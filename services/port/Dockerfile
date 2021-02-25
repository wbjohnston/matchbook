FROM rust:1.50-alpine3.12 as builder


RUN apk add --no-cache musl-dev
# WORKDIR /usr/src/

WORKDIR /usr/src/port
COPY packages/ /usr/packages/
RUN cargo init --bin --name port
COPY services/port/Cargo.lock ./Cargo.lock
COPY services/port/Cargo.toml ./Cargo.toml
RUN cargo build --release
# RUN cargo build
RUN rm src/*.rs
# build and cache dependencies 
COPY services/port/src ./src
RUN rm ./target/release/deps/port*
RUN cargo build --release

FROM alpine:3.12
COPY --from=builder /usr/src/port/target/release/port .
EXPOSE 8080
ENTRYPOINT [ "./port"]
