use std::error::Error;

use windows::{core::PCSTR, Win32::Foundation::GetLastError};

pub mod kernel32;
pub mod process;

pub fn get_last_error() -> Box<dyn Error> {
  unsafe { GetLastError().to_hresult().message().to_string().into() }
}

pub fn into_pcstr(s: &str) -> PCSTR {
  PCSTR::from_raw(format!("{}\0", s).as_ptr())
}
