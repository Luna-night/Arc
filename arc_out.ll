declare i32 @printf(i8*, ...)

@.fmt_int = private unnamed_addr constant [4 x i8] c"%d\0A\00"
@.fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"

@.str_0 = private unnamed_addr constant [15 x i8] c"Loop finished!\00"
define i32 @main() {
entry:
  %0 = alloca i64
  %1 = add i64 0, 0
  store i64 %1, i64* %0
  br label %block_0
block_0:
  %2 = load i64, i64* %0
  %3 = add i64 0, 5
  %4 = icmp slt i64 %2, %3
  br i1 %4, label %block_1, label %block_2
block_1:
  %5 = load i64, i64* %0
  %6 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  %7 = trunc i64 %5 to i32
  %8 = call i32 (i8*, ...) @printf(i8* %6, i32 %7)
  %9 = load i64, i64* %0
  %10 = add i64 0, 1
  %11 = add i64 %9, %10
  store i64 %11, i64* %0
  br label %block_0
block_2:
  %12 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %13 = getelementptr inbounds [15 x i8], [15 x i8]* @.str_0, i64 0, i64 0
  %14 = call i32 (i8*, ...) @printf(i8* %12, i8* %13)
  ret i32 0
}
