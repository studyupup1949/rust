// Flattened C entry points over Accelerate's sparse direct solvers.
//
// Accelerate's own SparseFactor / SparseSolve / SparseRefactor / SparseCleanup are Clang
// `__attribute__((overloadable))` functions, mostly `static inline`, that take and return structs
// by value. Nothing on the Rust side can call them. Compiling this translation unit with Clang
// resolves the overloads and the inline dispatch; the functions declared here are ordinary
// `extern "C"` symbols whose ABI is scalars and pointers only.
//
// Conventions used throughout:
//
//   - Every fallible entry point reports its outcome as an `int` status, either returned or
//     written through an out-parameter. No status ever crosses the boundary as a struct field,
//     and a NULL return is always accompanied by a status explaining it.
//   - Matrix attributes are passed as separate semantic selectors rather than as the raw bits of
//     `SparseAttributes_t`. That type is a bitfield whose layout is implementation-defined, so
//     assembling it on the Rust side would rest on an assumption no assertion can check; built
//     here by named field, the C compiler checks it instead.
//   - Handles are heap-boxed and opaque. The caller only ever holds a pointer.
//   - Everything downstream of the analysis is per-element-type, spelled with a `_d` suffix for
//     `double` and `_f` for `float`. The symbolic phase has no suffix: one analysis serves both,
//     which is Accelerate's own arrangement rather than a simplification made here.

#ifndef ACCELERATE_SPARSE_SHIM_H
#define ACCELERATE_SPARSE_SHIM_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Status codes. These are Accelerate's own values, checked against the SDK by _Static_assert in
// shim.c, plus one that is ours.
#define ACCSP_STATUS_OK                    0
#define ACCSP_STATUS_FACTORIZATION_FAILED (-1)
#define ACCSP_STATUS_MATRIX_IS_SINGULAR   (-2)
#define ACCSP_STATUS_INTERNAL_ERROR       (-3)
#define ACCSP_STATUS_PARAMETER_ERROR      (-4)
// Not an Accelerate code: reported when a handle could not be allocated, which happens before
// Accelerate is reached and so has no status of its own.
#define ACCSP_STATUS_ALLOCATION_FAILED    (-100)
// Also ours: a solve was asked of a factorization whose last attempt failed. Accelerate refuses
// this through its error callback rather than with a distinct code, and the code it does leave
// behind is the one describing the original failure, which is a different fact.
#define ACCSP_STATUS_NOT_FACTORED         (-101)
// Also ours: an operation the running OS, or the SDK this was built against, is too old to
// provide. Two cases, guarded for different reasons. LU below macOS 15.5 is inlined dispatch that
// would trap rather than return, so the shim intercepts it before the call. SparseGetInertia
// below macOS 13.0 is an ordinary exported symbol, which would be weakly linked and null; the
// guard keeps it from being called at all.
#define ACCSP_STATUS_UNSUPPORTED_OS       (-102)

// Factorization selectors. Accelerate's own values; the gaps are the kinds not yet wired up.
#define ACCSP_KIND_CHOLESKY        0
#define ACCSP_KIND_LDLT_UNPIVOTED  2
#define ACCSP_KIND_LDLT_SBK        3
#define ACCSP_KIND_LDLT_TPP        4
// LU of a general square matrix, needing the macOS 15.5 SDK to build and that OS to run. The
// values are Apple's and are declared unconditionally so the API does not change shape with the
// SDK; the drift checks and the OS guard for them are compiled only under ACCSP_HAVE_LU.
#define ACCSP_KIND_LU_UNPIVOTED   81
#define ACCSP_KIND_LU_SPP         82
#define ACCSP_KIND_LU_TPP         83
#define ACCSP_KIND_QR             40
#define ACCSP_KIND_CHOLESKY_ATA   41

