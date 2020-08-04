#include "float.h"
#include <stdio.h>

int main() {
  float f1 = 0.1;
  float f2 = 0.2;
  float f3 = f1 + f2;
  printf("%g + %g = %g\n", (double) f1, (double) f2, (double) f3);

  double d1 = 0.1;
  double d2 = 0.2;
  double d3 = d1 + d2;
  printf("%g + %g = %g\n", d1, d2, d3);
}
