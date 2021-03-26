# Matchbook

A toy stock exchange written to learn about Exchange technology and architecture. Inspired by [Brian Nigito's talk "How to Build an Exchange"](https://www.youtube.com/watch?v=b1e4t2k2KJY).

Matchbook accepts [Financial Information eXchange](https://en.wikipedia.org/wiki/Financial_Information_eXchange) (FIX) messages from incoming clients on TCP port `8080`.

[![Build status](https://github.com/wbjohnston/matchbook/actions/workflows/ci.yml/badge.svg)](https://github.com/wbjohnston/matchbook/actions?query=branch%3Amain)
[![dependency status](https://deps.rs/repo/github/wbjohnston/matchbook/status.svg)](https://deps.rs/repo/github/wbjohnston/matchbook)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

## Features

1. Limit order matching
2. Custom FIX implmentation
3. FIX sessions secured with TLS
4. Distributed architecture

## [Usage](./USER_GUIDE.md)

provided you have [docker compose](https://docs.docker.com/compose/install/) installed, you can start matchbook using:

```shell
docker-compose up
```

matchbook will start listening on `localhost:8080` for incoming TCP connections.

for a tutorial on how to interact with matchbook, check out [the user guide](./USER_GUIDE.md)

## Contributing

Interested in contributing? check out the [contributing guide](./CONTRIBUTING.md).

## License

This project is license under the [GNU GPLv3 Licence](./LICENSE)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Matchbook by you, shall be licensed as GNU GPLv3, without any additional terms or conditions.
