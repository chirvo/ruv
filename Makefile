# Simple Makefile to compile ruv.rs

# Compiler and flags
CARGO := cargo

# Default target
all:
	$(CARGO) build --release

# Clean target
clean:
	$(CARGO) clean

# Install target
install:
	install -m 0755 target/release/ruv /usr/local/bin/ruv