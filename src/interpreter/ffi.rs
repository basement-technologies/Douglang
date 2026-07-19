use crate::runtime::{self, FfiArgKind, FfiReturnKind, RuntimeError};
use crate::values::Value;

#[cfg(windows)]
pub fn ffi(func_name: &str, args: &[Value], linked_libs: &[String]) -> Result<Value, RuntimeError> {
    use std::ffi::{CString, c_void};

    unsafe extern "system" {
        fn LoadLibraryA(name: *const u8) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    }

    let name_cstring =
        CString::new(func_name).map_err(|_| RuntimeError::new("invalid function name"))?;
    let name_uds = CString::new(format!("_{func_name}"))
        .map_err(|_| RuntimeError::new("invalid function name"))?;
    let candidates = [
        name_cstring.as_ptr() as *const u8,
        name_uds.as_ptr() as *const u8,
    ];

    let default_libs = ["kernel32", "msvcrt"];
    let mut modules: Vec<*mut c_void> = Vec::new();

    for lib in default_libs.iter() {
        let lib_cstr =
            CString::new(lib.as_bytes()).map_err(|_| RuntimeError::new("invalid library name"))?;
        let module = unsafe { LoadLibraryA(lib_cstr.as_ptr() as *const u8) };
        if !module.is_null() {
            modules.push(module);
        }
    }

    for lib in linked_libs {
        let is_path =
            lib.contains('/') || lib.contains('\\') || lib.contains(".dll") || lib.contains(".so");
        if is_path {
            if let Ok(lib_cstr) = CString::new(lib.as_bytes()) {
                let module = unsafe { LoadLibraryA(lib_cstr.as_ptr() as *const u8) };
                if !module.is_null() {
                    modules.push(module);
                }
            }
        } else {
            for candidate in [format!("{lib}.dll"), format!("lib{lib}.dll")] {
                if let Ok(lib_cstr) = CString::new(candidate.as_bytes()) {
                    let module = unsafe { LoadLibraryA(lib_cstr.as_ptr() as *const u8) };
                    if !module.is_null() {
                        modules.push(module);
                        break;
                    }
                }
            }
        }
    }

    for &module in &modules {
        for &cand in &candidates {
            let addr = unsafe { GetProcAddress(module, cand) };
            if !addr.is_null() {
                return call_ffi(func_name, addr, args);
            }
        }
    }

    Err(RuntimeError::new(&format!(
        "You are literally trolling. FFI function '{}' not found in any linked library.",
        func_name
    )))
}

#[cfg(not(windows))]
pub fn ffi(func_name: &str, args: &[Value], linked_libs: &[String]) -> Result<Value, RuntimeError> {
    use std::ffi::{CString, c_void};

    unsafe extern "C" {
        fn dlopen(filename: *const i8, flags: i32) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const i8) -> *mut c_void;
    }
    const RTLD_LAZY: i32 = 0x00001;

    let name_cstring =
        CString::new(func_name).map_err(|_| RuntimeError::new("invalid function name"))?;
    let name_uds = CString::new(format!("_{func_name}"))
        .map_err(|_| RuntimeError::new("invalid function name"))?;
    let candidates = [name_cstring.as_ptr(), name_uds.as_ptr()];

    let mut handles: Vec<*mut c_void> = Vec::new();

    let default_libs = ["libc.so.6", "libc.dylib"];

    for lib in default_libs.iter().copied() {
        let lib_cstr =
            CString::new(lib.as_bytes()).map_err(|_| RuntimeError::new("invalid library name"))?;
        let handle = unsafe { dlopen(lib_cstr.as_ptr() as *const i8, RTLD_LAZY) };
        if !handle.is_null() {
            handles.push(handle);
        }
    }

    for lib in linked_libs {
        let is_path = lib.contains('/')
            || lib.contains(".so")
            || lib.contains(".dylib")
            || lib.contains(".dll");
        if is_path {
            if let Ok(lib_cstr) = CString::new(lib.as_bytes()) {
                let handle = unsafe { dlopen(lib_cstr.as_ptr() as *const i8, RTLD_LAZY) };
                if !handle.is_null() {
                    handles.push(handle);
                }
            }
        } else {
            for pat in [
                format!("lib{lib}.so"),
                format!("lib{lib}.so.6"),
                format!("lib{lib}.so.1"),
                format!("lib{lib}.dylib"),
                lib.clone(),
            ] {
                if let Ok(lib_cstr) = CString::new(pat.as_bytes()) {
                    let handle = unsafe { dlopen(lib_cstr.as_ptr() as *const i8, RTLD_LAZY) };
                    if !handle.is_null() {
                        handles.push(handle);
                        break;
                    }
                }
            }
        }
    }

    for &handle in &handles {
        for &cand in &candidates {
            let addr = unsafe { dlsym(handle, cand) };
            if !addr.is_null() {
                return call_ffi(func_name, addr, args);
            }
        }
    }

    for &cand in &candidates {
        let addr = unsafe { dlsym(std::ptr::null_mut(), cand) };
        if !addr.is_null() {
            return call_ffi(func_name, addr, args);
        }
    }

    Err(RuntimeError::new(&format!(
        "You are literally trolling. FFI function '{}' not found in any linked library.",
        func_name
    )))
}

fn call_ffi(
    func_name: &str,
    func_ptr: *mut std::ffi::c_void,
    args: &[Value],
) -> Result<Value, RuntimeError> {
    use libffi::high::call::*;
    use std::ffi::{CString, c_char, c_int, c_uint};

    enum PreparedArg {
        Double(f64),
        String(*const c_char),
        UInt(c_uint),
    }

    let signature = runtime::ffi_signature(func_name, args.len());
    let mut keepalive: Vec<CString> = Vec::new();
    let mut prepared = Vec::new();

    for (value, kind) in args.iter().zip(signature.arg_kinds) {
        match kind {
            FfiArgKind::Double => prepared.push(PreparedArg::Double(value.clone().into())),
            FfiArgKind::UInt => {
                let x: f64 = value.clone().into();
                prepared.push(PreparedArg::UInt(x as c_uint));
            }
            FfiArgKind::String => {
                let text = value.to_string();
                let cs = CString::new(text.as_str())
                    .map_err(|_| RuntimeError::new("ffi string contains null byte"))?;
                prepared.push(PreparedArg::String(cs.as_ptr()));
                keepalive.push(cs);
            }
        }
    }

    let fn_args: Vec<Arg<'_>> = prepared
        .iter()
        .map(|prepared| match prepared {
            PreparedArg::Double(v) => arg(v),
            PreparedArg::String(v) => arg(v),
            PreparedArg::UInt(v) => arg(v),
        })
        .collect();

    match signature.return_kind {
        FfiReturnKind::Int => {
            let result: c_int = unsafe { call(CodePtr(func_ptr), &fn_args) };
            Ok(Value::Number(result as f64))
        }
        FfiReturnKind::UInt => {
            let result: c_uint = unsafe { call(CodePtr(func_ptr), &fn_args) };
            Ok(Value::Number(result as f64))
        }
        FfiReturnKind::Double => {
            let result: f64 = unsafe { call(CodePtr(func_ptr), &fn_args) };
            Ok(Value::Number(result as f64))
        }
    }
}
