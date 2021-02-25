FROM rust:1.50-alpine3.12 as builder


RUN apk add --no-cache musl-dev

WORKDIR /usr/src/app
RUN cargo init --bin --name matching-engine
RUN touch src/lib.rs
COPY packages/ /usr/packages/
COPY services/matching-engine/Cargo.toml services/matching-engine/Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs
COPY services/matching-engine/src/ src/
RUN rm target/release/deps/matching_engine*
RUN rm target/release/deps/libmatching_engine*
RUN cargo build --release

FROM alpine:3.12
COPY --from=builder /usr/src/app/target/release/matching-engine .
EXPOSE 8080
ENTRYPOINT [ "./matching-engine"]
