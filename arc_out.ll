declare i32 @printf(i8*, ...)

@.fmt_int = private unnamed_addr constant [4 x i8] c"%d\0A\00"
@.fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"
@.fmt_float = private unnamed_addr constant [4 x i8] c"%f\0A\00"

@.str_1 = private unnamed_addr constant [10 x i8] c"Hello Arc\00"
@.str_0 = private unnamed_addr constant [26 x i8] c"Length of 'Hello Arc' is:\00"
@.str_2 = private unnamed_addr constant [24 x i8] c"Square root of 16.0 is:\00"
declare i32 @strlen(i8*)
declare double @sqrt(double)
define i32 @main() {
entry:
  %0 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %1 = getelementptr inbounds [26 x i8], [26 x i8]* @.str_0, i64 0, i64 0
  %2 = call i32 (i8*, ...) @printf(i8* %0, i8* %1)
  %3 = add i64 0, 0
  %4 = getelementptr inbounds [10 x i8], [10 x i8]* @.str_1, i64 0, i64 0
  %5 = call i32 @strlen(i8* %4)
  %6 = sext i32 %5 to i64
  %7 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0
  %8 = trunc i64 %6 to i32
  %9 = call i32 (i8*, ...) @printf(i8* %7, i32 %8)
  %10 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %11 = getelementptr inbounds [24 x i8], [24 x i8]* @.str_2, i64 0, i64 0
  %12 = call i32 (i8*, ...) @printf(i8* %10, i8* %11)
  %13 = fadd double 0.000000e+00, 1.6e1
  %14 = call double @sqrt(double %13)
  %15 = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_float, i64 0, i64 0
  %16 = call i32 (i8*, ...) @printf(i8* %15, double %14)
  ret i32 0
}