// Subfactor selectors use Accelerate's values. Availability depends on the factorization kind and,
// for scaling, whether scaling was applied; Accelerate reports unsupported requests. SR and SC
// arrived with LU, so their drift checks are compiled only under ACCSP_HAVE_LU. Their values remain
// declared unconditionally because Accelerate accepts the selector as a plain integer and reports
// an unknown value without dispatching to an unavailable symbol.
#define ACCSP_SUBFACTOR_INVALID  0
#define ACCSP_SUBFACTOR_P        1
#define ACCSP_SUBFACTOR_S        2
#define ACCSP_SUBFACTOR_L        3
#define ACCSP_SUBFACTOR_D        4
#define ACCSP_SUBFACTOR_PLPS     5
#define ACCSP_SUBFACTOR_Q        6
#define ACCSP_SUBFACTOR_R        7
#define ACCSP_SUBFACTOR_RP       8
#define ACCSP_SUBFACTOR_SR       9
#define ACCSP_SUBFACTOR_SC      10

// Matrix kind and triangle selectors, again Accelerate's own values.
#define ACCSP_MATRIX_ORDINARY   0
#define ACCSP_MATRIX_SYMMETRIC  3
#define ACCSP_TRIANGLE_UPPER    0
#define ACCSP_TRIANGLE_LOWER    1

// Ordering selectors for the symbolic phase.
#define ACCSP_ORDER_DEFAULT  0
#define ACCSP_ORDER_USER     1
#define ACCSP_ORDER_AMD      2
#define ACCSP_ORDER_METIS    3
#define ACCSP_ORDER_COLAMD   4

typedef struct accsp_symbolic    accsp_symbolic_t;
typedef struct accsp_numeric_d   accsp_numeric_d_t;
typedef struct accsp_numeric_f   accsp_numeric_f_t;
typedef struct accsp_subfactor_d accsp_subfactor_d_t;
typedef struct accsp_subfactor_f accsp_subfactor_f_t;

// How a matrix's storage is to be interpreted. `triangle` is meaningful only when `kind` selects
// a symmetric or triangular matrix, and `transpose` is applied by Accelerate on top of the rest.
typedef struct {
    int     kind;
    int     triangle;
    int     transpose;
    uint8_t block_size;
} accsp_attributes;

// A column-major dense operand with an explicit column stride, mirroring the shape of
// Accelerate's dense matrix type so an arbitrarily strided right-hand side block passes through
// without a copy. `column_stride` is in elements and must be at least `row_count`.
typedef struct {
    int     row_count;
    int     column_count;
    int     column_stride;
    double *data;
} accsp_dense_d;

typedef struct {
    int    row_count;
    int    column_count;
    int    column_stride;
    float *data;
} accsp_dense_f;

// Options for the two phases. A NULL options pointer selects Accelerate's defaults, which is not
// the same as a zeroed struct: the tolerances have non-zero defaults.
typedef struct {
    uint32_t control;
    uint8_t  order_method;
} accsp_symbolic_options;

typedef struct {
    uint32_t control;
    uint8_t  scaling_method;
    double   pivot_tolerance;
    double   zero_tolerance;
} accsp_numeric_options;

// Fill an options struct with Accelerate's own defaults.
//
// A zeroed struct is not the same thing: the numeric tolerances have non-zero defaults. A caller
// that wants to override one field and leave the rest alone starts here. The numeric defaults
// differ by element type — the single-precision tolerances are looser — so the shared struct has
// one entry point per type.
void accsp_default_symbolic_options(accsp_symbolic_options *out);
void accsp_default_numeric_options_d(accsp_numeric_options *out);
void accsp_default_numeric_options_f(accsp_numeric_options *out);

// Install the callback Accelerate reports parameter violations through.
//
// This exists because the failure path is not benign. Accelerate's parameter checks call
// `_SparseTrap()` when no error callback is set, and its default options leave the callback NULL,
// so an unguarded caller mistake aborts the process instead of returning a status. Every options
// struct this shim builds carries a callback for that reason.
//
// The handler must not unwind. Passing NULL restores the do-nothing default, which still returns
// normally rather than trapping.
void accsp_set_error_handler(void (*handler)(const char *message));

