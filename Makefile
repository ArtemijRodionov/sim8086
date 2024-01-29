.PHONY: test test-unit test-e2e-number test-e2e

SHELL=/bin/bash

files=$(wildcard ./sim8086/src/*.rs ./sim8086/src/*/*.rs)

sim8086.bin: $(files)
	@echo $(files)
	@cd sim8086 && cargo build --release && mv target/release/sim8086 ../sim8086.bin

test-unit:
	cd sim8086 && cargo test 

test-e2e: test-e2e-number sim8086.bin
	@./e2e $(T)

nasm-xxd:
	@cat n.asm && echo "" && nasm n.asm -o n && xxd n

