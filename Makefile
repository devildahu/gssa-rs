emulator = mgba -l 255
RUST_SRC = $(shell find src -type f)
build_flags = 

.DEFAULT_GOAL: check
.PHONY: run_cargo check

check:
	cargo clippy
	cargo doc
	
run_cargo:
	cargo build --release $(build_flags)

target/thumbv4t-none-eabi/release/gssa-rust: run_cargo

target/gssa-rust.gba: target/thumbv4t-none-eabi/release/gssa-rust
	arm-none-eabi-objcopy -O binary \
		target/thumbv4t-none-eabi/release/gssa-rust \
		target/gssa-rust.gba
	gbafix target/gssa-rust.gba

run: target/gssa-rust.gba
	$(emulator) target/gssa-rust.gba
