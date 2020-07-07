#include <stdio.h>
#include <string.h>
#include <signal.h>

volatile char *msg;
volatile int signal_num;

void sighandler(int sig) {
  msg = "Signal received";
  signal_num = sig;
}

int main() {
  struct sigaction sa;
  memset(&sa, 0, sizeof(struct sigaction));

  sa.sa_handler = sighandler;
  sigaction(SIGUSR1, &sa, NULL);

  puts("Raising signal...");
  raise(SIGUSR1);
  puts("Raised signal");

  printf("%s: %d\n", msg, signal_num);
}
