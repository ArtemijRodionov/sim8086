source := $(wildcard *.asm)
target := $(patsubst %.asm,%.o,$(source))

%.o: %.asm
	nasm $< -o $@

nasm: $(target)
clean:
	rm $(target)
