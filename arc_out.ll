declare i32 @printf(i8*, ...)

@.fmt_int = private unnamed_addr constant [4 x i8] c"%d\0A\00"
@.fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"
@.fmt_float = private unnamed_addr constant [4 x i8] c"%f\0A\00"

@.str_0 = private unnamed_addr constant [9 x i8] c"5 + 32 =\00"
@.str_1 = private unnamed_addr constant [19 x i8] c"Factorial of 5 is:\00"
define i64 @add(i64 %arg0, i64 %arg1) {
entry:
  %0 = alloca i64
  store i64 %arg0, i64* %0
  %1 = alloca i64
  store i64 %arg1, i64* %1
  %2 = load i64, i64* %0
  %3 = load i64, i64* %1
  %4 = add i64 %2, %3
  ret i64 %4
}

define i64 @factorial(i64 %arg0) {
entry:
  %0 = alloca i64
  store i64 %arg0, i64* %0
  %1 = load i64, i64* %0
  %2 = add i64 0, 1
  %3 = icmp sle i64 %1, %2
  br i1 %3, label %block_0, label %block_1
block_0:
  %4 = add i64 0, 1
  ret i64 %4
  br label %block_2
block_1:
  br label %block_2
block_2:
  %5 = load i64, i64* %0
  %6 = load i64, i64* %0
  %7 = add i64 0, 1
  %8 = sub i64 %6, %7
  %9 = call i64 @factorial(i64 %8)
  %10 = mul i64 %5, %9
  ret i64 %10
}

define i32 @main() {
entry:
  %0 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %1 = getelementptr inbounds [9 x i8], [9 x i8]* @.str_0, i64 0, i64 0
  %2 = call i32 (i8*, ...) @printf(i8* %0, i8* %1)
  %3 = add i64 0, 5
  %4 = add i64 0, 32
  %5 = call i64 @add(i64 %3, i64 %4)
  %7 = trunc i64 %5 to i32
  %6 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  %8 = call i32 (i8*, ...) @printf(i8* %6, i32 %7)
  %9 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %10 = getelementptr inbounds [19 x i8], [19 x i8]* @.str_1, i64 0, i64 0
  %11 = call i32 (i8*, ...) @printf(i8* %9, i8* %10)
  %12 = add i64 0, 5
  %13 = call i64 @factorial(i64 %12)
  %15 = trunc i64 %13 to i32
  %14 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  %16 = call i32 (i8*, ...) @printf(i8* %14, i32 %15)
  ret i32 0
}
