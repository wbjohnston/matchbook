# Matchbook

A stock exchange

![validate-all-projects workflow status](https://github.com/wbjohnston/matchbook/actions/workflows/validate-all-projects.yml/badge.svg)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

## Development

### Prerequisites

You'll need to install these things before you can start developing

* [Docker](https://docs.docker.com/get-docker/)
* [Docker Compose](https://docs.docker.com/compose/)
* [Rust](https://rustup.rs/)

### Running

You can start the project by running

```bash
docker-compose up
```

If you make code changed, you'll need to rebuild the project using

```bash
docker-compose build
```

Or, to build and start the project at the same time:

```bash
docker-compose up --build
```

This will start the project listening on `localhost:8080`

### Architecture

see [ARCHITECTURE.md](ARCHITECTURE.md)
