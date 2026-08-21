/**
 * merge.c — Demonstrates layering configs with aam_parse() + aam_merge().
 *
 * Covers:
 *   aam_parse  — initial load (replaces current state)
 *   aam_merge  — merge an additional snippet WITHOUT resetting existing keys
 *   aam_find_obj — verify keys from both layers are accessible
 *
 * Build:
 *   See examples/c/Makefile.
 */

#include <stdio.h>
#include "../../include/aam.h"

static void section(const char *title)
{
    printf("\n▶ %s\n", title);
}

static void print_key(const AamlHandle *h, const char *key)
{
    char *val = aam_find_obj(h, key);
    if (val) {
        printf("  %-20s = %s\n", key, val);
        aam_string_free(val);
    } else {
        printf("  %-20s = <not found>\n", key);
    }
}

int main(void)
{
    puts("═══════════════════════════════════════════════════════");
    puts("  aam-rs  C example — merge / layered config");
    puts("═══════════════════════════════════════════════════════");

    AamlHandle *h = aam_new();
    if (!h) {
        fputs("error: aam_new() returned NULL\n", stderr);
        return 1;
    }

    /* ── Layer 1: base / defaults ─────────────────────────────────────────── */
    const char *base =
        "app_name = my_app\n"
        "log_level = info\n"
        "timeout = 30\n"
        "retry = 3\n"
        "theme = light\n";

    section("1. aam_parse() — base / defaults layer");
    if (aam_parse(h, base) != 0) {
        fprintf(stderr, "  parse error: %s\n", aam_last_error(h));
        aam_free(h);
        return 1;
    }
    puts("  ✔ base layer parsed");

    puts("\n  Keys after base layer:");
    print_key(h, "app_name");
    print_key(h, "log_level");
    print_key(h, "timeout");
    print_key(h, "theme");
    print_key(h, "environment");   /* does not exist yet */

    /* ── Layer 2: environment-specific overrides ─────────────────────────── */
    const char *overrides =
        "log_level = debug\n"      /* override — child wins */
        "theme = dark\n"       /* override */
        "environment = production\n" /* new key */
        "max_conn = 100\n";       /* new key */

    section("2. aam_merge() — environment overrides (child-wins semantics)");
    if (aam_merge(h, overrides) != 0) {
        fprintf(stderr, "  merge error: %s\n", aam_last_error(h));
        aam_free(h);
        return 1;
    }
    puts("  ✔ overrides merged");

    puts("\n  Keys after merge (overrides + new keys visible):");
    print_key(h, "app_name");      /* from base, unchanged */
    print_key(h, "log_level");     /* overridden: info → debug */
    print_key(h, "timeout");       /* from base, unchanged */
    print_key(h, "theme");         /* overridden: light → dark */
    print_key(h, "environment");   /* new key from overrides */
    print_key(h, "max_conn");      /* new key from overrides */
    print_key(h, "retry");         /* from base, unchanged */

    /* ── Layer 3: second merge replaces state only for given keys ────────── */
    const char *patch =
        "timeout = 60\n"            /* patch a single key */
        "feature_x = enabled\n";

    section("3. Second aam_merge() — patch a single key");
    if (aam_merge(h, patch) != 0) {
        fprintf(stderr, "  merge error: %s\n", aam_last_error(h));
        aam_free(h);
        return 1;
    }
    puts("  ✔ patch applied");

    puts("\n  Keys after patch:");
    print_key(h, "timeout");       /* patched: 30 → 60 */
    print_key(h, "feature_x");     /* new */
    print_key(h, "log_level");     /* still debug from layer 2 */
    print_key(h, "app_name");      /* still unchanged from layer 1 */

    /* ── aam_parse() resets everything — contrast with aam_merge() ────────── */
    section("4. aam_parse() resets ALL previous state (contrast with merge)");
    const char *fresh = "brand_new_key = fresh_value\n";
    if (aam_parse(h, fresh) != 0) {
        fprintf(stderr, "  parse error: %s\n", aam_last_error(h));
        aam_free(h);
        return 1;
    }
    puts("  ✔ fresh parse done");

    puts("\n  After aam_parse() — previous keys are gone:");
    print_key(h, "app_name");        /* gone */
    print_key(h, "brand_new_key");   /* present */

    putchar('\n');
    aam_free(h);

    puts("═══════════════════════════════════════════════════════");
    puts("  Done.");
    puts("═══════════════════════════════════════════════════════");
    return 0;
}

