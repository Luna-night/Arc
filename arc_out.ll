declare i32 @printf(i8*, ...)

@.fmt_int = private unnamed_addr constant [4 x i8] c"%d\0A\00"
@.fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"

@.str_1 = private unnamed_addr constant [5 x i8] c"Fail\00"
@.str_2 = private unnamed_addr constant [8 x i8] c"X is 10\00"
@.str_0 = private unnamed_addr constant [5 x i8] c"Pass\00"
define i32 @main() {
entry:
  %0 = alloca i64
  %1 = alloca i64
  %2 = add i64 0, 85
  store i64 %2, i64* %0
  %3 = load i64, i64* %0
  %4 = add i64 0, 60
  %5 = icmp sge i64 %3, %4
  br i1 %5, label %block_0, label %block_1
block_0:
  %6 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %7 = getelementptr inbounds [5 x i8], [5 x i8]* @.str_0, i64 0, i64 0
  %8 = call i32 (i8*, ...) @printf(i8* %6, i8* %7)
  br label %block_2
block_1:
  %9 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %10 = getelementptr inbounds [5 x i8], [5 x i8]* @.str_1, i64 0, i64 0
  %11 = call i32 (i8*, ...) @printf(i8* %9, i8* %10)
  br label %block_2
block_2:
  %12 = add i64 0, 10
  store i64 %12, i64* %1
  %13 = load i64, i64* %1
  %14 = add i64 0, 10
  %15 = icmp eq i64 %13, %14
  br i1 %15, label %block_3, label %block_4
block_3:
  %16 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %17 = getelementptr inbounds [8 x i8], [8 x i8]* @.str_2, i64 0, i64 0
  %18 = call i32 (i8*, ...) @printf(i8* %16, i8* %17)
  %19 = add i64 0, 20
  store i64 %19, i64* %1
  %20 = load i64, i64* %1
  %21 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  %22 = trunc i64 %20 to i32
  %23 = call i32 (i8*, ...) @printf(i8* %21, i32 %22)
  br label %block_5
block_4:
  br label %block_5
block_5:
  ret i32 0
}
