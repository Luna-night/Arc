declare i32 @printf(i8*, ...)

@.fmt_int = private unnamed_addr constant [4 x i8] c"%d\0A\00"
@.fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"

@.str_1 = private unnamed_addr constant [10 x i8] c"100 / 4 =\00"
@.str_0 = private unnamed_addr constant [20 x i8] c"Calculation result:\00"

define i32 @main() {
entry:
  %0 = alloca i64
  %1 = alloca i64
  %2 = alloca i64
  %3 = add i64 0, 10
  store i64 %3, i64* %0
  %4 = add i64 0, 32
  store i64 %4, i64* %1
  %5 = load i64, i64* %0
  %6 = load i64, i64* %1
  %7 = add i64 %5, %6
  store i64 %7, i64* %2
  %8 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %9 = getelementptr inbounds [20 x i8], [20 x i8]* @.str_0, i64 0, i64 0
  %10 = call i32 (i8*, ...) @printf(i8* %8, i8* %9)
  %11 = load i64, i64* %2
  %12 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  %13 = trunc i64 %11 to i32
  %14 = call i32 (i8*, ...) @printf(i8* %12, i32 %13)
  %15 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %16 = getelementptr inbounds [10 x i8], [10 x i8]* @.str_1, i64 0, i64 0
  %17 = call i32 (i8*, ...) @printf(i8* %15, i8* %16)
  %18 = add i64 0, 100
  %19 = add i64 0, 4
  %20 = sdiv i64 %18, %19
  %21 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  %22 = trunc i64 %20 to i32
  %23 = call i32 (i8*, ...) @printf(i8* %21, i32 %22)
  ret i32 0
}
