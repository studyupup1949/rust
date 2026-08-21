# examples/c/Makefile — builds C examples against the aam-rs shared library.
#
# Quick start (from repo root):
#   cargo build --release --features ffi
#   make -C examples/c run
#
# Override defaults if needed:
#   make -C examples/c LIB_DIR=/custom/path INC_DIR=/custom/include run

CC      ?= cc
CFLAGS  ?= -std=c11 -Wall -Wextra -g
LIB_DIR ?= ../../target/release
INC_DIR ?= ../../include

BUILD   := build
SRCS    := basic.c load_file.c merge.c
TARGETS := $(SRCS:%.c=$(BUILD)/%)
# $ORIGIN resolves to the directory containing the binary at runtime,
# so the rpath works regardless of the current working directory.
LDFLAGS := -L$(LIB_DIR) -laam_rs -Wl,-rpath,'$$ORIGIN/../../../target/release'

.PHONY: all run clean

all: $(BUILD) $(TARGETS)

$(BUILD):
	mkdir -p $(BUILD)

$(BUILD)/%: %.c
	$(CC) $(CFLAGS) -I$(INC_DIR) $< $(LDFLAGS) -o $@

run: all
	cd ../.. && LD_LIBRARY_PATH=$(abspath $(LIB_DIR)):$$LD_LIBRARY_PATH ./examples/c/build/basic
	cd ../.. && LD_LIBRARY_PATH=$(abspath $(LIB_DIR)):$$LD_LIBRARY_PATH ./examples/c/build/load_file
	cd ../.. && LD_LIBRARY_PATH=$(abspath $(LIB_DIR)):$$LD_LIBRARY_PATH ./examples/c/build/merge

clean:
	rm -rf $(BUILD)

