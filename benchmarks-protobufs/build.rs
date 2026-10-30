use std::env;
use std::fs;
use std::path::PathBuf;

// Google protobuf version used for the C++ library and for protoc. It must
// match the version of the `protobuf` crate (upb), see Cargo.toml.
const PROTOBUF_VERSION: &str = "36.2";

static DATASET_PROTOS: &[&str] = &[
    "datasets/google_message1/proto2/benchmark_message1_proto2.proto",
    "datasets/google_message1/proto3/benchmark_message1_proto3.proto",
    "datasets/google_message2/benchmark_message2.proto",
];

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_dir = manifest_dir
        .join("..")
        .join("third_party")
        .join("old_protobuf_benchmarks")
        .canonicalize()
        .unwrap();

    // Builds the C API in `cpp/shim.cc` as a shared library, against an
    // installed protobuf if CMake finds one, otherwise against protobuf built
    // from source. Always in release mode: a debug build of libprotobuf would
    // not be a fair comparison.
    let dst = cmake::Config::new("cpp")
        .profile("Release")
        .define("PROTOBUF_VERSION", PROTOBUF_VERSION)
        .define("BENCHMARK_PROTO_DIR", &proto_dir)
        .define("BENCHMARK_PROTOS", DATASET_PROTOS.join(";"))
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .build();

    let lib_dir = dst.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=prost_benchmarks_google");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    println!("cargo:rerun-if-changed=cpp");
    println!("cargo:rerun-if-env-changed=CMAKE_PREFIX_PATH");

    let protoc = fs::read_to_string(dst.join("build").join("protoc-path.txt")).unwrap();
    let protoc = PathBuf::from(protoc.trim());

    let mut benchmark_protos = vec![proto_dir.join("benchmarks.proto")];
    benchmark_protos.extend(DATASET_PROTOS.iter().map(|proto| proto_dir.join(proto)));
    prost_build::Config::new()
        .protoc_executable(&protoc)
        .compile_protos(&benchmark_protos, &[&proto_dir])
        .unwrap();

    protobuf_codegen::CodeGen::new()
        .protoc_path(&protoc)
        .include(&proto_dir)
        .inputs(DATASET_PROTOS)
        .generate_and_compile()
        .unwrap();
}
