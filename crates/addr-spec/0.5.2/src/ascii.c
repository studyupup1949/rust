#include "ascii.h"

#define true 1
#define false 0

typedef int bool;

typedef struct {
  const char *first;
  const char *last;
} String;

// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
bool has_chr(String str, char chr) {
  for (; str.first < str.last; ++str.first) {
    if (*str.first == chr) {
      return true;
    }
  }
  return false;
}

// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
size_t ascii_escape_len(char esc_chr, const char *src, size_t len,
                        const char *cntl_set, size_t set_len) {
  String str = {cntl_set, cntl_set + set_len};
  size_t escape_count = 0;
  const char *end = src + len;
  for (; src < end; ++src) {
    char chr = *src;
    if (chr == esc_chr || has_chr(str, chr)) {
      escape_count += 1;
    }
  }
  return len + escape_count;
}

void ascii_escape(char esc_chr, const char *src, size_t len, char *dst,
                  const char *cntl_set, size_t set_len) {
  String str = {cntl_set, cntl_set + set_len};
  const char *end = src + len;
  for (; src < end; ++src) {
    char chr = *src;
    if (chr == esc_chr || has_chr(str, chr)) {
      *dst++ = esc_chr;
    }
    *dst++ = chr;
  }
}