#include <malloc.h>
#include <stdbool.h>
#include <stdio.h>

int main() {
  while (true) {
    char *buf = malloc(1 >> 20);
  }
  return 0;
}