// Analyze a sparsity pattern. `column_starts` has `columns + 1` entries and `row_indices` has
// `column_starts[columns]`; both are copied, so neither needs to outlive the call.
//
// Writes the outcome through `out_status` and returns NULL unless that outcome is
// ACCSP_STATUS_OK.
accsp_symbolic_t *accsp_symbolic_new(
    int                            kind,
    int                            rows,
    int                            columns,
    const long                    *column_starts,
    const int                     *row_indices,
    const accsp_attributes        *attributes,
    const accsp_symbolic_options  *options,
    int                           *out_status);

void accsp_symbolic_free(accsp_symbolic_t *symbolic);

// Bytes Accelerate reports it will need to hold a numeric factorization of the given element type
// built from this analysis. The analysis carries a separate figure per type.
size_t accsp_symbolic_factor_size_d(const accsp_symbolic_t *symbolic);
size_t accsp_symbolic_factor_size_f(const accsp_symbolic_t *symbolic);

// Numerically factor `values` against a previous analysis. `values` has one entry per stored
// non-zero, in the order the pattern declared them, and is not retained.
//
// Writes the outcome through `out_status` and returns NULL unless that outcome is
// ACCSP_STATUS_OK. The returned handle keeps its own copy of the pattern and remains valid after
// `symbolic` is freed.
accsp_numeric_d_t *accsp_numeric_new_d(
    const accsp_symbolic_t       *symbolic,
    const double                 *values,
    const accsp_numeric_options  *options,
    int                          *out_status);

accsp_numeric_f_t *accsp_numeric_new_f(
    const accsp_symbolic_t       *symbolic,
    const float                  *values,
    const accsp_numeric_options  *options,
    int                          *out_status);

// Re-form an existing factorization from new values for the same pattern, reusing its analysis
// and its already-allocated factor memory.
//
// A failure here leaves the handle allocated but unfactored: another refactor may be attempted
// and is the intended recovery, while solving is refused until one succeeds.
int accsp_numeric_refactor_d(
    accsp_numeric_d_t            *numeric,
    const double                 *values,
    const accsp_numeric_options  *options);

int accsp_numeric_refactor_f(
    accsp_numeric_f_t            *numeric,
    const float                  *values,
    const accsp_numeric_options  *options);

void accsp_numeric_free_d(accsp_numeric_d_t *numeric);
void accsp_numeric_free_f(accsp_numeric_f_t *numeric);

// Status of the last factorization or refactorization performed on this handle.
int accsp_numeric_status_d(const accsp_numeric_d_t *numeric);
int accsp_numeric_status_f(const accsp_numeric_f_t *numeric);

// Solve with the right-hand side and solution in separate operands.
int accsp_solve_d(const accsp_numeric_d_t *numeric,
                  const accsp_dense_d     *b,
                  const accsp_dense_d     *x);

int accsp_solve_f(const accsp_numeric_f_t *numeric,
                  const accsp_dense_f     *b,
                  const accsp_dense_f     *x);

// Solve with the right-hand side overwritten by the solution.
int accsp_solve_in_place_d(const accsp_numeric_d_t *numeric, const accsp_dense_d *xb);
int accsp_solve_in_place_f(const accsp_numeric_f_t *numeric, const accsp_dense_f *xb);

// Count the positive, zero and negative pivots of a factorization, writing each through its
// out-parameter. None of the three may be NULL. Counts are in scalars: observed to sum to the
// matrix dimension in scalars even when the block size is above one, which Apple's documentation
// does not state either way.
//
// Requires a factorization built as ACCSP_KIND_LDLT_TPP; every other kind reports
// ACCSP_STATUS_PARAMETER_ERROR. Requires macOS 13.0 to run and an SDK providing SparseGetInertia
// to build; either one missing reports ACCSP_STATUS_UNSUPPORTED_OS. A factorization whose last
// attempt failed reports ACCSP_STATUS_NOT_FACTORED, which is checked before either of the above.
//
// The out-parameters are left untouched unless ACCSP_STATUS_OK is returned. That is enforced here
// rather than inherited from Accelerate: its own function reports failure as a plain non-zero int
// rather than as a SparseStatus, and the value it uses collides with an unrelated status code, so
// the return is translated and the counts are copied out only on success.
int accsp_get_inertia_d(const accsp_numeric_d_t *numeric,
                        int                     *positive,
                        int                     *zero,
                        int                     *negative);

