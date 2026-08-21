"""aam_rs — compatibility shim for the aam-py Python bindings.

The compiled Rust extension is ``aam_py``. This package re-exports the
current AAM API for compatibility with ``import aam_rs``.
"""

from aam_py import AAM, AAMBuilder, SchemaField, __version__  # noqa: F401

__all__ = ["AAM", "AAMBuilder", "SchemaField", "__version__"]
