pub fn as_mut(t: &T)->&mut T{
    unsafe {&mut * (t as *const T as * mut T)}
}

#[inline(always)]
fn string2cstring(conn_str: String) -> CString {
    unsafe { CString::from_vec_unchecked(conn_str.into_bytes()) }
}
#[inline(always)]
fn charptr2string(c_char_ptr: *const c_char) -> String {
    unsafe { CStr::from_ptr(c_char_ptr).to_string_lossy().to_string() }
}


#[macro_export]
macro_rules! MakeDeref {
    ($name:ident, $target:ident, $field:ident) => {
        impl std::ops::Deref for $name {
            type Target = $target;
            fn deref<'a>(&'a self) -> &'a Self::Target {
                &self.$field
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target {
                &mut self.$field
            }
        }
    };
}