int accsp_get_inertia_f(const accsp_numeric_f_t *numeric,
                        int                     *positive,
                        int                     *zero,
                        int                     *negative);

// ---------------------------------------------------------------------------------------------
// Subfactors
//
// A subfactor is one piece of a factorization — L from LDL', Q or R from QR, a permutation, a
// scaling — exposed as an object that Multiply and Solve accept. It is not materialised as a
// sparse matrix and cannot be read entry by entry.
//
// Accelerate checks operand shapes on every one of these and reports a mismatch through the error
// callback, leaving the operands untouched — so a caller that ignores the status reads back
// whatever the buffer already held, exactly as for a plain solve. These entry points add no check
// of their own. The shape is not carried in the handle and depends on both the parent's kind and
// which subfactor was taken:
//
// Write s for the smaller of the parent's two scalar dimensions and l for the larger. Accelerate
// sizes from those rather than from rows and columns as given, so a parent wider than it is tall
// yields the same shapes as its transpose:
//
//   Cholesky, LDL' (square parent):  P, S, L, D, PLPS are all s x s
//   QR:                              P, R, RP are s x s; Q is l x s
//   CholeskyAtA:                     P, R, RP are s x s
//   LU (square parent):              P, Q, SR, SC are all s x s
//
// Observed on macOS 26; Apple documents the operand rules relative to an unspecified m x n. Solve
// takes an m-row right-hand side and produces an n-row solution, Multiply takes n rows and
// produces m, and the in-place forms need max(m, n) rows. The safe crate derives all of this and
// checks it before calling; a direct caller of this layer gets Accelerate's report instead.
//
// Three hazards that are not about shapes:
//
//   - A subfactor tracks the factorization it came from rather than a copy of it. Refactoring the
//     parent changes what the subfactor applies. If that refactor *fails*, the subfactor is left
//     applying a factorization that no longer exists: it returns values with no diagnostic, and
//     its own cached copy of the parent's status still reads OK, so nothing here can detect it.
//     A caller must not use a subfactor across a failed refactor of its parent. The safe crate
//     makes that unrepresentable by borrowing the factorization for the subfactor's lifetime.
//   - Multiplying by the ACCSP_SUBFACTOR_PLPS half-solve traps the process, uncatchably, when the
//     operands are the right shape — with the wrong shape the shape check reports first. This shim
//     refuses that combination before dispatch. The equivalent for ACCSP_SUBFACTOR_RP is reported
//     by Accelerate rather than fatal, and is left to it.
//   - Extracting a subfactor from a factorization whose numeric phase failed **traps the process**.
//     The error callback does not intercept it — observed, with a handler installed and without,
//     for a reason that is Accelerate's and not visible from here. The
//     status check at the top of accsp_subfactor_new_* is the only thing standing between a caller
//     and that abort: remove it and a failed refactor followed by an extraction kills the process
//     instead of returning ACCSP_STATUS_NOT_FACTORED.
// ---------------------------------------------------------------------------------------------

// Extract a subfactor. Reports ACCSP_STATUS_PARAMETER_ERROR when the factorization cannot supply
// the requested piece, and ACCSP_STATUS_NOT_FACTORED if the last factorization attempt failed.
// Which pieces a kind offers is fixed by the kind, the scaling ones included, and does not vary
// with the scaling method.
//
// The returned handle keeps the factorization alive, so it stays usable after the numeric handle
// it came from is freed.
accsp_subfactor_d_t *accsp_subfactor_new_d(const accsp_numeric_d_t *numeric,
                                           uint8_t                  subfactor,
                                           int                     *out_status);

