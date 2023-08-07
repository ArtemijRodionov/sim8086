source := $(wildcard *.asm)
target := $(patsubst %.asm,%.o,$(source))

all: nasm
nasm: $(target)
$(target): $(source)
	nasm $< -o $@

clean:
	rm $(target)
