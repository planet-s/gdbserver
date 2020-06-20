all: minimal-asm redox-asm minimal-c

clean:
	rm minimal-asm redox-asm minimal-c

%-asm.o: %.asm
	nix-shell -p nasm --run "nasm -f elf64 $< -g -o $@"
%-c: %.c
	nix-shell -p musl --run "musl-gcc -g -static $< -o $@"

%: %.o
	ld "$<" -o "$@"
	rm "$<"
