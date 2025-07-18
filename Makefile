# Cross-platform Makefile for LLM Terminal
# Works on Windows (with make), macOS, and Linux

# Detect operating system
ifeq ($(OS),Windows_NT)
	EXECUTABLE_EXT = .exe
	SHELL_CMD = cmd
	SETUP_SCRIPT = setup_env.bat
else
	EXECUTABLE_EXT = 
	SHELL_CMD = bash
	SETUP_SCRIPT = setup_env.sh
endif

# Binary name
BINARY_NAME = llm-terminal$(EXECUTABLE_EXT)
TARGET_DIR = target/release
EXECUTABLE_PATH = $(TARGET_DIR)/$(BINARY_NAME)

# Default target
.PHONY: all
all: build

# Build the project
.PHONY: build
build:
	@echo "🔨 Building LLM Terminal..."
	cargo build --release
	@echo "✅ Build complete: $(EXECUTABLE_PATH)"

# Build for development
.PHONY: dev
dev:
	@echo "🔨 Building LLM Terminal (debug mode)..."
	cargo build

# Run the application
.PHONY: run
run: build
	@echo "🚀 Starting LLM Terminal..."
	@echo "Press Ctrl+Q to quit, Ctrl+, for settings"
	@echo ""
	@$(EXECUTABLE_PATH)

# Run from source (development)
.PHONY: run-dev
run-dev:
	@echo "🚀 Starting LLM Terminal (from source)..."
	@echo "Press Ctrl+Q to quit, Ctrl+, for settings"
	@echo ""
	@cargo run

# Clean build artifacts
.PHONY: clean
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean complete"

# Run tests
.PHONY: test
test:
	@echo "🧪 Running tests..."
	cargo test

# Check code formatting and linting
.PHONY: check
check:
	@echo "🔍 Checking code..."
	cargo check
	@echo "✅ Check complete"

# Format code
.PHONY: fmt
format:
	@echo "🎨 Formatting code..."
	cargo fmt

# Setup environment (interactive)
.PHONY: setup
setup:
	@echo "🔧 Running setup script..."
ifeq ($(OS),Windows_NT)
	@$(SETUP_SCRIPT)
else
	@bash $(SETUP_SCRIPT)
endif

# Install dependencies (if needed)
.PHONY: deps
deps:
	@echo "📦 Installing dependencies..."
	cargo fetch

# Display help
.PHONY: help
help:
	@echo "🚀 LLM Terminal - Cross-platform Make Commands"
	@echo "=============================================="
	@echo ""
	@echo "Build commands:"
	@echo "  make build     - Build release version"
	@echo "  make dev       - Build debug version"
	@echo "  make clean     - Clean build artifacts"
	@echo ""
	@echo "Run commands:"
	@echo "  make run       - Build and run application"
	@echo "  make run-dev   - Run from source (development)"
	@echo ""
	@echo "Development commands:"
	@echo "  make test      - Run tests"
	@echo "  make check     - Check code (linting)"
	@echo "  make format    - Format code"
	@echo ""
	@echo "Setup commands:"
	@echo "  make setup     - Run interactive setup"
	@echo "  make deps      - Install dependencies"
	@echo ""
	@echo "Other commands:"
	@echo "  make help      - Show this help message"
	@echo ""
	@echo "Platform: $(shell uname -s 2>/dev/null || echo Windows)"
	@echo "Executable: $(EXECUTABLE_PATH)"

# Default help if no target specified
.DEFAULT_GOAL := help
