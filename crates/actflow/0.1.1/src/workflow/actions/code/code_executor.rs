use regex::Regex;
use rquickjs::{Context as JsContext, FromJs, Runtime as JsRuntime};
use rustpython_vm::{
    AsObject, Interpreter, PyObjectRef, VirtualMachine,
    builtins::{PyDict, PyFloat, PyInt, PyList, PyStr},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ActflowError, Result};

/// Code language
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeLanguage {
    Python3,
    Javascript,
}

pub struct JavascriptExecutor;

impl JavascriptExecutor {
    /// Execute JavaScript code with parameters and return the result
    ///
    /// # Arguments
    /// * `code` - JavaScript code containing a function (function name is auto-detected)
    /// * `params` - Parameters to pass to the function as a JSON object
    ///
    /// # Returns
    /// * `Result<Value>` - The return value from the function
    pub fn execute(
        code: &str,
        params: Value,
    ) -> Result<Value> {
        // Auto-detect function name from code
        let func_name = Self::extract_javascript_function_name(code).ok_or_else(|| ActflowError::Runtime("No function found in code".to_string()))?;

        let runtime = JsRuntime::new().map_err(|e| ActflowError::Runtime(e.to_string()))?;
        let ctx = JsContext::full(&runtime).map_err(|e| ActflowError::Runtime(e.to_string()))?;

        ctx.with(|ctx| {
            // Evaluate the code to define the function
            if let Err(rquickjs::Error::Exception) = ctx.eval::<(), _>(code) {
                let exception = rquickjs::Exception::from_js(&ctx, ctx.catch()).unwrap();
                return Err(ActflowError::Exception {
                    ecode: "JS_EVAL_ERROR".to_string(),
                    message: exception.message().unwrap_or_default(),
                });
            }

            // Call function with params
            let params_json = serde_json::to_string(&params).unwrap_or_default();
            let call_code = format!("JSON.stringify({}({}))", func_name, params_json);

            let result: std::result::Result<String, _> = ctx.eval(call_code);
            match result {
                Ok(json_str) => serde_json::from_str(&json_str).map_err(|e| ActflowError::Runtime(e.to_string())),
                Err(rquickjs::Error::Exception) => {
                    let exception = rquickjs::Exception::from_js(&ctx, ctx.catch()).unwrap();
                    Err(ActflowError::Exception {
                        ecode: "JS_EXEC_ERROR".to_string(),
                        message: exception.message().unwrap_or_default(),
                    })
                }
                Err(e) => Err(ActflowError::Runtime(e.to_string())),
            }
        })
    }

    /// Extract the first function name from JavaScript code
    fn extract_javascript_function_name(code: &str) -> Option<String> {
        // Match: function functionName(...) or function functionName (...
        let re = Regex::new(r"function\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").ok()?;
        re.captures(code).map(|caps| caps[1].to_string())
    }
}

pub struct PythonExecutor;

impl PythonExecutor {
    /// Execute Python code with parameters and return the result
    pub fn execute(
        code: &str,
        params: Value,
    ) -> Result<Value> {
        // Auto-detect function name from code
        let func_name = Self::extract_python_function_name(code).ok_or_else(|| ActflowError::Runtime("No function found in Python code".to_string()))?;

        Interpreter::without_stdlib(Default::default()).enter(|vm| {
            let scope = vm.new_scope_with_builtins();

            // Execute the user code to define the function
            let code_obj = vm.compile(code, rustpython_vm::compiler::Mode::Exec, "<embedded>".to_owned()).map_err(|e| ActflowError::Exception {
                ecode: "PY_COMPILE_ERROR".to_string(),
                message: format!("{:?}", e),
            })?;

            vm.run_code_obj(code_obj, scope.clone()).map_err(|e| ActflowError::Exception {
                ecode: "PY_EXEC_ERROR".to_string(),
                message: format!("{:?}", e),
            })?;

            // Get the function from scope
            let func = scope.globals.get_item(&func_name, vm).map_err(|e| ActflowError::Runtime(format!("{:?}", e)))?;

            // Convert params to Python dict
            let py_params = Self::json_to_pyobject(vm, &params);

            // Call the function with the params dict
            let result = func.call((py_params,), vm).map_err(|e| ActflowError::Exception {
                ecode: "PY_CALL_ERROR".to_string(),
                message: format!("{:?}", e),
            })?;

            // Convert result back to JSON
            Self::pyobject_to_json(vm, &result)
        })
    }

