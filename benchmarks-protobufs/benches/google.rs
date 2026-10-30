use std::hint::black_box;

use benchmarks_protobufs::{cpp, load_dataset, prost_benchmarks, upb};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

/// Benchmarks every operation of one dataset, with one function per
/// implementation in each group, so criterion compares them side by side.
fn benchmark_dataset<P, U>(
    criterion: &mut Criterion,
    name: &str,
    dataset: &'static [u8],
    kind: cpp::Kind,
) where
    P: prost::Message + Default,
    U: protobuf::Message,
{
    let payloads = load_dataset(dataset).payload;
    let payload_bytes = payloads.iter().map(Vec::len).sum::<usize>() as u64;

    let prost_messages = payloads
        .iter()
        .map(|payload| P::decode(payload.as_slice()).unwrap())
        .collect::<Vec<_>>();
    let upb_messages = payloads
        .iter()
        .map(|payload| U::parse(payload).unwrap())
        .collect::<Vec<_>>();
    let mut cpp_dataset = cpp::Dataset::new(kind, &payloads).unwrap();

    // All implementations must agree on the encoding before comparing their speed
    let encoded_len = prost_messages.iter().map(P::encoded_len).sum::<usize>();
    assert_eq!(cpp_dataset.encoded_len(), encoded_len);
    assert_eq!(
        upb_messages.iter().map(U::serialized_len).sum::<usize>(),
        encoded_len
    );

    let mut group = criterion.benchmark_group(format!("{name}/decode"));
    group.throughput(Throughput::Bytes(payload_bytes));
    group.bench_function("prost", |b| {
        b.iter(|| {
            for payload in &payloads {
                black_box(P::decode(payload.as_slice()).unwrap());
            }
        })
    });
    group.bench_function("upb", |b| {
        b.iter(|| {
            for payload in &payloads {
                black_box(U::parse(payload).unwrap());
            }
        })
    });
    group.bench_function("cpp", |b| b.iter(|| assert!(cpp_dataset.decode())));
    group.bench_function("cpp_arena", |b| {
        b.iter(|| assert!(cpp_dataset.decode_arena()))
    });
    group.finish();

    // upb is missing here: `clear_and_parse` keeps allocating on the arena of
    // the reused message, so its memory would grow on every iteration.
    let mut group = criterion.benchmark_group(format!("{name}/merge"));
    group.throughput(Throughput::Bytes(payload_bytes));
    group.bench_function("prost", |b| {
        let mut message = P::default();
        b.iter(|| {
            for payload in &payloads {
                message.clear();
                message.merge(payload.as_slice()).unwrap();
                black_box(&message);
            }
        })
    });
    group.bench_function("cpp", |b| b.iter(|| assert!(cpp_dataset.merge())));
    group.finish();

    let mut group = criterion.benchmark_group(format!("{name}/encode"));
    group.throughput(Throughput::Bytes(encoded_len as u64));
    group.bench_function("prost", |b| {
        let mut buf = Vec::with_capacity(encoded_len);
        b.iter(|| {
            buf.clear();
            for message in &prost_messages {
                message.encode(&mut buf).unwrap();
            }
            black_box(&buf);
        })
    });
    // upb can only serialize into a new `Vec`
    group.bench_function("upb", |b| {
        b.iter(|| {
            for message in &upb_messages {
                black_box(message.serialize().unwrap());
            }
        })
    });
    group.bench_function("cpp", |b| {
        let mut buf = Vec::with_capacity(encoded_len);
        b.iter(|| {
            assert!(cpp_dataset.encode(&mut buf));
            black_box(&buf);
        })
    });
    group.finish();

    let mut group = criterion.benchmark_group(format!("{name}/encoded_len"));
    group.bench_function("prost", |b| {
        b.iter(|| black_box(prost_messages.iter().map(P::encoded_len).sum::<usize>()))
    });
    group.bench_function("upb", |b| {
        b.iter(|| black_box(upb_messages.iter().map(U::serialized_len).sum::<usize>()))
    });
    group.bench_function("cpp", |b| b.iter(|| black_box(cpp_dataset.encoded_len())));
    group.finish();
}

macro_rules! dataset {
    ($name:ident, $prost:ty, $upb:ty, $kind:expr) => {
        fn $name(criterion: &mut Criterion) {
            benchmark_dataset::<$prost, $upb>(
                criterion,
                stringify!($name),
                benchmarks_protobufs::dataset::$name(),
                $kind,
            );
        }
    };
}

dataset!(
    google_message1_proto2,
    prost_benchmarks::proto2::GoogleMessage1,
    upb::proto2::GoogleMessage1,
    cpp::Kind::GoogleMessage1Proto2
);
dataset!(
    google_message1_proto3,
    prost_benchmarks::proto3::GoogleMessage1,
    upb::proto3::GoogleMessage1,
    cpp::Kind::GoogleMessage1Proto3
);
dataset!(
    google_message2,
    prost_benchmarks::proto2::GoogleMessage2,
    upb::proto2::GoogleMessage2,
    cpp::Kind::GoogleMessage2
);

criterion_group!(
    google,
    google_message1_proto2,
    google_message1_proto3,
    google_message2,
);

criterion_main!(google);