accsp_subfactor_f_t *accsp_subfactor_new_f(const accsp_numeric_f_t *numeric,
                                           uint8_t                  subfactor,
                                           int                     *out_status);

void accsp_subfactor_free_d(accsp_subfactor_d_t *subfactor);
void accsp_subfactor_free_f(accsp_subfactor_f_t *subfactor);

// A separately reference-counted handle on the transpose of a subfactor. The original stays valid
// and both must be freed.
accsp_subfactor_d_t *accsp_subfactor_transpose_d(const accsp_subfactor_d_t *subfactor,
                                                 int                       *out_status);
accsp_subfactor_f_t *accsp_subfactor_transpose_f(const accsp_subfactor_f_t *subfactor,
                                                 int                       *out_status);

// Which subfactor this holds, as one of the ACCSP_SUBFACTOR_* values, and whether it is to be
// applied transposed.
uint8_t accsp_subfactor_contents_d(const accsp_subfactor_d_t *subfactor);
uint8_t accsp_subfactor_contents_f(const accsp_subfactor_f_t *subfactor);
int     accsp_subfactor_is_transposed_d(const accsp_subfactor_d_t *subfactor);
int     accsp_subfactor_is_transposed_f(const accsp_subfactor_f_t *subfactor);

// Bytes of scratch space an operation on this subfactor needs, as
// `static_bytes + column_count * per_rhs_bytes`. The entry points below allocate it themselves;
// these figures are reported because that allocation is per call and a caller looping over many
// applications may want to know its size.
void accsp_subfactor_workspace_d(const accsp_subfactor_d_t *subfactor,
                                 size_t                    *static_bytes,
                                 size_t                    *per_rhs_bytes);
void accsp_subfactor_workspace_f(const accsp_subfactor_f_t *subfactor,
                                 size_t                    *static_bytes,
                                 size_t                    *per_rhs_bytes);

// Solve `subfactor * x = b`, and multiply `y = subfactor * x`. The in-place forms overwrite their
// single operand.
//
// The half-solve selectors are solve-only. Multiplying by ACCSP_SUBFACTOR_PLPS reports
// ACCSP_STATUS_PARAMETER_ERROR from here, because letting it reach Accelerate traps the process;
// multiplying by ACCSP_SUBFACTOR_RP is passed through and reported by Accelerate.
//
// See the shape rules above. Accelerate checks them and reports a mismatch; nothing here does.
int accsp_subfactor_solve_d(const accsp_subfactor_d_t *subfactor,
                            const accsp_dense_d       *b,
                            const accsp_dense_d       *x);
int accsp_subfactor_solve_f(const accsp_subfactor_f_t *subfactor,
                            const accsp_dense_f       *b,
                            const accsp_dense_f       *x);
int accsp_subfactor_solve_in_place_d(const accsp_subfactor_d_t *subfactor,
                                     const accsp_dense_d       *xb);
int accsp_subfactor_solve_in_place_f(const accsp_subfactor_f_t *subfactor,
                                     const accsp_dense_f       *xb);
int accsp_subfactor_multiply_d(const accsp_subfactor_d_t *subfactor,
                               const accsp_dense_d       *x,
                               const accsp_dense_d       *y);
int accsp_subfactor_multiply_f(const accsp_subfactor_f_t *subfactor,
                               const accsp_dense_f       *x,
                               const accsp_dense_f       *y);
int accsp_subfactor_multiply_in_place_d(const accsp_subfactor_d_t *subfactor,
                                        const accsp_dense_d       *xy);
int accsp_subfactor_multiply_in_place_f(const accsp_subfactor_f_t *subfactor,
                                        const accsp_dense_f       *xy);

#ifdef __cplusplus
}
#endif

#endif // ACCELERATE_SPARSE_SHIM_H
