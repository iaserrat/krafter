# cqt — code-quality toolkit. Build & setup.
# No prebuilt binary is shipped (build from source you can audit), so
# `make setup` produces the binary the quality-branch skill expects.

CARGO ?= cargo
BIN   := target/release/cqt

.PHONY: help setup check release build smoke test fmt clippy loc arch magic positionals quality clean

help:
	@echo "cqt toolkit — targets:"
	@echo "  make setup    verify toolchain, build release binary"
	@echo "  make release  optimized build -> $(BIN)  (what the skill runs)"
	@echo "  make build    debug build"
	@echo "  make smoke    build + verify the binary runs on itself"
	@echo "  make test     structural gates, then cargo test"
	@echo "  make loc      fail if any Rust file exceeds 100 LOC"
	@echo "  make arch     fail if command/shared modules regress to god files"
	@echo "  make magic    fail on production magic-literal regressions"
	@echo "  make positionals fail on production tuple-field/signature regressions"
	@echo "  make quality  run structural gates"
	@echo "  make clean    remove build artifacts"

setup: check release
	@echo ""
	@echo "cqt ready -> $(BIN)"
	@echo "next: ./$(BIN) --help   (run inside the repo you want to measure)"

check:
	@command -v $(CARGO) >/dev/null 2>&1 || { \
		echo "ERROR: '$(CARGO)' not found. Install the Rust toolchain: https://rustup.rs"; \
		echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"; \
		exit 1; }
	@$(CARGO) --version

release:
	$(CARGO) build --release

build:
	$(CARGO) build

smoke: release
	@./$(BIN) --version
	@./$(BIN) --repo . metrics --paths src/main.rs >/dev/null \
		&& echo "smoke OK: cqt measured its own source" \
		|| { echo "smoke FAIL"; exit 1; }

test: quality
	$(CARGO) test

loc:
	@sh scripts/check-rust-loc.sh

arch:
	@sh scripts/check-rust-architecture.sh

magic:
	@sh scripts/check-rust-magic.sh

positionals:
	@sh scripts/check-rust-positionals.sh

quality: loc arch magic positionals

fmt:
	$(CARGO) fmt

clippy:
	$(CARGO) clippy --release

clean:
	$(CARGO) clean
