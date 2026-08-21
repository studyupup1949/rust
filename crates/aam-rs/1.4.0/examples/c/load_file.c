/**
 * load_file.c — Demonstrates loading .aam files from disk with aam_load().
 *
 * Covers:
 *   aam_load         — parse a .aam file on disk
 *   aam_last_error   — read the error when a file is missing
 *   aam_find_obj     — forward lookup after file load
 *
 * Build:
 *   See examples/c/Makefile.
 *
 * Run from the repo root so that the relative path "examples/c/config.aam"
 * resolves correctly, OR pass the path as the first argument:
 *
 *   ./build/load_file
 *   ./build/load_file path/to/custom.aam
 */

#include <stdio.h>
#include "../../include/aam.h"

static void print_key(const AamlHandle *h, const char *key)
{
    char *val = aam_find_obj(h, key);
    if (val) {
        printf("  %-15s = %s\n", key, val);
        aam_string_free(val);
    } else {
        printf("  %-15s = <not found>\n", key);
    }
}

int main(int argc, char *argv[])
{
    puts("═══════════════════════════════════════════════════════");
    puts("  aam-rs  C example — load file");
    puts("═══════════════════════════════════════════════════════\n");

    AamlHandle *h = aam_new();
    if (!h) {
        fputs("error: aam_new() returned NULL\n", stderr);
        return 1;
    }

    /* ── 1. Intentional failure: non-existent file ───────────────────────── */
    puts("▶ 1. aam_load() with a non-existent path (expected error)");
    if (aam_load(h, "/no/such/file.aam") != 0) {
        printf("  ✔ error as expected: %s\n\n", aam_last_error(h));
    } else {
        puts("  (unexpected success)\n");
    }

    /* ── 2. Load config.aam from disk ────────────────────────────────────── */
    const char *path = (argc > 1) ? argv[1] : "examples/c/config.aam";
    printf("▶ 2. aam_load(\"%s\")\n", path);

    if (aam_load(h, path) != 0) {
        fprintf(stderr, "  ✘ load failed: %s\n", aam_last_error(h));
        aam_free(h);
        return 1;
    }
    puts("  ✔ loaded successfully\n");

    /* ── 3. Read several keys ────────────────────────────────────────────── */
    puts("▶ 3. Key lookups after file load:");
    print_key(h, "app_name");
    print_key(h, "version");
    print_key(h, "log_level");
    print_key(h, "theme");
    print_key(h, "timeout");
    print_key(h, "nonexistent");
    putchar('\n');

    /* ── 4. Deep lookup through alias chain ──────────────────────────────── */
    puts("▶ 4. aam_find_deep() — alias chain from config.aam:");
    puts("     alias_a → alias_b → alias_c → terminal_value");
    {
        char *val = aam_find_deep(h, "alias_a");
        if (val) {
            printf("  find_deep(\"alias_a\")  →  \"%s\"\n", val);
            aam_string_free(val);
        } else {
            puts("  find_deep(\"alias_a\")  →  (not found)");
        }
    }
    putchar('\n');

    aam_free(h);

    puts("═══════════════════════════════════════════════════════");
    puts("  Done.");
    puts("═══════════════════════════════════════════════════════");
    return 0;
}

