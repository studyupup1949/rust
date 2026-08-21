/**
 * @file aam.h
 * @brief C bindings for aam-rs — the AAML configuration parser.
 *
 * Build the shared library first:
 *   cargo build --release --features ffi
 *
 * Then compile your C program:
 *   cc my_program.c -I include -L target/release -l aam_rs -o my_program
 *
 * Memory rules
 * ------------
 * - Strings returned by aam_find_*() are heap-allocated. Free them with
 *   aam_string_free() — do NOT use free() from <stdlib.h>.
 * - The AamlHandle must be destroyed with aam_free() when no longer needed.
 * - The pointer returned by aam_last_error() is owned by the handle and is
 *   only valid until the next API call on that handle. Do NOT free it.
 */

#ifndef AAM_H
#define AAM_H

#ifdef __cplusplus
extern "C" {
#endif

/* ── Opaque handle ──────────────────────────────────────────────────────────*/

/**
 * Opaque handle to an AAML parser instance.
 *
 * Always create via aam_new() and destroy via aam_free().
 */
typedef struct AamlHandle AamlHandle;

/* ── Lifecycle ──────────────────────────────────────────────────────────────*/

/**
 * Creates a new AAML handle with all default commands registered.
 *
 * @return A newly allocated handle, or NULL on allocation failure.
 */
AamlHandle *aam_new(void);

/**
 * Frees an AAML handle previously created by aam_new().
 *
 * No-op when handle is NULL.
 *
 * @param handle  Handle to free. Must not be used after this call.
 */
void aam_free(AamlHandle *handle);

/* ── Parsing ────────────────────────────────────────────────────────────────*/

/**
 * Parses AAML content from a null-terminated string and replaces the current
 * state of the handle.
 *
 * @param handle   A valid AamlHandle created by aam_new().
 * @param content  Null-terminated UTF-8 AAML source.
 * @return 0 on success, -1 on error (see aam_last_error()).
 */
int aam_parse(AamlHandle *handle, const char *content);

/**
 * Loads an AAML file from disk and replaces the current state of the handle.
 *
 * @param handle  A valid AamlHandle.
 * @param path    Null-terminated path to the .aam file.
 * @return 0 on success, -1 on error (see aam_last_error()).
 */
int aam_load(AamlHandle *handle, const char *path);

/**
 * Parses AAML content and *merges* it into the existing state of the handle.
 * Use this to layer multiple .aam snippets without resetting state.
 *
 * @param handle   A valid AamlHandle.
 * @param content  Null-terminated UTF-8 AAML source.
 * @return 0 on success, -1 on error (see aam_last_error()).
 */
int aam_merge(AamlHandle *handle, const char *content);

/* ── Lookup ─────────────────────────────────────────────────────────────────*/

/**
 * Looks up key in the AAML map.
 *
 * First performs a forward lookup (key → value), then a reverse lookup
 * (searches for an entry whose *value* equals key).
 *
 * @param handle  A valid AamlHandle.
 * @param key     Null-terminated key string.
 * @return A newly allocated C string (free with aam_string_free()), or NULL.
 */
char *aam_find_obj(const AamlHandle *handle, const char *key);

/**
 * Reverse lookup — finds the key whose stored value equals value.
 *
 * @param handle  A valid AamlHandle.
 * @param value   Null-terminated value string to search for.
 * @return A newly allocated C string (free with aam_string_free()), or NULL.
 */
char *aam_find_key(const AamlHandle *handle, const char *value);

/**
 * Deep lookup — follows the chain key → value → key until a terminal value
 * is reached or a cycle is detected.
 *
 * @param handle  A valid AamlHandle.
 * @param key     Starting key.
 * @return A newly allocated C string (free with aam_string_free()), or NULL.
 */
char *aam_find_deep(const AamlHandle *handle, const char *key);

/* ── Memory management ──────────────────────────────────────────────────────*/

/**
 * Frees a C string returned by aam_find_obj(), aam_find_key(), or
 * aam_find_deep().
 *
 * No-op when s is NULL.
 *
 * @param s  String to free. Must not be used after this call.
 */
void aam_string_free(char *s);

/* ── Error reporting ────────────────────────────────────────────────────────*/

/**
 * Returns a pointer to the last error message stored in handle, or NULL if
 * the previous operation succeeded.
 *
 * The returned pointer is owned by handle and is valid only until the next
 * API call on that handle. Do NOT pass it to aam_string_free().
 *
 * @param handle  A valid AamlHandle.
 * @return Null-terminated error string, or NULL on success.
 */
const char *aam_last_error(const AamlHandle *handle);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* AAM_H */

