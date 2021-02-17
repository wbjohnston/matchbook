FROM rust:1.50-alpine3.12 as builder


RUN apk add --no-cache musl-dev
# WORKDIR /usr/src/
# RUN cargo new port

WORKDIR /usr/src/port
COPY packages/ /usr/packages/
COPY services/port/Cargo.toml services/port/Cargo.lock ./
# build and cache dependencies 
COPY services/port/src/ src/
RUN cargo build --release

FROM alpine:3.12
COPY --from=builder /usr/src/port/target/release/port .
EXPOSE 8080
ENTRYPOINT [ "./port"]
