RFLAGS="-C link-arg=-s"

build: build-buyback build-mock-ref-exchange build-mock-ft

build-buyback: contracts/buyback
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p buyback --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/buyback.wasm ./res/buyback.wasm

build-mock-ref-exchange: contracts/mock-ref-exchange
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-ref-exchange --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_ref_exchange.wasm ./res/mock_ref_exchange.wasm

build-mock-ft: contracts/mock-ft
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-ft --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_ft.wasm ./res/mock_ft.wasm

release:
	$(call docker_build,_rust_setup.sh)
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/buyback.wasm res/buyback_release.wasm

unittest: build
ifdef TC
	RUSTFLAGS=$(RFLAGS) cargo test $(TC) -p buyback --lib -- --nocapture
else
	RUSTFLAGS=$(RFLAGS) cargo test -p buyback --lib -- --nocapture
endif

test: build
ifdef TF
	RUSTFLAGS=$(RFLAGS) cargo test -p buyback --test $(TF) -- --nocapture
else
	RUSTFLAGS=$(RFLAGS) cargo test -p buyback --tests -- --nocapture
endif

clean:
	cargo clean
	rm -rf res/

define docker_build
	docker build -t my-buyback-builder .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t my-buyback-builder \
		/bin/bash $(1)
endef