emulator = mgba -l 255
RUST_SRC = $(shell find src -type f)
build_flags = 

.DEFAULT_GOAL: build
.PHONY: run_cargo

run_cargo:
	cargo build --release $(build_flags)

target/thumbv4t-none-eabi/release/gssa-rust: run_cargo

build: target/thumbv4t-none-eabi/release/gssa-rust
	arm-none-eabi-objcopy -O binary \
		target/thumbv4t-none-eabi/release/gssa-rust \
		target/gssa-rust.gba
	gbafix target/gssa-rust.gba

run: build
	$(emulator) target/gssa-rust.gba