    /// Extract the first function name from Python code
    fn extract_python_function_name(code: &str) -> Option<String> {
        // Match: def function_name(...) or def function_name (...
        let re = Regex::new(r"def\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").ok()?;
        re.captures(code).map(|caps| caps[1].to_string())
    }

    /// Convert serde_json::Value to Python object
    fn json_to_pyobject(
        vm: &VirtualMachine,
        value: &Value,
    ) -> PyObjectRef {
        match value {
            Value::Null => vm.ctx.none(),
            Value::Bool(b) => vm.ctx.new_bool(*b).into(),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    vm.ctx.new_int(i).into()
                } else if let Some(f) = n.as_f64() {
                    vm.ctx.new_float(f).into()
                } else {
                    vm.ctx.none()
                }
            }
            Value::String(s) => vm.ctx.new_str(s.as_str()).into(),
            Value::Array(arr) => {
                let py_list: Vec<PyObjectRef> = arr.iter().map(|v| Self::json_to_pyobject(vm, v)).collect();
                vm.ctx.new_list(py_list).into()
            }
            Value::Object(obj) => {
                let py_dict = vm.ctx.new_dict();
                for (k, v) in obj {
                    py_dict.set_item(k.as_str(), Self::json_to_pyobject(vm, v), vm).unwrap();
                }
                py_dict.into()
            }
        }
    }

    /// Convert Python object to serde_json::Value
    fn pyobject_to_json(
        vm: &VirtualMachine,
        obj: &PyObjectRef,
    ) -> Result<Value> {
        if vm.is_none(obj) {
            return Ok(Value::Null);
        }

        // Check bool first (before int, since bool is a subclass of int in Python)
        // In RustPython, bool is a subclass of int, so we check the type name
        if obj.fast_isinstance(vm.ctx.types.bool_type) {
            // Check if it's the true_value singleton
            let is_true = obj.is(&vm.ctx.true_value);
            return Ok(Value::Bool(is_true));
        }

        if let Some(i) = obj.payload::<PyInt>() {
            if let Ok(n) = i.try_to_primitive::<i64>(vm) {
                return Ok(Value::Number(n.into()));
            }
        }

        if let Some(f) = obj.payload::<PyFloat>() {
            if let Some(n) = serde_json::Number::from_f64(f.to_f64()) {
                return Ok(Value::Number(n));
            }
        }

        if let Some(s) = obj.payload::<PyStr>() {
            return Ok(Value::String(s.as_str().to_string()));
        }

        if let Some(list) = obj.payload::<PyList>() {
            let mut arr = Vec::new();
            for item in list.borrow_vec().iter() {
                arr.push(Self::pyobject_to_json(vm, item)?);
            }
            return Ok(Value::Array(arr));
        }

        if let Some(dict) = obj.payload::<PyDict>() {
            let mut map = serde_json::Map::new();
            for (k, v) in dict {
                let key_str = k.payload::<PyStr>().map(|s| s.as_str().to_string()).unwrap_or_else(|| format!("{:?}", k));
                map.insert(key_str, Self::pyobject_to_json(vm, &v)?);
            }
            return Ok(Value::Object(map));
        }

        // Fallback: try to get string representation
        Ok(Value::String(format!("{:?}", obj)))
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::{JavascriptExecutor, PythonExecutor};

    #[test]
    fn test_extract_function_name() {
        assert_eq!(
            JavascriptExecutor::extract_javascript_function_name("function main() {}"),
            Some("main".to_string())
        );
        assert_eq!(
            JavascriptExecutor::extract_javascript_function_name("function processData({a}) {}"),
            Some("processData".to_string())
        );
        assert_eq!(
            JavascriptExecutor::extract_javascript_function_name("  function  _helper123 () {}"),
            Some("_helper123".to_string())
        );
        assert_eq!(JavascriptExecutor::extract_javascript_function_name("const x = 1;"), None);
    }

    #[test]
    fn test_execute_with_params() {
        let javascript_code = r#"
        function main({arg1, arg2}) {
            return {
                result: arg1 + arg2
            }
        }
        "#;

        let params = json!({"arg1": 10, "arg2": 20});
        let result = JavascriptExecutor::execute(javascript_code, params).unwrap();

        assert_eq!(result, json!({"result": 30}));
    }

    #[test]
    fn test_execute_string_concat() {
        let javascript_code = r#"
        function main({name, greeting}) {
            return {
                message: greeting + ", " + name + "!"
            }
        }
        "#;

        let params = json!({"name": "World", "greeting": "Hello"});
        let result = JavascriptExecutor::execute(javascript_code, params).unwrap();

        assert_eq!(result, json!({"message": "Hello, World!"}));
    }

    #[test]
    fn test_execute_with_console() {
        let javascript_code = r#"
        function main({value}) {
            console.log("value:" +value);
            console.log("Processing value: " + value);
            return { doubled: value * 2 }
        }
        "#;

        let params = json!({"value": 5});
        let result = JavascriptExecutor::execute(javascript_code, params).unwrap();

        assert_eq!(result, json!({"doubled": 10}));
    }

    #[test]
    fn test_execute_json_format() {
        // value is already a JS object, no need to JSON.parse
        let javascript_code = r#"
        function main({value}) {
            console.log("value:" + JSON.stringify(value));
            return { status_code: value.status_code, message: value.body.message }
        }
        "#;

        let params = json!({"value": {"status_code": 200, "body": {"message": "Hello World"}}});
        let result = JavascriptExecutor::execute(javascript_code, params).unwrap();

        assert_eq!(result, json!({"status_code": 200, "message": "Hello World"}));
    }

    #[test]
    fn test_execute_custom_function_name() {
        // Function name is auto-detected from code
        let javascript_code = r#"
        function processData({json_str}) {
            const json = JSON.parse(json_str);
            return { status_code: json.status_code, message: json.body.message }
        }
        "#;

        let json_value = r#"{"status_code": 200, "body": {"message": "Hello World"}}"#;
        let params = json!({"json_str": json_value});
        let result = JavascriptExecutor::execute(javascript_code, params).unwrap();

        assert_eq!(result, json!({"status_code": 200, "message": "Hello World"}));
    }

    // Python tests
    #[test]
    fn test_python_extract_function_name() {
        assert_eq!(
            PythonExecutor::extract_python_function_name("def main(): pass"),
            Some("main".to_string())
        );
        assert_eq!(
            PythonExecutor::extract_python_function_name("def process_data(params): pass"),
            Some("process_data".to_string())
        );
        assert_eq!(
            PythonExecutor::extract_python_function_name("  def  _helper123 (): pass"),
            Some("_helper123".to_string())
        );
        assert_eq!(PythonExecutor::extract_python_function_name("x = 1"), None);
    }

    #[test]
    fn test_python_execute_with_params() {
        let python_code = r#"
def main(params):
    return {"result": params["arg1"] + params["arg2"]}
"#;

        let params = json!({"arg1": 10, "arg2": 20});
        let result = PythonExecutor::execute(python_code, params).unwrap();

        assert_eq!(result, json!({"result": 30}));
    }

    #[test]
    fn test_python_execute_string_concat() {
        let python_code = r#"
def greet(params):
    return {"message": params["greeting"] + ", " + params["name"] + "!"}
"#;

        let params = json!({"name": "World", "greeting": "Hello"});
        let result = PythonExecutor::execute(python_code, params).unwrap();

        assert_eq!(result, json!({"message": "Hello, World!"}));
    }

    #[test]
    fn test_python_execute_with_list() {
        let python_code = r#"
def process(params):
    numbers = params["numbers"]
    return {"sum": sum(numbers), "count": len(numbers)}
"#;

        let params = json!({"numbers": [1, 2, 3, 4, 5]});
        let result = PythonExecutor::execute(python_code, params).unwrap();

        assert_eq!(result, json!({"sum": 15, "count": 5}));
    }

    #[test]
    fn test_python_execute_with_bool() {
        let python_code = r#"
def check(params):
    return {"is_valid": params["value"] > 10, "is_negative": params["value"] < 0}
"#;

        let params = json!({"value": 15});
        let result = PythonExecutor::execute(python_code, params).unwrap();

        assert_eq!(result, json!({"is_valid": true, "is_negative": false}));
    }
}
