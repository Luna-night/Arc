func factorial(n = Int) -> Int {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
print("Factorial of 5 is:");
print(factorial(5));
