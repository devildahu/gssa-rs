emulator = mgba -l 255
RUST_SRC = $(shell find src -type f)
build_flags = 

.DEFAULT_GOAL: check
.PHONY: run_cargo check

check:
	cargo clippy

# Force running cargo build (and waiting until completion)
# when executing the rule to produce release/gssa-rust.
# Otherwise, the next rule in pracitce does nothing
FORCE: ;

target/thumbv4t-none-eabi/release/gssa-rust: FORCE
	cargo build --release $(build_flags)

target/gssa-rust.gba: target/thumbv4t-none-eabi/release/gssa-rust
	arm-none-eabi-objcopy -O binary \
		target/thumbv4t-none-eabi/release/gssa-rust \
		target/gssa-rust.gba
	gbafix target/gssa-rust.gba

run: target/gssa-rust.gba
	$(emulator) target/gssa-rust.gba
