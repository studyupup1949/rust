#include "ascii.h"
#include <string.h>

// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
size_t ascii_escape_len(char esc_chr, const char *src, size_t len,
                        const char *cntl_chrs, size_t cntl_len) {
  size_t escape_count = 0;
  const char *end = src + len;
  for (; src < end; ++src) {
    char chr = *src;
    if (chr == esc_chr || memchr(cntl_chrs, chr, cntl_len)) {
      escape_count += 1;
    }
  }
  return len + escape_count;
}

void ascii_escape(char esc_chr, const char *src, size_t len, char *dst,
                  const char *cntl_chrs, size_t cntl_len) {
  const char *end = src + len;
  for (; src < end; ++src) {
    char chr = *src;
    if (chr == esc_chr || memchr(cntl_chrs, chr, cntl_len)) {
      *dst++ = esc_chr;
    }
    *dst++ = chr;
  }
}