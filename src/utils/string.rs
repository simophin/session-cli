use std::ffi::{c_char, CStr, CString};

pub trait StringExt {
    fn to_cstr(self) -> impl AsRef<CStr>;
}

impl StringExt for &'_ str {
    fn to_cstr(self) -> impl AsRef<CStr> {
        CString::new(self.to_string()).expect("Failed to convert string to CString")
    }
}

impl StringExt for String {
    fn to_cstr(self) -> impl AsRef<CStr> {
        CString::new(self).expect("Failed to convert string to CString")
    }
}

pub trait CArrayExt {
    fn cstr_to_str(&self) -> Option<&str>;
    fn write_cstr(&mut self, s: &str) -> bool;
}

impl<const N: usize> CArrayExt for [c_char; N] {
    fn cstr_to_str(&self) -> Option<&str> {
        let cstr = CStr::from_bytes_until_nul(unsafe {
            std::slice::from_raw_parts(self.as_ptr() as *const u8, N)
        })
        .ok()?;

        cstr.to_str().ok()
    }

    fn write_cstr(&mut self, s: &str) -> bool {
        let bytes = s.as_bytes();
        if bytes.len() >= N {
            return false;
        }

        self[..bytes.len()].copy_from_slice(unsafe { std::mem::transmute(bytes) });
        self[bytes.len()] = 0;
        true
    }
}

impl<const N: usize> CArrayExt for [u8; N] {
    fn cstr_to_str(&self) -> Option<&str> {
        CStr::from_bytes_until_nul(self.as_slice())
            .ok()?
            .to_str()
            .ok()
    }

    fn write_cstr(&mut self, s: &str) -> bool {
        let bytes = s.as_bytes();
        if bytes.len() >= N {
            return false;
        }

        self[..bytes.len()].copy_from_slice(bytes);
        self[bytes.len()] = 0;
        true
    }
}
