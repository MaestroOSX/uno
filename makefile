.PHONY: build install clean run help

help:
	@echo "uno - Maestro package manager"
	@echo ""
	@echo "Usage:"
	@echo "  make build    - Build the project"
	@echo "  make install  - Install uno globally"
	@echo "  make clean    - Remove build artifacts"
	@echo "  make run      - Run uno"

build:
	cargo build --release

install: build
	cargo install --path .

clean:
	cargo clean

run:
	cargo run --