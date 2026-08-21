# Variables
CARGO = cargo
FMT = rustfmt
CLIPPY = cargo clippy
TEST = cargo test

# Default target: run the program
run:
	$(CARGO) run -- $(log_data)

# Run tests
test:
	$(TEST)

# Format code using rustfmt
fmt:
	$(FMT)

# Run Clippy (linter)
clippy:
	$(CLIPPY)

# Build the project (optional but useful for ensuring compilation works)
build:
	$(CARGO) build

# Clean the project
clean:
	$(CARGO) clean

# Lint and format the code before committing
pre-commit: fmt clippy test

# Helpful to check project status (all the above commands in one)
status:
	$(CARGO) check

# Run the project with a sample log data argument
run-sample:
	$(CARGO) run -- "2023-07-20 12:34:56 123e4567-e89b-12d3-a456-426614174000 192.168.1.1 9.99" "2023-07-21 14:00:00 123e4567-e89b-12d3-a456-426614174001 192.168.1.2 19.99"

# If running any specific commands or arguments, use the following example:
run-credits:
	$(CARGO) run -- --credits
