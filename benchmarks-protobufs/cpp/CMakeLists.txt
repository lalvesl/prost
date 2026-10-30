cmake_minimum_required(VERSION 3.16)

project(prost-benchmarks-google CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
# The shim is a shared library, so it must be position independent, and only
# its C API is exported.
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
set(CMAKE_CXX_VISIBILITY_PRESET hidden)
set(CMAKE_VISIBILITY_INLINES_HIDDEN ON)
# Keep the path to an installed libprotobuf in the installed shim
set(CMAKE_INSTALL_RPATH_USE_LINK_PATH ON)

# Prefer an installed protobuf of the same version, for example from a package
# manager. protobuf only matches the full version.
find_package(protobuf ${PROTOBUF_VERSION}.0 EXACT CONFIG QUIET)

if(protobuf_FOUND)
  message(STATUS "Using installed protobuf ${protobuf_VERSION} from ${protobuf_DIR}")
else()
  message(STATUS "Building protobuf ${PROTOBUF_VERSION} from source")

  include(FetchContent)

  FetchContent_Declare(
    protobuf
    GIT_REPOSITORY https://github.com/protocolbuffers/protobuf.git
    GIT_TAG v${PROTOBUF_VERSION}
    GIT_SHALLOW TRUE
  )

  set(ABSL_PROPAGATE_CXX_STD ON)
  set(ABSL_BUILD_TESTING OFF)
  # Only the shim is installed, otherwise the next configure would find this
  # protobuf with `find_package`
  set(protobuf_INSTALL OFF)
  set(protobuf_BUILD_TESTS OFF)
  set(protobuf_BUILD_CONFORMANCE OFF)
  set(protobuf_BUILD_SHARED_LIBS OFF)
  set(protobuf_WITH_ZLIB OFF)
  # Do not pick up a different abseil version installed on the system
  set(protobuf_FORCE_FETCH_DEPENDENCIES ON)

  FetchContent_MakeAvailable(protobuf)
endif()

# Tells build.rs which protoc to use for prost-build and protobuf-codegen
file(GENERATE OUTPUT ${CMAKE_BINARY_DIR}/protoc-path.txt CONTENT $<TARGET_FILE:protobuf::protoc>)

set(GENERATED_DIR ${CMAKE_CURRENT_BINARY_DIR}/generated)
file(MAKE_DIRECTORY ${GENERATED_DIR})

set(GENERATED_SOURCES)
foreach(proto ${BENCHMARK_PROTOS})
  string(REGEX REPLACE "\\.proto$" "" proto_name ${proto})
  set(outputs ${GENERATED_DIR}/${proto_name}.pb.cc ${GENERATED_DIR}/${proto_name}.pb.h)
  add_custom_command(
    OUTPUT ${outputs}
    COMMAND $<TARGET_FILE:protobuf::protoc>
      --proto_path=${BENCHMARK_PROTO_DIR}
      --cpp_out=${GENERATED_DIR}
      ${BENCHMARK_PROTO_DIR}/${proto}
    DEPENDS protobuf::protoc ${BENCHMARK_PROTO_DIR}/${proto}
  )
  list(APPEND GENERATED_SOURCES ${GENERATED_DIR}/${proto_name}.pb.cc)
endforeach()

add_library(prost_benchmarks_google SHARED shim.cc ${GENERATED_SOURCES})
target_include_directories(prost_benchmarks_google PRIVATE ${GENERATED_DIR})
target_link_libraries(prost_benchmarks_google PRIVATE protobuf::libprotobuf)
if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
  # Keep the symbols of protobuf and abseil private, when built from source
  target_link_options(prost_benchmarks_google PRIVATE "LINKER:--exclude-libs,ALL")
endif()

install(TARGETS prost_benchmarks_google LIBRARY DESTINATION lib)
