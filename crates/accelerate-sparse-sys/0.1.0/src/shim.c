// Implementation of the flattened entry points declared in shim.h. See that header for the
// conventions this file follows.

#include <Accelerate/Accelerate.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <string.h>

#include "shim.h"

// ---------------------------------------------------------------------------------------------
// Drift checks
//
// The Rust declarations for this shim are written by hand, so the constants in shim.h are not
// generated from anything and could fall out of step with the SDK. These make that a compile
// error on the machine doing the building, at no runtime cost. A generator would instead
// regenerate the changed value silently and leave the Rust side to mismatch it.
// ---------------------------------------------------------------------------------------------

_Static_assert(ACCSP_STATUS_OK == SparseStatusOK, "status code drift: OK");
_Static_assert(ACCSP_STATUS_FACTORIZATION_FAILED == SparseFactorizationFailed,
               "status code drift: factorization failed");
_Static_assert(ACCSP_STATUS_MATRIX_IS_SINGULAR == SparseMatrixIsSingular,
               "status code drift: singular");
_Static_assert(ACCSP_STATUS_INTERNAL_ERROR == SparseInternalError, "status code drift: internal");
_Static_assert(ACCSP_STATUS_PARAMETER_ERROR == SparseParameterError, "status code drift: parameter");

_Static_assert(ACCSP_KIND_CHOLESKY == SparseFactorizationCholesky, "factorization kind drift: Cholesky");
_Static_assert(ACCSP_KIND_LDLT_UNPIVOTED == SparseFactorizationLDLTUnpivoted, "factorization kind drift: LDLT-Unpivoted");
_Static_assert(ACCSP_KIND_LDLT_SBK == SparseFactorizationLDLTSBK, "factorization kind drift: LDLT-SBK");
_Static_assert(ACCSP_KIND_LDLT_TPP == SparseFactorizationLDLTTPP, "factorization kind drift: LDLT-TPP");
#ifdef ACCSP_HAVE_LU
// Naming an API_AVAILABLE enumerator draws -Wunguarded-availability-new when the deployment target
// is below the version it was introduced in. That diagnostic is about calls the running OS might
// not provide, and a _Static_assert emits no code: it compares two compile-time constants and is
// gone before anything runs. Suppressed for these three lines only — the runtime guard that does
// matter is in accsp_symbolic_new.
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wunguarded-availability-new"
_Static_assert(ACCSP_KIND_LU_UNPIVOTED == SparseFactorizationLUUnpivoted, "factorization kind drift: LU-Unpivoted");
_Static_assert(ACCSP_KIND_LU_SPP == SparseFactorizationLUSPP, "factorization kind drift: LU-SPP");
_Static_assert(ACCSP_KIND_LU_TPP == SparseFactorizationLUTPP, "factorization kind drift: LU-TPP");
#pragma clang diagnostic pop
#endif
_Static_assert(ACCSP_KIND_QR == SparseFactorizationQR, "factorization kind drift: QR");
_Static_assert(ACCSP_KIND_CHOLESKY_ATA == SparseFactorizationCholeskyAtA, "factorization kind drift: CholeskyAtA");

_Static_assert(ACCSP_MATRIX_ORDINARY == SparseOrdinary, "matrix kind drift: ordinary");
_Static_assert(ACCSP_MATRIX_SYMMETRIC == SparseSymmetric, "matrix kind drift: symmetric");
_Static_assert(ACCSP_TRIANGLE_UPPER == SparseUpperTriangle, "triangle drift: upper");
_Static_assert(ACCSP_TRIANGLE_LOWER == SparseLowerTriangle, "triangle drift: lower");

