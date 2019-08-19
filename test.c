// Compile with: musl-gcc -static -g test.c
#include <stdio.h>

int main() {
    for (int i = 0; i < 5; ++i) {
        puts("Hello World");
    }
}
