bridge c "libc.so.6" {
    func strlen(s = String) -> Int
}

bridge c "libm.so.6" {
    func sqrt(x = Float) -> Float
}

print("Length of 'Hello Arc' is:");
print(strlen("Hello Arc"));

print("Square root of 16.0 is:");
print(sqrt(16.0));
