declare i32 @printf(i8*, ...)

@.fmt_int = private unnamed_addr constant [4 x i8] c"%d\0A\00"
@.fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"
@.fmt_float = private unnamed_addr constant [4 x i8] c"%f\0A\00"

@.str_1 = private unnamed_addr constant [19 x i8] c"Factorial of 5 is:\00"
@.str_0 = private unnamed_addr constant [9 x i8] c"5 + 32 =\00"
define i64 @add(i64 %arg0, i64 %arg1) {
entry:
  %t0 = alloca i64
  store i64 %arg0, i64* %t0
  %t1 = alloca i64
  store i64 %arg1, i64* %t1
  %t2 = load i64, i64* %t0
  %t3 = load i64, i64* %t1
  %t4 = add i64 %t2, %t3
  ret i64 %t4
}

define i64 @factorial(i64 %arg0) {
entry:
  %t0 = alloca i64
  store i64 %arg0, i64* %t0
  %t1 = load i64, i64* %t0
  %t2 = add i64 0, 1
  %t3 = icmp sle i64 %t1, %t2
  br i1 %t3, label %block_0, label %block_1
block_0:
  %t4 = add i64 0, 1
  ret i64 %t4
  br label %block_2
block_1:
  br label %block_2
block_2:
  %t5 = load i64, i64* %t0
  %t6 = load i64, i64* %t0
  %t7 = add i64 0, 1
  %t8 = sub i64 %t6, %t7
  %t9 = call i64 @factorial(i64 %t8)
  %t10 = mul i64 %t5, %t9
  ret i64 %t10
}

define i32 @main() {
entry:
  %t0 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %t1 = getelementptr inbounds [9 x i8], [9 x i8]* @.str_0, i64 0, i64 0
  %t2 = call i32 (i8*, ...) @printf(i8* %t0, i8* %t1)
  %t3 = add i64 0, 5
  %t4 = add i64 0, 32
  %t5 = call i64 @add(i64 %t3, i64 %t4)
  %t7 = trunc i64 %t5 to i32
  %t6 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  %t8 = call i32 (i8*, ...) @printf(i8* %t6, i32 %t7)
  %t9 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %t10 = getelementptr inbounds [19 x i8], [19 x i8]* @.str_1, i64 0, i64 0
  %t11 = call i32 (i8*, ...) @printf(i8* %t9, i8* %t10)
  %t12 = add i64 0, 5
  %t13 = call i64 @factorial(i64 %t12)
  %t15 = trunc i64 %t13 to i32
  %t14 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  %t16 = call i32 (i8*, ...) @printf(i8* %t14, i32 %t15)
  ret i32 0
}
