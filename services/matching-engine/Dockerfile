FROM rust:1.50-alpine3.12 as builder


RUN apk add --no-cache musl-dev

WORKDIR /usr/src/app
# We need to do this 
COPY packages/ /usr/packages/
COPY services/matching-engine/Cargo.toml services/matching-engine/Cargo.lock ./
# build and cache dependencies 
COPY services/matching-engine/src/ src/
RUN cargo build --release

FROM alpine:3.12
COPY --from=builder /usr/src/app/target/release/matching-engine .
EXPOSE 8080
ENTRYPOINT [ "./matching-engine"]
