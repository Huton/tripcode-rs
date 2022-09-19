# This file can be used to build in the interim until cargo-features =
# ["different-binary-name"] is stabilized. See comment in bin/Cargo.toml for
# more information.

target/%/tripcode-cli:
	cargo build $$(test "${TARGET}" = "release" && echo "--release") $$CARGO_FLAGS

target/%/tripcode: target/%/tripcode-cli
	mv $< $@

release: TARGET = release
release: target/release/tripcode

debug: TARGET = debug
debug: target/debug/tripcode

all: release

clean:
	rm -fr target/debug/tripcode target/release/tripcode
