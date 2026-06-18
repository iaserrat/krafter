# rtk — red-team toolkit. Build & setup.
# No prebuilt binary is shipped (offensive tooling should be built from source
# you can audit), so `make setup` produces the binary the skill expects.

CARGO   ?= cargo
BIN      := target/release/rtk
CONFIG   := redteam.toml
EXAMPLE  := redteam.example.toml
# Known HMAC-SHA256(key="key", "The quick brown fox jumps over the lazy dog")
SMOKE_VEC := f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8

.PHONY: help setup check release build config smoke test fmt clippy loc arch magic positionals oracle-coverage quality clean

help:
	@echo "rtk toolkit — targets:"
	@echo "  make setup    verify toolchain, build release binary, create $(CONFIG)"
	@echo "  make release  optimized build -> $(BIN)  (what the skill runs)"
	@echo "  make build    debug build"
	@echo "  make config   create $(CONFIG) from $(EXAMPLE) if missing"
	@echo "  make smoke     build + verify the binary runs (HMAC self-check)"
	@echo "  make test     cargo test"
	@echo "  make loc      fail if any Rust file exceeds 100 LOC"
	@echo "  make arch     fail if command/shared modules regress to god files"
	@echo "  make magic    fail on common production magic-literal regressions"
	@echo "  make positionals fail on production tuple-field/signature regressions"
	@echo "  make oracle-coverage fail if a detection probe lacks a safe-clear (negative-control) test"
	@echo "  make quality  run structural gates"
	@echo "  make fmt      cargo fmt"
	@echo "  make clippy   cargo clippy"
	@echo "  make clean    remove build artifacts"

setup: check release config
	@echo ""
	@echo "rtk ready -> $(BIN)"
	@echo "next: edit $(CONFIG) (base_url + attacker auth headers), then ./$(BIN) --help"

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

config:
	@if [ -f $(CONFIG) ]; then \
		echo "$(CONFIG) already exists — leaving it untouched"; \
	else \
		cp $(EXAMPLE) $(CONFIG) && echo "created $(CONFIG) from $(EXAMPLE) — edit base_url + auth headers"; \
	fi

smoke: release
	@./$(BIN) --version
	@./$(BIN) sign --secret key --payload "The quick brown fox jumps over the lazy dog" 2>/dev/null \
		| grep -q $(SMOKE_VEC) \
		&& echo "smoke OK: HMAC vector matches, binary works" \
		|| { echo "smoke FAIL: unexpected output"; exit 1; }

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

oracle-coverage:
	@sh scripts/check-rust-oracle-coverage.sh

quality: loc arch magic positionals oracle-coverage

fmt:
	$(CARGO) fmt

clippy:
	$(CARGO) clippy --release

clean:
	$(CARGO) clean
