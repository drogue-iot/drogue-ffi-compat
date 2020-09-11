use core::mem::{size_of, transmute};
use core::ffi::c_void;

pub type va_list = __builtin_va_list;
pub type __gnuc_va_list = __builtin_va_list;
pub type __builtin_va_list = __va_list;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __va_list {
    pub __ap: *mut core::ffi::c_void,
}

impl __va_list {
    pub fn as_accessor(va_list: va_list) -> VaList {
        VaList {
            ap: va_list.__ap,
        }
    }
}

pub struct VaList {
    ap: *mut core::ffi::c_void,
}

impl VaList {
    pub fn va_arg<E>(&mut self) -> E {
        unsafe {
            let arg_ptr = self.ap;
            self.ap = self.ap.add(Self::va_argsiz::<E>());
            transmute::<*mut c_void, *const E>(arg_ptr).read()
        }
    }

    fn va_argsiz<E>() -> usize {
        unsafe {
            (((size_of::<E>() + size_of::<u32>() - 1) / size_of::<u32>()) * size_of::<u32>())
        }
    }
}

impl From<__va_list> for VaList {
    fn from(va_list: __va_list) -> Self {
        VaList {
            ap: va_list.__ap
        }
    }
}