"""Sphinx configuration for aam-py documentation."""

import os

project = "aam-py"
author = "Nikita Goncharov"
copyright = "2025, Nikita Goncharov"
release = os.getenv("READTHEDOCS_VERSION", "latest")

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
    "sphinx.ext.viewcode",
    "myst_parser",
    "sphinx_copybutton",
]

# Let autodoc run even when the Rust extension is not installed by mocking it.
autodoc_mock_imports = ["aam_py"]

html_theme = "furo"
html_static_path = ["_static"]

source_suffix = {
    ".rst": "restructuredtext",
    ".md": "markdown",
}

master_doc = "index"

