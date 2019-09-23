all: minimal-asm minimal-c

clean:
	rm minimal-asm minimal-c

minimal-asm.o: minimal.asm
	nix-shell -p nasm --run "nasm -f elf64 minimal.asm -g -o minimal-asm.o"
minimal-asm: minimal-asm.o
	ld minimal-asm.o -o minimal-asm
	rm minimal-asm.o

minimal-c: minimal.c
	nix-shell -p musl --run "musl-gcc -g -static minimal.c -o minimal-c"
