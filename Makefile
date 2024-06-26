.PHONY: all test test-unit test-e2e test-e2e nasm-xxd lint lint-fix

SHELL=/bin/bash

files=$(wildcard ./sim8086/src/*.rs ./sim8086/src/*/*.rs)

all: sim8086.bin

sim8086.bin: $(files)
	@echo $(files)
	@cd sim8086 && cargo build --release && mv target/release/sim8086 ../sim8086.bin

test: lint test-unit test-e2e

lint:
	cd sim8086 && cargo clippy

lint-fix:
	cd sim8086 && cargo clippy --fix

test-unit:
	cd sim8086 && cargo test 

test-e2e: sim8086.bin
	@python3 e2e.py $(T)

nasm-xxd:
	@cat n.asm && echo "" && nasm n.asm -o n && xxd n

