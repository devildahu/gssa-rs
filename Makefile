emulator = mgba-qt -l 255
TARGET = $(shell cargo metadata --format-version=1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
build_flags =

.DEFAULT_GOAL: check
.PHONY: check

check:
	cargo clippy

# Force running cargo build (and waiting until completion)
# when executing the rule to produce release/gssa-rust.
# Otherwise, the next rule in pracitce does nothing
FORCE: ;

$(TARGET)/thumbv4t-none-eabi/release/gssa-rust: FORCE
	cargo build --release $(build_flags)

build/gssa-rust.gba: $(TARGET)/thumbv4t-none-eabi/release/gssa-rust
	mkdir build || true
	arm-none-eabi-objcopy -O binary \
		$(TARGET)/thumbv4t-none-eabi/release/gssa-rust \
		build/gssa-rust.gba
	gbafix build/gssa-rust.gba

run: build/gssa-rust.gba
	$(emulator) build/gssa-rust.gba