_Static_assert(ACCSP_SUBFACTOR_INVALID == SparseSubfactorInvalid, "subfactor drift: invalid");
_Static_assert(ACCSP_SUBFACTOR_P == SparseSubfactorP, "subfactor drift: P");
_Static_assert(ACCSP_SUBFACTOR_S == SparseSubfactorS, "subfactor drift: S");
_Static_assert(ACCSP_SUBFACTOR_L == SparseSubfactorL, "subfactor drift: L");
_Static_assert(ACCSP_SUBFACTOR_D == SparseSubfactorD, "subfactor drift: D");
_Static_assert(ACCSP_SUBFACTOR_PLPS == SparseSubfactorPLPS, "subfactor drift: PLPS");
_Static_assert(ACCSP_SUBFACTOR_Q == SparseSubfactorQ, "subfactor drift: Q");
_Static_assert(ACCSP_SUBFACTOR_R == SparseSubfactorR, "subfactor drift: R");
_Static_assert(ACCSP_SUBFACTOR_RP == SparseSubfactorRP, "subfactor drift: RP");
#ifdef ACCSP_HAVE_LU
// As with the LU kind constants: naming an API_AVAILABLE enumerator warns at a low deployment
// target, and a static assert emits no code, so the diagnostic cannot apply to it.
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wunguarded-availability-new"
_Static_assert(ACCSP_SUBFACTOR_SR == SparseSubfactorSr, "subfactor drift: Sr");
_Static_assert(ACCSP_SUBFACTOR_SC == SparseSubfactorSc, "subfactor drift: Sc");
#pragma clang diagnostic pop
#endif

_Static_assert(ACCSP_ORDER_DEFAULT == SparseOrderDefault, "order method drift: default");
_Static_assert(ACCSP_ORDER_USER == SparseOrderUser, "order method drift: user");
_Static_assert(ACCSP_ORDER_AMD == SparseOrderAMD, "order method drift: AMD");
_Static_assert(ACCSP_ORDER_METIS == SparseOrderMetis, "order method drift: Metis");
_Static_assert(ACCSP_ORDER_COLAMD == SparseOrderCOLAMD, "order method drift: COLAMD");

// The one layout fact the Rust side depends on indirectly: attributes are built here by named
// field, but the surrounding structs are laid out around this type's size.
_Static_assert(sizeof(SparseAttributes_t) == 4, "attributes layout drift");

// The Rust declarations require a 32-bit C int.
_Static_assert(sizeof(int) == 4, "int is not 32 bits");

// ---------------------------------------------------------------------------------------------
// Error reporting
// ---------------------------------------------------------------------------------------------

static _Atomic(void (*)(const char *)) accsp_error_handler = NULL;

void accsp_set_error_handler(void (*handler)(const char *message)) {
    atomic_store_explicit(&accsp_error_handler, handler, memory_order_release);
}

// Installed on every options struct built below. Returning normally when no handler is
// registered is the point: that is still an improvement on Accelerate's default, which traps.
static void accsp_report_error(const char *message) {
    void (*handler)(const char *) =
        atomic_load_explicit(&accsp_error_handler, memory_order_acquire);
    if (handler != NULL) {
        handler(message);
    }
}

// ---------------------------------------------------------------------------------------------
// Handles
// ---------------------------------------------------------------------------------------------

// Both handles own a copy of the sparsity pattern. The symbolic one needs it because Accelerate
// builds a numeric factorization from a matrix rather than from indices alone; the numeric one
// needs its own because it must stay refactorable after the symbolic handle is freed, which
// Accelerate supports for the factorization it retains internally but not for our index arrays.
typedef struct {
    int                rows;
    int                columns;
    long               nnz;
    long              *column_starts;
    int               *row_indices;
    SparseAttributes_t attributes;
    uint8_t            block_size;
} accsp_pattern;

struct accsp_symbolic {
    SparseOpaqueSymbolicFactorization inner;
    accsp_pattern                     pattern;
};

static void accsp_pattern_release(accsp_pattern *pattern) {
    free(pattern->column_starts);
    free(pattern->row_indices);
    pattern->column_starts = NULL;
    pattern->row_indices = NULL;
}

