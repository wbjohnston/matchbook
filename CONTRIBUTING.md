# Contributing to Matchbook

Thanks for your interest in the project!

**No contribution is too small and all contributions are valued.**

## Code of conduct

This repository conforms to the [Rust code of conduct](https://www.rust-lang.org/policies/code-of-conduct)

## Architecture

![Matchbook Architecture](./doc/img/architecture.svg)

The architecture of matchbook is **heavily** inspired by a the Brian Nigito's talk ["How to Build an Exchange"](https://www.youtube.com/watch?v=b1e4t2k2KJY).

### Services

Matchbook is composed of the following services. Every service is indirectly connected to eachother using [multicast](https://en.wikipedia.org/wiki/Multicast).

#### [Port](./services/port)

Entrypoint for trading clients. Provides a [FIX](https://en.wikipedia.org/wiki/Financial_Information_eXchange) interface for clients.

#### [Matching Engine](./services/matching-engine)

Matches customer orders and reports the result to the Matchbook network.

#### [Retransmitter](./services/retransmitter)

Provides a degree of network durability. Services aren't connected via a reliable transport protocol. In order for Matchbook to recover from transmission errors, the retransmitter listens for all messages and will retransmit any known message.

#### 🔨 Cancel Fairy (Not implemented)

To offload tracking order cancellations scheduled in the future, the Cancel Fairy will listen to cancel later messages and emit them as cancel now messages when they are scheduled to be canceled.

#### 🔨 Drop (Not implemented)

Stream for clearing houses to use to clear orders.

#### 🔨 Market Data (Not implemented)

Stream of anonymized market data.

#### 🔨 Passive Matching Engine (Not implemented)

Backup matching engine. Listens to messages emitted by the matching engine to maintain state parity. In the case of the matching engine going offline, the passive matching engine will be promoted to the primary matching engine.

### Packages

#### [matchbook-types](./packages/matchbook-types)

Provides the messages that are passed between matchbook services.

#### [matchbook-util](./packages/matchbook-util)

Provides code shared between matchbook services.

#### [fixer-upper](./packages/fixer-upper)

Provides a custom FIX implementation. Currently only provides FIX JSON message parsing.
