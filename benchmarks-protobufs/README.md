# `benchmarks-protobufs`

Compares prost with Google's protobuf implementations, on the datasets of
`third_party/old_protobuf_benchmarks`:

- `prost`
- `upb` (C FFI): Google's official Rust crate ([`protobuf`](https://crates.io/crates/protobuf) 4.x), backed by the upb C library
- `cpp`: Google's C++ library, called through the C API in `cpp/shim.cc`
- `cpp_arena`: the same, with messages allocated on a reused `Arena`

Each dataset has one criterion group per operation:

| Group         | prost                  | upb (C FFI)         | C++                                              |
| ------------- | ---------------------- | ------------------- | ------------------------------------------------ |
| `decode`      | `Message::decode`      | `Parse::parse`      | `ParseFromArray` on a new message                |
| `merge`       | `clear` + `merge`      | -                   | `ParseFromArray` on a reused message             |
| `encode`      | `encode` to a reused `Vec` | `serialize` to a new `Vec` | `SerializeWithCachedSizesToArray` to a reused buffer |
| `encoded_len` | `encoded_len`          | `serialized_len`    | `ByteSizeLong`                                   |

upb has no `merge` benchmark: `clear_and_parse` allocates on the arena of the
reused message, which is never freed, so memory grows on every iteration.

The C++ functions process the whole dataset in one call, so the FFI overhead
is paid once per iteration.

## Running

```sh
cargo bench --manifest-path benchmarks-protobufs/Cargo.toml
```

This crate is not part of the prost workspace, and is not built by CI.

It needs CMake and a C++17 compiler. protoc and libprotobuf must be the version
that matches the `protobuf` crate (`PROTOBUF_VERSION` in `build.rs`):

- If CMake finds an installed protobuf of exactly that version, it is used.
- Otherwise protobuf is built from source in the target directory. The first
  build takes several minutes, and is repeated for each cargo profile.

To update protobuf, change `PROTOBUF_VERSION` in `build.rs` and the version of
the `protobuf` and `protobuf-codegen` crates in `Cargo.toml` together.
