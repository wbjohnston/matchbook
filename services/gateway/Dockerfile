FROM rust:1.50-alpine3.12 as builder

WORKDIR /usr/src/gateway
RUN apk add --no-cache musl-dev
COPY Cargo.toml .
COPY . .
RUN cargo build --release

FROM alpine:3.12
COPY --from=builder /usr/src/gateway/target/release/gateway .
ENTRYPOINT [ "./gateway" ]