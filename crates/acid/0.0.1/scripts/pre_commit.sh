#!/bin/sh

echo "Running cargo fmt --check..."
if ! cargo fmt --check; then
  echo "Formatting issues detected. Run 'cargo fmt' to fix them."
  exit 1
fi
