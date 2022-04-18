#pragma GCC optimize("O0")
#include <malloc.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

int main() {
  while (true)
    memset(malloc(1 >> 10), 0, 1 >> 10);
  return 0;
}
