use std::borrow::Cow;
use cpp::cpp;

cpp! {{
    #include <iostream>
    #include <string>
    #include <cstring>
    typedef void* (*memset_t)(void*, int, size_t);
    static volatile memset_t memset_func = memset;
}}

pub struct CppSecureString {
    raw_ptr: *mut core::ffi::c_void,
}

impl CppSecureString {
    pub fn new() -> Self {
        let raw_ptr = unsafe {
            cpp!([] -> *mut core::ffi::c_void as "std::string *" {
                return new std::string();
            })
        };
        Self { raw_ptr }
    }

    pub fn raw_ptr(&mut self) -> *mut core::ffi::c_void {
        self.raw_ptr
    }

    pub fn to_str(&self) -> Cow<'_, str> {
        let ptr = self.raw_ptr;
        let c_ptr: *const core::ffi::c_char = unsafe {
            cpp!([ptr as "std::string *"] -> *const core::ffi::c_char as "const char *" {
                return ptr->c_str();
            })
        };
        let c_str = unsafe { std::ffi::CStr::from_ptr(c_ptr) };
        c_str.to_string_lossy()
    }

    pub fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}

impl Drop for CppSecureString {
    fn drop(&mut self) {
        let ptr = self.raw_ptr;
        unsafe {
            cpp!([ptr as "std::string *"] {
                ptr->resize(ptr->capacity(), 0);
                memset_func((void *)ptr->c_str(), 0, ptr->length());
                delete ptr;
            });
        }
    }
}

fn main() {
    let mut cpp_str = CppSecureString::new();
    let str_ptr = cpp_str.raw_ptr();
    unsafe {
        cpp!([str_ptr as "std::string *"] {
           str_ptr->assign("my internal c++ result");
        });
    }


    let s1 = cpp_str.to_str();
    let rust_str = cpp_str.to_string();
    println!("cow: {s1}");
    drop(cpp_str);
    println!("my rust string: {rust_str}");

}