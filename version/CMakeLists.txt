cmake_minimum_required(VERSION 3.19)
project(version C)

message(STATUS ${CMAKE_PROGRAM_PATH})

set(CMAKE_SHARED_LINKER_FLAGS /DEF:${CMAKE_CURRENT_SOURCE_DIR}/version.def)
set(CMAKE_C_STANDARD 99)

add_library(version SHARED version.c version.h)