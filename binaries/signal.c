#include <stdio.h>
#include <signal.h>

int main() {
  puts("Raising signal...");
  raise(SIGUSR1);
  puts("Raised signal");
}
