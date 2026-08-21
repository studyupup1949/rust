#!/bin/bash

# ABPilot CC SDK Full Integration Test
# This script tests the complete workflow from authentication to cleanup

set -e

echo "=== ABPilot CC SDK Integration Test ==="
echo ""

# Check if email is provided
if [ -z "$TEST_EMAIL" ]; then
    echo "❌ Error: TEST_EMAIL environment variable not set"
    echo "Usage: TEST_EMAIL=your@email.com ./run_full_test.sh"
    exit 1
fi

echo "📧 Testing with email: $TEST_EMAIL"
echo ""

# Step 1: Run the full test
echo "🚀 Starting full integration test..."
echo ""

cargo run --example full_test --all-features

echo ""
echo "✅ Integration test completed successfully!"
