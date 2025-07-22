
struct Session<'a> {
    sess: *mut ffi::Session,
    _f: &'a (),
}

fn str_to_cstring(s: &str) -> Result<std::ffi::CString, Error> {
    std::ffi::CString::new(s).map_err(|_| Error)
}

struct Error;

macro_rules! check {
    ($expr:expr) => {{
        let ret = $expr;
        if ret != 0 {
            return Err(Error);
        }
    }};
}

impl Session<'_> {
    //#[rpl::dump_mir(dump_cfg, dump_ddg)]
    pub fn attach(&mut self, table: Option<&str>) -> Result<(), Error> {
        let table = if let Some(table) = table {
            #[allow(dangling_pointers_from_temporaries)]
            str_to_cstring(table)?.as_ptr()
            //~^ NOTE: the `std::ffi::CString` value is dropped here
        } else {
            std::ptr::null()
        };
        unsafe { check!(ffi::sqlite3session_attach(self.sess, table)) };
        //~^ ERROR: use a pointer from `std::ffi::CString` after dropped
        //~| NOTE: `#[deny(rpl::use_after_drop)]` on by default
        Ok(())
    }
}

mod ffi {
    pub type Session = std::ffi::c_void;
    unsafe extern "C" {
        pub fn sqlite3session_attach(s: *mut Session, table: *const std::ffi::c_char) -> i32;
    }
}

fn main() {}