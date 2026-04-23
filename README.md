<!--
Local development notes and usage for m-cc.
-->

# m-cc

`m-cc` is a small topic-based message queue service with a simple HTTP API and
per-topic, per-consumer-group offsets.

## Current functionality

- Topic-based append-only logs on disk.
- `POST /push` to append messages for a topic.
- `GET /pull` to fetch the next batch of messages for a consumer group.
- `GET /metrics` to inspect simple counters (`received`, `delivered`).
- Optional runtime config via YAML file; defaults are loaded from
  `./config/default.yaml` when no file is supplied.

Message format for `push`/`pull` payloads:

- Each message is encoded as: `u32` big-endian length + raw message bytes.
- `pull` returns the same framed format, so clients can decode batch responses
  deterministically.

## Run the server

From repo root:

```bash
cargo run --release
```

If you want to use a custom config file:

```bash
cargo run --release -- --config config.yaml
```

Config keys:

- `server.address`
- `server.port`
- `runtime.worker_threads`
- `storage.data_dir`
- `writer.max_batch_commands`
- `writer.flush_bytes`
- `writer.flush_interval_ms`

Default config file path is `./config/default.yaml`.

The server binds to `server.address:server.port` and serves:

- `POST /push?topic=<topic>`
- `GET /pull?topic=<topic>&consumer_id=<consumer>&batch=<n>`
- `GET /metrics`

## Run the load test crate

From repo root, with the server running first:

```bash
cargo run --release --manifest-path load-test/Cargo.toml
```

Useful options for the load test:

- `--address <host>` target server address (default: `localhost`)
- `--port <port>` target server port (default: `3000`)
- `--feature` sends minimum-size messages for feature-level verification.

Example:

```bash
cargo run --release --manifest-path load-test/Cargo.toml -- --feature
```

Example with custom target:

```bash
cargo run --release --manifest-path load-test/Cargo.toml -- --address 127.0.0.1 --port 4000
```

Notes:

- The load test targets `http://<address>:<port>` and defaults to `http://localhost:3000`.
- It uses `NUM_TOPICS`, `NUM_CONSUMER_GROUPS`, producer/consumer worker counts, and
  batch sizes defined in `load-test/src/main.rs`. Adjust those constants there.

