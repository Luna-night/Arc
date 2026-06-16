use pyo3::prelude::*;
use pyo3::types::PyModule;

// 初始化 Python（pyo3 的 auto-initialize 特性会自动处理）

/// 通用的 Python 函数调用包装器
pub fn call_python_function(module_name: &str, func_name: &str, args: Vec<f64>) -> PyResult<f64> {
    Python::with_gil(|py| {
        // 导入模块
        let module = PyModule::import(py, module_name)?;
        // 获取函数
        let func = module.getattr(func_name)?;
        // 调用函数（目前只支持 Float 参数）
        let py_args = args.iter().map(|&x| x).collect::<Vec<_>>();
        let result = func.call1(py_args)?;
        // 提取返回值
        result.extract()
    })
}

// 为每个 bridge py 声明生成一个 Rust 包装函数
// 这将在编译时通过宏或代码生成实现，