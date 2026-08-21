cmake_minimum_required(VERSION 3.10)
project(advantage)

option(BUILD_TESTS "Build unit tests" ON)

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)

# Find dependencies
find_package(Threads REQUIRED)
find_package(Doxygen)

# Move cargo target directory to build dir
set(CARGO_TARGET_DIR ${PROJECT_BINARY_DIR}/target)

set(cargo_flags --features ffi)
if (CMAKE_BUILD_TYPE MATCHES DEBUG)
	set(cargo_library_path ${CARGO_TARGET_DIR}/debug/libadvantage.a)
else ()
	set(cargo_flags ${cargo_flags} --release)
	set(cargo_library_path ${CARGO_TARGET_DIR}/release/libadvantage.a)
endif ()

include(ExternalProject)
ExternalProject_add(advantage_rust
	SOURCE_DIR ${PROJECT_SOURCE_DIR}
	BUILD_IN_SOURCE TRUE
	BUILD_ALWAYS TRUE
	CONFIGURE_COMMAND ""
	BUILD_COMMAND CARGO_TARGET_DIR=${CARGO_TARGET_DIR} cargo +nightly build ${cargo_flags}
	TEST_COMMAND CARGO_TARGET_DIR=${CARGO_TARGET_DIR} cargo +nightly test ${cargo_flags}
	INSTALL_COMMAND ""
)

add_library(advantage SHARED
	src/Context.cpp
	src/Double.cpp
)
add_dependencies(advantage advantage_rust)
target_link_libraries(advantage PRIVATE
	${cargo_library_path}
	${CMAKE_DL_LIBS}
	Threads::Threads
)
target_include_directories(advantage PRIVATE
	${CARGO_TARGET_DIR}
)
target_include_directories(advantage PUBLIC
	${PROJECT_SOURCE_DIR}/include
)

if (BUILD_TESTS)
	enable_testing()
	find_package(GTest REQUIRED)
	add_subdirectory(tests)
endif ()

if (DOXYGEN_FOUND)
	doxygen_add_docs(doc include)
endif ()
