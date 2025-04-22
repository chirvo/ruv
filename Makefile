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
install: all
	install -m 0755 target/release/ruv /usr/local/bin/ruv
	install -m 0644 ruv.service /etc/systemd/system/ruv.service
	systemctl daemon-reload
	systemctl enable ruv.service

# Uninstall target
uninstall:
	systemctl disable ruv.service || true # Ignore error if not enabled
	rm -f /etc/systemd/system/ruv.service
	rm -f /usr/local/bin/ruv
	systemctl daemon-reload