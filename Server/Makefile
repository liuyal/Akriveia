SHELL := /bin/bash

DEBUG_ARGS = --debug
RELEASE_ARGS = --release
FRONTEND_ARGS =  -p frontend --target wasm32-unknown-unknown
BACKEND_ARGS = -p backend

all: build-front build-back

SOURCE_CARGO=source ~/.cargo/env

# build the backend, for independant work from the frontend
build-back:
	$(SOURCE_CARGO) && cargo build $(BACKEND_ARGS)

build-front:
	$(SOURCE_CARGO) && cargo web build $(FRONTEND_ARGS)

deploy: build-front build-back
	$(SOURCE_CARGO) && cargo web deploy -p frontend

# it looks like there is a bug with wasm right now, only frontend release works with yew fetch? https://github.com/DenisKolodin/yew/issues/455
run-back:
	$(SOURCE_CARGO) && cargo web deploy $(FRONTEND_ARGS) $(RELEASE_ARGS)
	$(SOURCE_CARGO) && cargo run $(BACKEND_ARGS)

# start the debug server, independant of the backend.
run-front:
	$(SOURCE_CARGO) && cargo web start $(FRONTEND_ARGS) $(RELEASE_ARGS) --auto-reload --host 0.0.0.0

test:
	$(SOURCE_CARGO) && cargo test $(BACKEND_ARGS) -- --test-threads=1

back: build-back
front: build-front
run: run-back

b: build-back
f: build-front
r: run-back
t: test

