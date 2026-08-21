/* aam-rs C API
 *
 * Memory ownership rules:
 * - Handles created by `aam_new()` must be destroyed with `aam_free()`.
 * - Strings returned by `aam_find_*()` are heap-allocated by Rust and must be
 *   released with `aam_string_free()`.
 * - `aam_last_error()` returns an internal pointer owned by the handle;
 *   do not free it.
 */

#ifndef AAM_RS_AAM_H
#define AAM_RS_AAM_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct AamHandle AamHandle;


AamHandle *aam_new(void);
void aam_free(AamHandle *handle);

int aam_parse(AamHandle *handle, const char *content);
int aam_load(AamHandle *handle, const char *path);
char *aam_format(AamHandle *handle, const char *content);

char *aam_get(const AamHandle *handle, const char *key);
char *aam_find(const AamHandle *handle, const char *query);
char *aam_deep_search(const AamHandle *handle, const char *pattern);
char *aam_reverse_search(const AamHandle *handle, const char *value);

char *aam_schema_names(const AamHandle *handle);
char *aam_type_names(const AamHandle *handle);

void aam_string_free(char *s);
const char *aam_last_error(const AamHandle *handle);

#ifdef __cplusplus
}
#endif

#endif

