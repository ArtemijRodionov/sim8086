.PHONY: test test-unit test-e2e-number test-e2e

SHELL=/bin/bash

files=$(wildcard ./sim8086/*/*.go ./sim8086/*/*/*.go)

sim8086.bin: $(files)
	@echo $(files)
	@cd sim8086 && go build ./cmd/sim8086 && mv sim8086 ../sim8086.bin

test-unit:
	cd sim8086 && go test ./...

test-e2e: test-e2e-number sim8086.bin
	@./e2e $(T)


