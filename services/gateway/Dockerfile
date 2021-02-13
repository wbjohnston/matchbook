FROM rust:1.50-alpine3.12 as builder

WORKDIR /usr/src/gateway
RUN apk add --no-cache musl-dev
RUN cargo init . --name=gateway
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo fetch
COPY src/ src/ 
RUN cargo build --release

FROM alpine:3.12
COPY --from=builder /usr/src/gateway/target/release/gateway .
ENTRYPOINT [ "./gateway" ]