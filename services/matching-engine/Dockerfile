FROM rust:1.50-alpine3.12 as builder


RUN apk add --no-cache musl-dev
# WORKDIR /usr/src/
# RUN cargo new gateway



WORKDIR /usr/src/app
# We need to do this 
COPY Cargo.toml Cargo.lock ./
# build and cache dependencies 
COPY src/ src/
RUN cargo build --release

FROM alpine:3.12
COPY --from=builder /usr/src/app/target/release/matching-engine .
EXPOSE 8080
ENTRYPOINT [ "./matching-engine"]
