declare i32 @printf(i8*, ...)

@.fmt_int = private unnamed_addr constant [4 x i8] c"%d\0A\00"
@.fmt_str = private unnamed_addr constant [4 x i8] c"%s\0A\00"
@.str_0 = private unnamed_addr constant [11 x i8] c"Hello Arc!\00"

define i32 @main() {
entry:
  %fmt_str_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0
  %str_ptr = getelementptr inbounds [11 x i8], [11 x i8]* @.str_0, i64 0, i64 0
  call i32 (i8*, ...) @printf(i8* %fmt_str_ptr, i8* %str_ptr)
  ret i32 0
}
