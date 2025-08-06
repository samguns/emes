cmake_minimum_required(VERSION 3.14)
project(qwen2_vl_cpp)

set(CMAKE_CXX_STANDARD 17)

add_library(qwen2_vl_cpp STATIC
    src/qwen2.cpp)

install(TARGETS qwen2_vl_cpp
    DESTINATION lib)
