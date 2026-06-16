func add(a = Int, b = Int) -> Int {
    return a + b;
}

func factorial(n = Int) -> Int {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

print("5 + 32 =");
print(add(5, 32));

print("Factorial of 5 is:");
print(factorial(5));
