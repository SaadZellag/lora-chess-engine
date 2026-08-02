# OpenBench-compliant Makefile for LORA Chess Engine
# Supports: EXE=, CC=, CXX=, EVALFILE=, and bench command

# Default values
EXE ?= lora
EVALFILE ?=
CARGO ?= cargo

# Cargo build flags - always release with native CPU optimization
CARGO_FLAGS = --release
RUSTFLAGS = -C target-cpu=native

# Determine binary name
BINARY = target/release/lora
FINAL_BINARY = $(EXE)

.PHONY: all build clean bench help

all: build

build: $(FINAL_BINARY)

$(FINAL_BINARY): $(BINARY)
	@cp $(BINARY) $(FINAL_BINARY)
	@echo "Built: $(FINAL_BINARY)"

# Build the Rust binary with cargo
$(BINARY):
	@echo "Building LORA Chess Engine (release, native CPU)..."
	@if [ -n "$(EVALFILE)" ]; then \
		echo "Compiling with Network file: $(EVALFILE)"; \
		RUSTFLAGS="$(RUSTFLAGS)" EVALFILE=$(EVALFILE) $(CARGO) build $(CARGO_FLAGS); \
	else \
		RUSTFLAGS="$(RUSTFLAGS)" $(CARGO) build $(CARGO_FLAGS); \
	fi

# Benchmark target - runs the engine bench command
bench: $(FINAL_BINARY)
	@echo "Running benchmark..."
	@./$(FINAL_BINARY) bench

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@$(CARGO) clean
	@rm -f $(EXE)
	@echo "Clean complete"

# Help target
help:
	@echo "LORA Chess Engine - OpenBench Makefile"
	@echo ""
	@echo "Usage: make [target] [OPTION=value]"
	@echo ""
	@echo "Targets:"
	@echo "  all      - Build the engine (default)"
	@echo "  build    - Build the engine"
	@echo "  bench    - Build and run benchmark"
	@echo "  clean    - Remove build artifacts"
	@echo "  help     - Display this help message"
	@echo ""
	@echo "Options:"
	@echo "  EXE=<name>       - Output binary name (default: lora)"
	@echo "  EVALFILE=<path>  - Path to network file to embed"
	@echo "  CC=<compiler>    - C compiler (default: gcc)"
	@echo "  CXX=<compiler>   - C++ compiler (default: g++)"
	@echo "  - Always built in release mode with native CPU optimization"
	@echo ""
	@echo "Examples:"
	@echo "  make                              # Build in release mode"
	@echo "  make EXE=lora-v1.0               # Build with custom name"
	@echo "  make bench                        # Run benchmark"
	@echo "  make EXE=lora-ABCDEFGH           # OpenBench build command"
	@echo "  make EVALFILE=/path/to/net.pb    # Build with network"
