// C API over the C++ protobuf generated code, used by the Rust benchmarks.
//
// Every function processes a whole dataset, so the cost of crossing the FFI
// boundary is paid once per benchmark iteration, not once per message.

#include <cstddef>
#include <cstdint>
#include <memory>
#include <vector>

#include "google/protobuf/arena.h"
#include "google/protobuf/message_lite.h"

#include "datasets/google_message1/proto2/benchmark_message1_proto2.pb.h"
#include "datasets/google_message1/proto3/benchmark_message1_proto3.pb.h"
#include "datasets/google_message2/benchmark_message2.pb.h"

#define EXPORT extern "C" __attribute__((visibility("default")))

struct Slice {
  const uint8_t* data;
  size_t len;
};

// Must match `Kind` in `src/lib.rs`
enum Kind : uint32_t {
  GOOGLE_MESSAGE1_PROTO2 = 0,
  GOOGLE_MESSAGE1_PROTO3 = 1,
  GOOGLE_MESSAGE2 = 2,
};

namespace {

class Dataset {
 public:
  virtual ~Dataset() = default;
  virtual bool decode(const Slice* payloads, size_t len) = 0;
  virtual bool decode_arena(const Slice* payloads, size_t len) = 0;
  virtual bool merge(const Slice* payloads, size_t len) = 0;
  virtual size_t encode(uint8_t* buf, size_t capacity) const = 0;
  virtual size_t encoded_len() const = 0;
};

template <typename M>
class TypedDataset final : public Dataset {
 public:
  bool init(const Slice* payloads, size_t len) {
    messages_.resize(len);
    for (size_t i = 0; i < len; ++i) {
      if (!messages_[i].ParseFromArray(payloads[i].data, payloads[i].len)) {
        return false;
      }
    }
    return true;
  }

  // A new message for every payload, like `prost::Message::decode`
  bool decode(const Slice* payloads, size_t len) override {
    for (size_t i = 0; i < len; ++i) {
      M message;
      if (!message.ParseFromArray(payloads[i].data, payloads[i].len)) {
        return false;
      }
    }
    return true;
  }

  // A new message for every payload, allocated on an arena that is reused
  bool decode_arena(const Slice* payloads, size_t len) override {
    for (size_t i = 0; i < len; ++i) {
      M* message = google::protobuf::Arena::Create<M>(&arena_);
      bool ok = message->ParseFromArray(payloads[i].data, payloads[i].len);
      arena_.Reset();
      if (!ok) {
        return false;
      }
    }
    return true;
  }

  // One message reused for every payload, like `prost::Message::merge` after
  // `prost::Message::clear`
  bool merge(const Slice* payloads, size_t len) override {
    for (size_t i = 0; i < len; ++i) {
      if (!reused_.ParseFromArray(payloads[i].data, payloads[i].len)) {
        return false;
      }
    }
    return true;
  }

  size_t encode(uint8_t* buf, size_t capacity) const override {
    size_t written = 0;
    for (const M& message : messages_) {
      // `SerializeToArray` would compute the size a second time
      size_t len = message.ByteSizeLong();
      if (len > capacity - written) {
        return 0;
      }
      written = message.SerializeWithCachedSizesToArray(buf + written) - buf;
    }
    return written;
  }

  size_t encoded_len() const override {
    size_t len = 0;
    for (const M& message : messages_) {
      len += message.ByteSizeLong();
    }
    return len;
  }

 private:
  std::vector<M> messages_;
  M reused_;
  google::protobuf::Arena arena_;
};

template <typename M>
Dataset* new_dataset(const Slice* payloads, size_t len) {
  auto dataset = std::make_unique<TypedDataset<M>>();
  if (!dataset->init(payloads, len)) {
    return nullptr;
  }
  return dataset.release();
}

}  // namespace

// Returns null if `kind` is unknown or a payload fails to parse
EXPORT Dataset* google_dataset_new(uint32_t kind, const Slice* payloads,
                                   size_t len) {
  switch (kind) {
    case GOOGLE_MESSAGE1_PROTO2:
      return new_dataset<benchmarks::proto2::GoogleMessage1>(payloads, len);
    case GOOGLE_MESSAGE1_PROTO3:
      return new_dataset<benchmarks::proto3::GoogleMessage1>(payloads, len);
    case GOOGLE_MESSAGE2:
      return new_dataset<benchmarks::proto2::GoogleMessage2>(payloads, len);
    default:
      return nullptr;
  }
}

EXPORT void google_dataset_free(Dataset* dataset) { delete dataset; }

EXPORT bool google_dataset_decode(Dataset* dataset, const Slice* payloads,
                                  size_t len) {
  return dataset->decode(payloads, len);
}

EXPORT bool google_dataset_decode_arena(Dataset* dataset, const Slice* payloads,
                                        size_t len) {
  return dataset->decode_arena(payloads, len);
}

EXPORT bool google_dataset_merge(Dataset* dataset, const Slice* payloads,
                                 size_t len) {
  return dataset->merge(payloads, len);
}

// Returns the number of bytes written, or 0 if `capacity` is too small
EXPORT size_t google_dataset_encode(const Dataset* dataset, uint8_t* buf,
                                    size_t capacity) {
  return dataset->encode(buf, capacity);
}

EXPORT size_t google_dataset_encoded_len(const Dataset* dataset) {
  return dataset->encoded_len();
}
