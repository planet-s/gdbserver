all: minimal-asm redox-asm minimal-c signal-c sighandler-c sigsegv-c float-c

clean:
	rm minimal-asm redox-asm minimal-c signal-c sighandler-c

%-asm.o: %.asm
	nix-shell -p nasm --run "nasm -f elf64 $< -g -o $@"
%-c: %.c
	nix-shell -p musl --run "musl-gcc -g -static $< -o $@"

%: %.o
	ld "$<" -o "$@"
	rm "$<"
