declare i32 @printf(i8*, ...)

@.fmt_int = private unnamed_addr constant [4 x i8] c"%d\0A\00"
@.fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"
@.fmt_float = private unnamed_addr constant [4 x i8] c"%f\0A\00"

@.str_0 = private unnamed_addr constant [26 x i8] c"Length of 'Hello Arc' is:\00"
@.str_2 = private unnamed_addr constant [24 x i8] c"Square root of 16.0 is:\00"
@.str_1 = private unnamed_addr constant [10 x i8] c"Hello Arc\00"
declare i32 @strlen(i8*)
declare double @sqrt(double)
define i32 @main() {
entry:
  %t0 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %t1 = getelementptr inbounds [26 x i8], [26 x i8]* @.str_0, i64 0, i64 0
  %t2 = call i32 (i8*, ...) @printf(i8* %t0, i8* %t1)
  %t3 = add i64 0, 0
  %t4 = getelementptr inbounds [10 x i8], [10 x i8]* @.str_1, i64 0, i64 0
  %t5 = call i32 @strlen(i8* %t4)
  %t6 = sext i32 %t5 to i64
  %t8 = trunc i64 %t6 to i32
  %t7 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  %t9 = call i32 (i8*, ...) @printf(i8* %t7, i32 %t8)
  %t10 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %t11 = getelementptr inbounds [24 x i8], [24 x i8]* @.str_2, i64 0, i64 0
  %t12 = call i32 (i8*, ...) @printf(i8* %t10, i8* %t11)
  %t13 = fadd double 0.000000e+00, 1.6e1
  %t14 = call double @sqrt(double %t13)
  %t15 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_float, i64 0, i64 0
  %t16 = call i32 (i8*, ...) @printf(i8* %t15, double %t14)
  ret i32 0
}
