declare i32 @printf(i8*, ...)

@.fmt_int = private unnamed_addr constant [4 x i8] c"%d\0A\00"

define i32 @main() {
entry:
  %fmt_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  call i32 (i8*, ...) @printf(i8* %fmt_ptr, i32 42)
  ret i32 0
}
