FROM rust:1.50-alpine3.12 as builder


RUN apk add --no-cache musl-dev
# WORKDIR /usr/src/
# RUN cargo new gateway

WORKDIR /usr/src/gateway
COPY packages/ /usr/packages/
COPY services/gateway/Cargo.toml services/gateway/Cargo.lock ./
# build and cache dependencies 
COPY services/gateway/src/ src/
RUN cargo build --release

FROM alpine:3.12
COPY --from=builder /usr/src/gateway/target/release/gateway .
EXPOSE 8080
ENTRYPOINT [ "./gateway"]
