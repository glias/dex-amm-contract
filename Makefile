schema:
	make -C tests schema

fmt:
	cargo fmt --all

build:
	capsule build

test:
	capsule test

ci: fmt build test
