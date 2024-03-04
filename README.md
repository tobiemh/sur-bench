# sur-bench

How to use?

```bash
cargo run -r -- --help
```

```bash
Usage: sur-bench [OPTIONS] --database <DATABASE> --samples <SAMPLES> --threads <THREADS>

Options:
  -i, --image <IMAGE>        Docker image
  -d, --database <DATABASE>  Database [possible values: dry, surrealdb, surrealdb-memory, surrealdb-rocksdb, surrealdb-speedb, mongodb, postgresql]
  -s, --samples <SAMPLES>    Number of samples
  -t, --threads <THREADS>    Number of concurrent threads
  -h, --help                 Print help
```

Run the benchmark without interaction with any database:

```bash
cargo run -r -- -d dry -s 100000 -t 3
```

Run the benchmark against Postgres:

```bash
cargo run -r -- -d postgresql -s 100000 -t 3
```

Run the benchmark against Postgres:

```bash
cargo run -r -- -d surrealdb-memory -s 100000 -t 3
```

Run the benchmark against SurreadDB with RocksDB:

```bash
cargo run -r -- -d surrealdb-rocksdb -s 100000 -t 3
```

Run the benchmark against SurrealDB with Memory:

```bash
cargo run -r -- -d surrealdb-memory -s 100000 -t 3
```

Run the benchmark against an already running SurrealDB instance:

```bash
cargo run -r -- -d surrealdb -s 100000 -t 3
```