// Copies the caller's arrays. Returns false if either allocation fails, having released whatever
// it did manage to take.
static bool accsp_pattern_copy(accsp_pattern       *out,
                               int                  rows,
                               int                  columns,
                               const long          *column_starts,
                               const int           *row_indices,
                               SparseAttributes_t   attributes,
                               uint8_t              block_size) {
    long nnz = column_starts[columns];

    out->rows = rows;
    out->columns = columns;
    out->nnz = nnz;
    out->attributes = attributes;
    out->block_size = block_size;
    // The `?: 1` keeps a zero-non-zero pattern well defined: malloc(0) may return NULL, which
    // would be indistinguishable from failure.
    out->column_starts = malloc(sizeof(long) * (size_t)(columns + 1));
    out->row_indices = malloc(sizeof(int) * (size_t)(nnz > 0 ? nnz : 1));
    if (out->column_starts == NULL || out->row_indices == NULL) {
        accsp_pattern_release(out);
        return false;
    }

    memcpy(out->column_starts, column_starts, sizeof(long) * (size_t)(columns + 1));
    memcpy(out->row_indices, row_indices, sizeof(int) * (size_t)nnz);
    return true;
}

static SparseMatrixStructure accsp_structure(const accsp_pattern *pattern) {
    SparseMatrixStructure structure;
    structure.rowCount = pattern->rows;
    structure.columnCount = pattern->columns;
    structure.columnStarts = pattern->column_starts;
    structure.rowIndices = pattern->row_indices;
    structure.attributes = pattern->attributes;
    structure.blockSize = pattern->block_size;
    return structure;
}

// ---------------------------------------------------------------------------------------------
// Options
//
// Both start from Accelerate's own defaults rather than from field-by-field literals, so this
// file never transcribes Apple's chosen tolerances, and the callback is then installed on top.
// ---------------------------------------------------------------------------------------------

static SparseSymbolicFactorOptions accsp_symbolic_factor_options(
    const accsp_symbolic_options *options) {
    SparseSymbolicFactorOptions out = _SparseDefaultSymbolicFactorOptions;
    if (options != NULL) {
        out.control = (SparseControl_t)options->control;
        out.orderMethod = (SparseOrder_t)options->order_method;
    }
    out.reportError = accsp_report_error;
    return out;
}

void accsp_default_symbolic_options(accsp_symbolic_options *out) {
    out->control = (uint32_t)_SparseDefaultSymbolicFactorOptions.control;
    out->order_method = (uint8_t)_SparseDefaultSymbolicFactorOptions.orderMethod;
}

static SparseAttributes_t accsp_make_attributes(const accsp_attributes *attributes) {
    SparseAttributes_t out = (SparseAttributes_t){0};
    out.kind = (SparseKind_t)attributes->kind;
    out.triangle = (SparseTriangle_t)attributes->triangle;
    out.transpose = attributes->transpose != 0;
    return out;
}

// ---------------------------------------------------------------------------------------------
// Symbolic phase
// ---------------------------------------------------------------------------------------------

#ifdef ACCSP_HAVE_LU
static bool accsp_is_lu_kind(int kind) {
    return kind == ACCSP_KIND_LU_UNPIVOTED || kind == ACCSP_KIND_LU_SPP ||
           kind == ACCSP_KIND_LU_TPP;
}
#endif

