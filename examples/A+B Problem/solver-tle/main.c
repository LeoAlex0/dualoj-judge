#include <stdbool.h>
#include <stdio.h>

int main() {
  char buf[50];
  while (true) {
    fgets(buf, sizeof(buf), stdin);
  }
  return 0;
}