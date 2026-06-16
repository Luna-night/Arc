bridge c "libc.so.6" {
    func getpid() -> Int
}

print("My Process ID is:")
print(getpid())
