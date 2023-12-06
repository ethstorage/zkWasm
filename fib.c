#include <stdio.h>

long long fibonacci(long long n) {
    const long long rem = 100;
    long first = 0, second = 1, next, i;

    if (n <= 0) return 0;
    if (n == 1) return 1;

    for (i = 2; i <= n; i++) {
        next = (first + second) % rem;
        first = second;
        second = next;
    }

    return second;
}

int main()
{
    printf("fib(30) %lld\n", fibonacci(30));

    return 0;
}

