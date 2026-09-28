cmake_minimum_required(VERSION 3.22)
project(tektalk_core VERSION 0.1.0 LANGUAGES CXX)

add_library(tektalk_core STATIC
    src/session.cpp
    src/ffi.cpp
)

target_include_directories(tektalk_core PUBLIC include)
target_compile_features(tektalk_core PUBLIC cxx_std_20)
target_compile_options(tektalk_core PRIVATE
    $<$<CXX_COMPILER_ID:Clang,GNU>:-Wall -Wextra -Wpedantic -Werror>
)

enable_testing()
add_executable(tektalk_core_tests tests/session_test.cpp)
target_link_libraries(tektalk_core_tests PRIVATE tektalk_core)
add_test(NAME tektalk_core_tests COMMAND tektalk_core_tests)
