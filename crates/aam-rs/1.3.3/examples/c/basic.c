/**
 * basic.c — Demonstrates the core aam-rs C API.
 *
 * Covers:
 *   aam_new / aam_free
 *   aam_parse        — parse an inline AAML string
 *   aam_find_obj     — forward lookup  (key  → value)
 *   aam_find_key     — reverse lookup  (value → key)
 *   aam_find_deep    — deep chain      (key → value → key → … → terminal)
 *   aam_string_free  — free returned strings
 *   aam_last_error   — read the last error message
 *
 * Build:
 *   See examples/c/Makefile  (or the README).
 */

#include <stdio.h>
#include "../../include/aam.h"

/* ── helpers ──────────────────────────────────────────────────────────────── */

static void print_find_obj(const AamlHandle *h, const char *key)
{
    char *val = aam_find_obj(h, key);
    if (val) {
        printf("  find_obj (\"%s\")  →  \"%s\"\n", key, val);
        aam_string_free(val);
    } else {
        printf("  find_obj (\"%s\")  →  (not found)\n", key);
    }
}

static void print_find_key(const AamlHandle *h, const char *value)
{
    char *key = aam_find_key(h, value);
    if (key) {
        printf("  find_key (\"%s\")  →  \"%s\"\n", value, key);
        aam_string_free(key);
    } else {
        printf("  find_key (\"%s\")  →  (not found)\n", value);
    }
}

static void print_find_deep(const AamlHandle *h, const char *key)
{
    char *val = aam_find_deep(h, key);
    if (val) {
        printf("  find_deep(\"%s\")  →  \"%s\"\n", key, val);
        aam_string_free(val);
    } else {
        printf("  find_deep(\"%s\")  →  (not found)\n", key);
    }
}

/* ── main ─────────────────────────────────────────────────────────────────── */

int main(void)
{
    puts("═══════════════════════════════════════════════════════");
    puts("  aam-rs  C example — basic API");
    puts("═══════════════════════════════════════════════════════\n");

    /* ── 1. Create a handle ──────────────────────────────────────────────── */
    AamlHandle *h = aam_new();
    if (!h) {
        fputs("error: aam_new() returned NULL\n", stderr);
        return 1;
    }

    /* ── 2. Parse an inline AAML snippet ─────────────────────────────────── */
    const char *content =
        "app_name = my_app\n"
        "version = 1.0.0\n"
        "log_level = debug\n"
        "theme = dark\n"
        "debug = dark\n"         /* same value as 'theme' — reverse-lookup demo */
        "timeout = 30\n"
        "alias_a = alias_b\n"      /* chain: alias_a → alias_b → alias_c → end */
        "alias_b = alias_c\n"
        "alias_c = terminal_value\n";

    puts("▶ 1. aam_parse() — inline string");
    if (aam_parse(h, content) != 0) {
        fprintf(stderr, "  parse error: %s\n", aam_last_error(h));
        aam_free(h);
        return 1;
    }
    puts("  ✔ parsed successfully\n");

    /* ── 3. Forward lookup ───────────────────────────────────────────────── */
    puts("▶ 2. aam_find_obj() — forward lookup (key → value)");
    print_find_obj(h, "app_name");
    print_find_obj(h, "version");
    print_find_obj(h, "log_level");
    print_find_obj(h, "missing_key");   /* will be NULL */
    putchar('\n');

    /* ── 4. Reverse lookup ───────────────────────────────────────────────── */
    puts("▶ 3. aam_find_key() — reverse lookup (value → key)");
    print_find_key(h, "my_app");        /* finds "app_name" */
    print_find_key(h, "dark");          /* finds "theme" or "debug" */
    print_find_key(h, "no_such_value");
    putchar('\n');

    /* ── 5. Deep / chain lookup ──────────────────────────────────────────── */
    puts("▶ 4. aam_find_deep() — follows alias chain until terminal");
    puts("     alias_a → alias_b → alias_c → terminal_value");
    print_find_deep(h, "alias_a");
    print_find_deep(h, "alias_b");
    print_find_deep(h, "alias_c");      /* already terminal */
    print_find_deep(h, "app_name");     /* no chain, returns the direct value */
    putchar('\n');

    /* ── 6. Error handling ───────────────────────────────────────────────── */
    puts("▶ 5. aam_last_error() — trigger a parse error");
    if (aam_parse(h, "@import /no/such/file.aam") != 0) {
        printf("  ✔ got expected error: %s\n", aam_last_error(h));
    }
    putchar('\n');

    /* ── 7. Cleanup ──────────────────────────────────────────────────────── */
    aam_free(h);

    puts("═══════════════════════════════════════════════════════");
    puts("  Done.");
    puts("═══════════════════════════════════════════════════════");
    return 0;
}

