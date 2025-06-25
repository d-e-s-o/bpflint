.PHONY: deploy
deploy: build
	@cargo build --features='deploy' --release --target=wasm32-unknown-unknown

.PHONY: build
build:
	@cargo build --release --target=wasm32-unknown-unknown

.PHONY: run
run:
	@cd www && python3 -m http.server
