# Results

Time of each implementation relative to prost (prost = 1.00, lower is faster),
from the second of two consecutive runs of:

```sh
cargo bench --manifest-path benchmarks-protobufs/Cargo.toml
```

Between the two runs, 32 of the 36 benchmarks changed by less than criterion's
noise threshold. The raw output is in [`run-1.txt`](run-1.txt) and
[`run-2.txt`](run-2.txt).

Compared versions: protobuf 36.2 (C++, shared library) and the `protobuf`
crate 4.36.2-release (upb).

## google_message1_proto2

| | prost | upb (C FFI) | C++ | C++ arena |
| --- | --- | --- | --- | --- |
| decode | **1.00** | 1.93 | 1.08 | 1.21 |
| merge | 1.00 | — | **0.65** | — |
| encode | 1.00 | 2.03 | **0.58** | — |
| encoded_len | **1.00** | 10.4 | 1.05 | — |

## google_message1_proto3

| | prost | upb (C FFI) | C++ | C++ arena |
| --- | --- | --- | --- | --- |
| decode | **1.00** | 2.28 | 1.21 | 1.42 |
| merge | **1.00** | — | 1.01 | — |
| encode | 1.00 | 2.45 | **0.94** | — |
| encoded_len | **1.00** | 12.4 | 1.09 | — |

## google_message2

| | prost | upb (C FFI) | C++ | C++ arena |
| --- | --- | --- | --- | --- |
| decode | 1.00 | 1.35 | 1.35 | **0.81** |
| merge | 1.00 | — | **0.43** | — |
| encode | 1.00 | 1.47 | **0.68** | — |
| encoded_len | **1.00** | 10.9 | 1.53 | — |

upb has no `merge` result: its `clear_and_parse` allocates on the arena of the
reused message, which is never freed.
