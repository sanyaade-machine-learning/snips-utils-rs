use std::ffi::CString;

use libc;
use failure::{Error, ResultExt};

use conversions::*;

#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum SNIPS_RESULT {
    SNIPS_RESULT_OK = 0,
    SNIPS_RESULT_KO = 1,
}

#[repr(C)]
#[derive(Debug)]
pub struct CStringArray {
    pub data: *const *const libc::c_char,
    // Note: we can't use `libc::size_t` because it's not supported by JNA
    pub size: libc::c_int,
}

unsafe impl Sync for CStringArray {}

impl AsRust<Vec<String>> for CStringArray {
    fn as_rust(&self) -> Result<Vec<String>, Error> {
        let mut result = vec![];

        let strings = unsafe {
            ::std::slice::from_raw_parts_mut(
                self.data as *mut *mut libc::c_char,
                self.size as usize,
            )
        };

        for s in strings {
            result.push(create_rust_string_from!(*s))
        }

        Ok(result)
    }
}

impl CReprOf<Vec<String>> for CStringArray {
    fn c_repr_of(input: Vec<String>) -> Result<Self, Error> {
        Ok(Self {
            size: input.len() as libc::c_int,
            data: Box::into_raw(
                input
                    .into_iter()
                    .map(|s| convert_to_c_string_result!(s))
                    .collect::<Result<Vec<*const libc::c_char>, _>>()
                    .context("Could not convert Vector of Strings to C Repr")?
                    .into_boxed_slice(),
            ) as *const *const libc::c_char,
        })
    }
}

impl Drop for CStringArray {
    fn drop(&mut self) {
        let _ = unsafe {
            let y = Box::from_raw(::std::slice::from_raw_parts_mut(
                self.data as *mut *mut libc::c_char,
                self.size as usize,
            ));
            for p in y.into_iter() {
                let _ = CString::from_raw_pointer(*p); // let's not panic if we fail here
            }
        };
    }
}

