#include "float.h"
#include <stdio.h>

int main() {
  volatile float f1 = 0.1;
  volatile float f2 = 0.2;
  float f3 = f1 + f2;
  printf("%g + %g = %g\n", (double) f1, (double) f2, (double) f3);

  volatile double d1 = 0.1;
  volatile double d2 = 0.2;
  double d3 = d1 + d2;
  printf("%g + %g = %g\n", d1, d2, d3);
}
