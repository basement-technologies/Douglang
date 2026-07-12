use super::error::RuntimeError;
use super::value::Value;

#[cfg(windows)]
pub fn ffi(func_name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
    use std::ffi::{CString, c_void};

    unsafe extern "system" {
        fn LoadLibraryA(name: *const u8) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    }

    let candidates = ["msvcrt\0", "kernel32\0"];
    let name_cstring = CString::new(func_name)
        .map_err(|_| RuntimeError::new("invalid function name"))?;
    let name_uds = CString::new(format!("_{func_name}"))
        .map_err(|_| RuntimeError::new("invalid function name"))?;

    let mut func_ptr: *mut c_void = std::ptr::null_mut();
    for lib_name in &candidates {
        let module = unsafe { LoadLibraryA(lib_name.as_ptr()) };
        if module.is_null() {
            continue;
        }
        for cand in [name_cstring.as_ptr() as *const u8, name_uds.as_ptr() as *const u8] {
            let addr = unsafe { GetProcAddress(module, cand) };
            if !addr.is_null() {
                func_ptr = addr;
                break;
            }
        }
        if !func_ptr.is_null() {
            break;
        }
    }

    if func_ptr.is_null() {
        return Err(RuntimeError::new(&format!(
            "You are literally trolling. FFI function '{}' not found in any linked library.",
            func_name
        )));
    }

    call_ffi(func_ptr, args)
}

#[cfg(not(windows))]
pub fn ffi(func_name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
    use std::ffi::{CString, c_void};

    unsafe extern "C" {
        fn dlopen(filename: *const i8, flags: i32) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const i8) -> *mut c_void;
    }
    const RTLD_LAZY: i32 = 0x00001;

    let name_cstring = CString::new(func_name)
        .map_err(|_| RuntimeError::new("invalid function name"))?;
    let name_uds = CString::new(format!("_{func_name}"))
        .map_err(|_| RuntimeError::new("invalid function name"))?;

    let lib_names: &[&str] = &["libc.so.6\0", "libc.dylib\0"];
    let mut func_ptr: *mut c_void = std::ptr::null_mut();

    for lib_name in lib_names {
        let handle = unsafe { dlopen(lib_name.as_ptr() as *const i8, RTLD_LAZY) };
        if handle.is_null() {
            continue;
        }
        for cand in [name_cstring.as_ptr(), name_uds.as_ptr()] {
            let addr = unsafe { dlsym(handle, cand) };
            if !addr.is_null() {
                func_ptr = addr;
                break;
            }
        }
        if !func_ptr.is_null() {
            break;
        }
    }

    if func_ptr.is_null() {
        for cand in [name_cstring.as_ptr(), name_uds.as_ptr()] {
            let addr = unsafe { dlsym(std::ptr::null_mut(), cand) };
            if !addr.is_null() {
                func_ptr = addr;
                break;
            }
        }
    }

    if func_ptr.is_null() {
        return Err(RuntimeError::new(&format!(
            "You are literally trolling. FFI function '{}' not found in any linked library.",
            func_name
        )));
    }

    call_ffi(func_ptr, args)
}

fn call_ffi(
    func_ptr: *mut std::ffi::c_void,
    args: &[Value],
) -> Result<Value, RuntimeError> {
    use std::ffi::CString;

    let mut raw: Vec<isize> = Vec::new();
    let mut _keepalive: Vec<CString> = Vec::new();

    for a in args {
        match a {
            Value::Str(s) => {
                let cs = CString::new(s.as_str())
                    .map_err(|_| RuntimeError::new("ffi string contains null byte"))?;
                raw.push(cs.as_ptr() as isize);
                _keepalive.push(cs);
            }
            Value::Float(v) => {
                raw.push(*v as isize);
            }
            Value::Int(v) => {
                raw.push(*v as isize);
            }
        }
    }

    let result: isize = unsafe {
        match raw.len() {
            0 => {
                let f: extern "C" fn() -> isize = std::mem::transmute(func_ptr);
                f()
            }
            1 => {
                let f: extern "C" fn(isize) -> isize = std::mem::transmute(func_ptr);
                f(raw[0])
            }
            2 => {
                let f: extern "C" fn(isize, isize) -> isize = std::mem::transmute(func_ptr);
                f(raw[0], raw[1])
            }
            3 => {
                let f: extern "C" fn(isize, isize, isize) -> isize = std::mem::transmute(func_ptr);
                f(raw[0], raw[1], raw[2])
            }
            4 => {
                let f: extern "C" fn(isize, isize, isize, isize) -> isize = std::mem::transmute(func_ptr);
                f(raw[0], raw[1], raw[2], raw[3])
            }
            _ => {
                return Err(RuntimeError::new("ffi calls with 4+ arguments are not supported"));
            }
        }
    };

    Ok(Value::Int(result as i64))
}
