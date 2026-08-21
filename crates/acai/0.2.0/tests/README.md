# ACAI Test Suite

This directory contains tests for the ACAI implementation of the A2A protocol.

## Test Types

- **Unit Tests**: Tests for individual components
- **Integration Tests**: Tests for interaction between components
- **A2A Protocol Tests**: Tests for compatibility with the A2A protocol specification

## A2A Integration Tests

The file `a2a_integration.rs` includes tests for compatibility with the A2A protocol:

1. `rust_server_with_a2a_protocol`: Tests the ACAI server implementation against an ACAI client
2. `acai_client_with_python_a2a_server`: Tests the ACAI client against the official Python reference implementation

### Running the Python Integration Test

The Python integration test requires the A2A Python reference implementation to be installed. 

To run this test:

1. Clone the A2A repository:
   ```
   git clone https://github.com/google/A2A.git A2A
   ```

2. Install the Python dependencies:
   ```
   cd A2A/samples/python
   pip install -e .
   ```

3. Run the tests:
   ```
   cargo test
   ```

Or run a specific test:
   ```
   cargo test a2a_integration::acai_client_with_python_a2a_server
   ```

## Webhook Notification Test

The file `webhook_notification.rs` contains a test for the push notification system, which verifies:

1. Task status updates are properly sent to a webhook
2. JWT signing and verification works correctly
3. Webhook validation mechanism functions as expected

## Task Manager Tests

The `task_manager.rs` file contains tests for the task manager component, which is responsible for:

1. Creating and managing tasks
2. Updating task status
3. Adding artifacts to tasks
4. Managing push notification configurations