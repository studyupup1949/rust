cmake_minimum_required(VERSION 3.10)
project(AAM_C_Examples C)

find_package(aam-c REQUIRED)

add_executable(basic basic.c)
target_link_libraries(basic PRIVATE aam-c::aam-c)

add_executable(load_file load_file.c)
target_link_libraries(load_file PRIVATE aam-c::aam-c)

add_executable(merge merge.c)
target_link_libraries(merge PRIVATE aam-c::aam-c)