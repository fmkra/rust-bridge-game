To run client:

```
cargo run --bin client
```

To run server:

```
cargo run --bin server
```

### Notes:

`Arc<str>` is used throughout the code to reduce memory usage when immutable strings are cloned many time.
