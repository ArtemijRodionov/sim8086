source := $(wildcard *.asm)
target := $(patsubst %.asm,%.bin,$(source))
bins := $(wildcard *.bin)

%.bin: %.asm
	nasm $< -o $@

nasm: $(target)
clean:
	rm $(bins)