accsp_symbolic_t *accsp_symbolic_new(int                           kind,
                                     int                           rows,
                                     int                           columns,
                                     const long                   *column_starts,
                                     const int                    *row_indices,
                                     const accsp_attributes       *attributes,
                                     const accsp_symbolic_options *options,
                                     int                          *out_status) {
#ifdef ACCSP_HAVE_LU
    // SparseFactor dispatches LU through a __builtin_verbose_trap on an OS older than 15.5, and
    // that trap is not interceptable by the error callback. Catch it here, before the call, so an
    // unsupported OS returns a status rather than aborting the process. __builtin_available must
    // be the whole condition of its `if`, so it cannot be folded into the kind test.
    if (accsp_is_lu_kind(kind)) {
        if (__builtin_available(macOS 15.5, *)) {
            // Supported on this OS; fall through to the normal path.
        } else {
            *out_status = ACCSP_STATUS_UNSUPPORTED_OS;
            return NULL;
        }
    }
#endif

    accsp_pattern pattern;
    SparseAttributes_t sparse_attributes = accsp_make_attributes(attributes);
    if (!accsp_pattern_copy(&pattern, rows, columns, column_starts, row_indices,
                            sparse_attributes, attributes->block_size)) {
        *out_status = ACCSP_STATUS_ALLOCATION_FAILED;
        return NULL;
    }

    SparseSymbolicFactorOptions factor_options = accsp_symbolic_factor_options(options);
    SparseOpaqueSymbolicFactorization inner =
        SparseFactor((SparseFactorization_t)kind, accsp_structure(&pattern), factor_options);

    *out_status = (int)inner.status;
    if (inner.status != SparseStatusOK) {
        SparseCleanup(inner);
        accsp_pattern_release(&pattern);
        return NULL;
    }

    accsp_symbolic_t *handle = malloc(sizeof(accsp_symbolic_t));
    if (handle == NULL) {
        SparseCleanup(inner);
        accsp_pattern_release(&pattern);
        *out_status = ACCSP_STATUS_ALLOCATION_FAILED;
        return NULL;
    }

    handle->inner = inner;
    handle->pattern = pattern;
    return handle;
}

void accsp_symbolic_free(accsp_symbolic_t *symbolic) {
    if (symbolic == NULL) {
        return;
    }
    SparseCleanup(symbolic->inner);
    accsp_pattern_release(&symbolic->pattern);
    free(symbolic);
}

// ---------------------------------------------------------------------------------------------
// Numeric phase and solve, per element type
//
// Everything below the analysis differs between `double` and `float` only in the types it names:
// Accelerate's own entry points are overloads over the scalar, and this file resolves them. The
// two sets are generated from one macro rather than written twice because the parts that would be
// duplicated are the allocation and cleanup ordering, which is where a divergence between two
// hand-maintained copies would be a leak or a double free rather than a compile error.
//
// `SUFFIX` names the entry points, `SCALAR` is the element type, and `PRECISION` selects
// Accelerate's typed names (`Double` / `Float`), its per-type default options, and the matching
// factor-size field.
// ---------------------------------------------------------------------------------------------

// The inertia query, in its own macro because the SDK guard around it is a preprocessor
// conditional and those cannot appear inside a macro body.
//
// SparseGetInertia is an ordinary exported function rather than one of Accelerate's inlined
// dispatchers, so an OS older than 13.0 leaves it weakly linked and null instead of trapping.
// __builtin_available keeps it from being called there, and must be the whole condition of its
// `if` — Clang rejects it inside `&&` or `!`.
//
// It reports failure as a plain non-zero int, not a SparseStatus. The value observed for a
// factorization of the wrong kind is -1, which is SparseFactorizationFailed's value; forwarding
// it would answer "why did this fail?" with an unrelated fact, so any non-zero is translated to
// a parameter error here.
//
// Accelerate is observed to leave the out-parameters untouched when it refuses, but the header
// promises the caller that nothing is written unless OK is returned, and that promise should not
// rest on an observation. So the counts are collected into locals and copied out only on success:
// the guarantee then holds whatever a future Accelerate writes before noticing the wrong kind.
#ifdef ACCSP_HAVE_INERTIA
#define ACCSP_DEFINE_INERTIA(SUFFIX, PRECISION)                                                  \
    static int accsp_inertia_##SUFFIX(SparseOpaqueFactorization_##PRECISION factored,            \
                                      int *positive, int *zero, int *negative) {                 \
        if (__builtin_available(macOS 13.0, *)) {                                                \
            int p = 0, z = 0, n = 0;                                                             \
            if (SparseGetInertia(factored, &p, &z, &n) != 0) {                                  \
                return ACCSP_STATUS_PARAMETER_ERROR;                                             \
            }                                                                                    \
            *positive = p;                                                                       \
            *zero = z;                                                                           \
            *negative = n;                                                                       \
            return ACCSP_STATUS_OK;                                                              \
        }                                                                                        \
        return ACCSP_STATUS_UNSUPPORTED_OS;                                                      \
    }
