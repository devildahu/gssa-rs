emulator = mgba-qt
RUST_SRC = $(shell find src -type f)

.DEFAULT_GOAL = build

run: build
	$(emulator) target/gssa-rust.gba

target:
	mkdir target

target/crt0.o: target crt0.s
	arm-none-eabi-as crt0.s -o target/crt0.o

target/thumbv4-none-agb/release/gssa-rust:  target/crt0.o thumbv4-none-agb.json $(RUST_SRC)
	cargo xbuild --target thumbv4-none-agb.json --release

target/thumbv4-none-agb/debug/gssa-rust:  target/crt0.o thumbv4-none-agb.json $(RUST_SRC)
	cargo xbuild --target thumbv4-none-agb.json

build: target/thumbv4-none-agb/debug/gssa-rust
	arm-none-eabi-objcopy -O binary \
		target/thumbv4-none-agb/debug/gssa-rust \
		target/gssa-rust.gba
	gbafix target/gssa-rust.gba

release: target/thumbv4-none-agb/release/gssa-rust
	arm-none-eabi-objcopy -O binary \
		target/thumbv4-none-agb/release/gssa-rust \
		target/gssa-rust.gba
	gbafix target/gssa-rust.gba

