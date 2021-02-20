# Matchbook

A stock exchange

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
