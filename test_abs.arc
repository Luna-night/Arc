bridge c "libc.so.6" {
    func abs(x = Int) -> Int
}

print("Absolute value of -42 is:")
print(abs(-42))