#else
#define ACCSP_DEFINE_INERTIA(SUFFIX, PRECISION)                                                  \
    static int accsp_inertia_##SUFFIX(SparseOpaqueFactorization_##PRECISION factored,            \
                                      int *positive, int *zero, int *negative) {                 \
        (void)factored;                                                                          \
        (void)positive;                                                                          \
        (void)zero;                                                                              \
        (void)negative;                                                                          \
        return ACCSP_STATUS_UNSUPPORTED_OS;                                                      \
    }
#endif

#define ACCSP_DEFINE_NUMERIC(SUFFIX, SCALAR, PRECISION)                                          \
                                                                                                 \
    ACCSP_DEFINE_INERTIA(SUFFIX, PRECISION)                                                      \
                                                                                                 \
    struct accsp_numeric_##SUFFIX {                                                              \
        SparseOpaqueFactorization_##PRECISION inner;                                             \
        accsp_pattern                         pattern;                                           \
    };                                                                                           \
                                                                                                 \
    static SparseNumericFactorOptions accsp_numeric_factor_options_##SUFFIX(                     \
        const accsp_numeric_options *options) {                                                  \
        SparseNumericFactorOptions out = _SparseDefaultNumericFactorOptions_##PRECISION;         \
        if (options != NULL) {                                                                   \
            out.control = (SparseControl_t)options->control;                                     \
            out.scalingMethod = (SparseScaling_t)options->scaling_method;                        \
            out.pivotTolerance = options->pivot_tolerance;                                       \
            out.zeroTolerance = options->zero_tolerance;                                         \
        }                                                                                        \
        return out;                                                                              \
    }                                                                                            \
                                                                                                 \
    void accsp_default_numeric_options_##SUFFIX(accsp_numeric_options *out) {                    \
        out->control = (uint32_t)_SparseDefaultNumericFactorOptions_##PRECISION.control;         \
        out->scaling_method =                                                                    \
            (uint8_t)_SparseDefaultNumericFactorOptions_##PRECISION.scalingMethod;               \
        out->pivot_tolerance = _SparseDefaultNumericFactorOptions_##PRECISION.pivotTolerance;    \
        out->zero_tolerance = _SparseDefaultNumericFactorOptions_##PRECISION.zeroTolerance;      \
    }                                                                                            \
                                                                                                 \
    static DenseMatrix_##PRECISION accsp_dense_matrix_##SUFFIX(const accsp_dense_##SUFFIX *d) {  \
        DenseMatrix_##PRECISION out;                                                             \
        out.rowCount = d->row_count;                                                             \
        out.columnCount = d->column_count;                                                       \
        out.columnStride = d->column_stride;                                                     \
        out.attributes = (SparseAttributes_t){0};                                                \
        out.data = d->data;                                                                      \
        return out;                                                                              \
    }                                                                                            \
                                                                                                 \
    size_t accsp_symbolic_factor_size_##SUFFIX(const accsp_symbolic_t *symbolic) {               \
        return symbolic->inner.factorSize_##PRECISION;                                           \
    }                                                                                            \
                                                                                                 \
    accsp_numeric_##SUFFIX##_t *accsp_numeric_new_##SUFFIX(                                      \
        const accsp_symbolic_t      *symbolic,                                                   \
        const SCALAR                *values,                                                     \
        const accsp_numeric_options *options,                                                    \
        int                         *out_status) {                                               \
        accsp_pattern pattern;                                                                   \
        if (!accsp_pattern_copy(&pattern, symbolic->pattern.rows, symbolic->pattern.columns,     \
                                symbolic->pattern.column_starts, symbolic->pattern.row_indices,  \
                                symbolic->pattern.attributes, symbolic->pattern.block_size)) {   \
            *out_status = ACCSP_STATUS_ALLOCATION_FAILED;                                        \
            return NULL;                                                                         \
        }                                                                                        \
                                                                                                 \
        SparseMatrix_##PRECISION matrix = {                                                      \
            .structure = accsp_structure(&pattern),                                              \
            .data = (SCALAR *)values,                                                            \
        };                                                                                       \
        SparseNumericFactorOptions factor_options =                                              \
            accsp_numeric_factor_options_##SUFFIX(options);                                      \
        SparseOpaqueFactorization_##PRECISION inner =                                            \
            SparseFactor(symbolic->inner, matrix, factor_options);                               \
                                                                                                 \
        *out_status = (int)inner.status;                                                         \
        if (inner.status != SparseStatusOK) {                                                    \
            SparseCleanup(inner);                                                                \
            accsp_pattern_release(&pattern);                                                     \
            return NULL;                                                                         \
        }                                                                                        \
                                                                                                 \
        accsp_numeric_##SUFFIX##_t *handle = malloc(sizeof(accsp_numeric_##SUFFIX##_t));         \
        if (handle == NULL) {                                                                    \
            SparseCleanup(inner);                                                                \
            accsp_pattern_release(&pattern);                                                     \
            *out_status = ACCSP_STATUS_ALLOCATION_FAILED;                                        \
            return NULL;                                                                         \
        }                                                                                        \
                                                                                                 \
        handle->inner = inner;                                                                   \
        handle->pattern = pattern;                                                               \
        return handle;                                                                           \
    }                                                                                            \
                                                                                                 \
    int accsp_numeric_refactor_##SUFFIX(accsp_numeric_##SUFFIX##_t  *numeric,                    \
                                        const SCALAR                *values,                     \
                                        const accsp_numeric_options *options) {                  \
        SparseMatrix_##PRECISION matrix = {                                                      \
            .structure = accsp_structure(&numeric->pattern),                                     \
            .data = (SCALAR *)values,                                                            \
        };                                                                                       \
        SparseNumericFactorOptions factor_options =                                              \
            accsp_numeric_factor_options_##SUFFIX(options);                                      \
                                                                                                 \
        /* SparseRefactor reports its outcome by writing the factorization's status field      */\
        /* rather than by returning it.                                                        */\
        SparseRefactor(matrix, &numeric->inner, factor_options);                                 \
        return (int)numeric->inner.status;                                                       \
    }                                                                                            \
                                                                                                 \
    void accsp_numeric_free_##SUFFIX(accsp_numeric_##SUFFIX##_t *numeric) {                      \
        if (numeric == NULL) {                                                                   \
            return;                                                                              \
        }                                                                                        \
        SparseCleanup(numeric->inner);                                                           \
        accsp_pattern_release(&numeric->pattern);                                                \
        free(numeric);                                                                           \
    }                                                                                            \
                                                                                                 \
    int accsp_numeric_status_##SUFFIX(const accsp_numeric_##SUFFIX##_t *numeric) {               \
        return (int)numeric->inner.status;                                                       \
    }                                                                                            \
                                                                                                 \
    /* Accelerate refuses to solve with a factorization whose status is not OK, and reports    */\
    /* that refusal through the error callback rather than to the caller. Checking first is    */\
    /* what turns it into a status the caller can act on.                                      */\
    int accsp_solve_##SUFFIX(const accsp_numeric_##SUFFIX##_t *numeric,                          \
                             const accsp_dense_##SUFFIX       *b,                                \
                             const accsp_dense_##SUFFIX       *x) {                              \
        if (numeric->inner.status != SparseStatusOK) {                                           \
            return ACCSP_STATUS_NOT_FACTORED;                                                    \
        }                                                                                        \
        SparseSolve(numeric->inner, accsp_dense_matrix_##SUFFIX(b),                              \
                    accsp_dense_matrix_##SUFFIX(x));                                             \
        return ACCSP_STATUS_OK;                                                                  \
    }                                                                                            \
                                                                                                 \
    int accsp_solve_in_place_##SUFFIX(const accsp_numeric_##SUFFIX##_t *numeric,                 \
                                      const accsp_dense_##SUFFIX       *xb) {                    \
        if (numeric->inner.status != SparseStatusOK) {                                           \
            return ACCSP_STATUS_NOT_FACTORED;                                                    \
        }                                                                                        \
        SparseSolve(numeric->inner, accsp_dense_matrix_##SUFFIX(xb));                            \
        return ACCSP_STATUS_OK;                                                                  \
    }                                                                                            \
                                                                                                 \
    int accsp_get_inertia_##SUFFIX(const accsp_numeric_##SUFFIX##_t *numeric,                    \
                                   int                             *positive,                    \
                                   int                             *zero,                        \
                                   int                             *negative) {                  \
        if (numeric->inner.status != SparseStatusOK) {                                           \
            return ACCSP_STATUS_NOT_FACTORED;                                                    \
        }                                                                                        \
        return accsp_inertia_##SUFFIX(numeric->inner, positive, zero, negative);                 \
    }                                                                                            \
                                                                                                 \
    struct accsp_subfactor_##SUFFIX {                                                            \
        SparseOpaqueSubfactor_##PRECISION inner;                                                  \
    };                                                                                           \
                                                                                                 \
    /* Boxes a subfactor Accelerate has already produced. An invalid one is never boxed, so    */\
    /* every handle the caller holds refers to a piece the factorization really has.           */\
    static accsp_subfactor_##SUFFIX##_t *accsp_box_subfactor_##SUFFIX(                            \
        SparseOpaqueSubfactor_##PRECISION inner, int *out_status) {                               \
        if (inner.contents == SparseSubfactorInvalid) {                                          \
            /* Nothing was created, so there is nothing to clean up: an invalid subfactor is   */\
            /* Accelerate's way of saying the request did not apply, not a live object.        */\
            *out_status = ACCSP_STATUS_PARAMETER_ERROR;                                          \
            return NULL;                                                                         \
        }                                                                                        \
        accsp_subfactor_##SUFFIX##_t *handle = malloc(sizeof(accsp_subfactor_##SUFFIX##_t));     \
        if (handle == NULL) {                                                                    \
            SparseCleanup(inner);                                                                \
            *out_status = ACCSP_STATUS_ALLOCATION_FAILED;                                        \
            return NULL;                                                                         \
        }                                                                                        \
        handle->inner = inner;                                                                   \
        *out_status = ACCSP_STATUS_OK;                                                           \
        return handle;                                                                            \
    }                                                                                            \
                                                                                                 \
    /* SparseCreateSubfactor traps on a failed numeric factorization and takes no options        */\
    /* through which the error callback could intercept it.                                     */\
    accsp_subfactor_##SUFFIX##_t *accsp_subfactor_new_##SUFFIX(                                   \
        const accsp_numeric_##SUFFIX##_t *numeric, uint8_t subfactor, int *out_status) {          \
        if (numeric->inner.status != SparseStatusOK) {                                            \
            *out_status = ACCSP_STATUS_NOT_FACTORED;                                             \
            return NULL;                                                                         \
        }                                                                                        \
        return accsp_box_subfactor_##SUFFIX(                                                      \
            SparseCreateSubfactor((SparseSubfactor_t)subfactor, numeric->inner), out_status);     \
    }                                                                                            \
                                                                                                 \
    accsp_subfactor_##SUFFIX##_t *accsp_subfactor_transpose_##SUFFIX(                             \
        const accsp_subfactor_##SUFFIX##_t *subfactor, int *out_status) {                         \
        return accsp_box_subfactor_##SUFFIX(SparseGetTranspose(subfactor->inner), out_status);    \
    }                                                                                            \
                                                                                                 \
    void accsp_subfactor_free_##SUFFIX(accsp_subfactor_##SUFFIX##_t *subfactor) {                \
        if (subfactor == NULL) {                                                                 \
            return;                                                                              \
        }                                                                                        \
        SparseCleanup(subfactor->inner);                                                          \
        free(subfactor);                                                                          \
    }                                                                                            \
                                                                                                 \
    uint8_t accsp_subfactor_contents_##SUFFIX(const accsp_subfactor_##SUFFIX##_t *subfactor) {   \
        return (uint8_t)subfactor->inner.contents;                                                \
    }                                                                                            \
                                                                                                 \
    int accsp_subfactor_is_transposed_##SUFFIX(const accsp_subfactor_##SUFFIX##_t *subfactor) {  \
        return subfactor->inner.attributes.transpose != 0;                                        \
    }                                                                                            \
                                                                                                 \
    void accsp_subfactor_workspace_##SUFFIX(const accsp_subfactor_##SUFFIX##_t *subfactor,       \
                                            size_t *static_bytes, size_t *per_rhs_bytes) {       \
        *static_bytes = subfactor->inner.workspaceRequiredStatic;                                 \
        *per_rhs_bytes = subfactor->inner.workspaceRequiredPerRHS;                                \
    }                                                                                            \
                                                                                                 \
    int accsp_subfactor_solve_##SUFFIX(const accsp_subfactor_##SUFFIX##_t *subfactor,             \
                                       const accsp_dense_##SUFFIX       *b,                      \
                                       const accsp_dense_##SUFFIX       *x) {                    \
        SparseSolve(subfactor->inner, accsp_dense_matrix_##SUFFIX(b),                             \
                    accsp_dense_matrix_##SUFFIX(x));                                              \
        return ACCSP_STATUS_OK;                                                                  \
    }                                                                                            \
                                                                                                 \
    int accsp_subfactor_solve_in_place_##SUFFIX(const accsp_subfactor_##SUFFIX##_t *subfactor,    \
                                                const accsp_dense_##SUFFIX       *xb) {          \
        SparseSolve(subfactor->inner, accsp_dense_matrix_##SUFFIX(xb));                           \
        return ACCSP_STATUS_OK;                                                                  \
    }                                                                                            \
                                                                                                 \
    /* Multiplying by the half-solve traps the process rather than reporting, so it is refused */\
    /* here. A wrong operand shape would be caught by Accelerate's own check first, which is   */\
    /* why this cannot be left to a shape guard.                                              */\
    int accsp_subfactor_multiply_##SUFFIX(const accsp_subfactor_##SUFFIX##_t *subfactor,          \
                                          const accsp_dense_##SUFFIX       *x,                    \
                                          const accsp_dense_##SUFFIX       *y) {                  \
        if (subfactor->inner.contents == SparseSubfactorPLPS) {                                  \
            return ACCSP_STATUS_PARAMETER_ERROR;                                                 \
        }                                                                                        \
        SparseMultiply(subfactor->inner, accsp_dense_matrix_##SUFFIX(x),                          \
                       accsp_dense_matrix_##SUFFIX(y));                                           \
        return ACCSP_STATUS_OK;                                                                  \
    }                                                                                            \
                                                                                                 \
    int accsp_subfactor_multiply_in_place_##SUFFIX(                                               \
        const accsp_subfactor_##SUFFIX##_t *subfactor, const accsp_dense_##SUFFIX *xy) {          \
        if (subfactor->inner.contents == SparseSubfactorPLPS) {                                  \
            return ACCSP_STATUS_PARAMETER_ERROR;                                                 \
        }                                                                                        \
        SparseMultiply(subfactor->inner, accsp_dense_matrix_##SUFFIX(xy));                        \
        return ACCSP_STATUS_OK;                                                                  \
    }

ACCSP_DEFINE_NUMERIC(d, double, Double)
ACCSP_DEFINE_NUMERIC(f, float, Float)
