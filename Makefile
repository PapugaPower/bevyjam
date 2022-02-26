default: debug-run

debug:
	cargo build --features bevy/dynamic

debug-run:
	cargo run --features bevy/dynamic

release:
	cargo build --release
