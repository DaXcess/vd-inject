use std::error::Error;

use windows::Win32::{
  Foundation::{CloseHandle, FARPROC, HANDLE, WIN32_ERROR},
  System::{
    LibraryLoader::{GetProcAddress, LoadLibraryA},
    Threading::WaitForSingleObject,
  },
};

use crate::win32::into_pcstr;

pub fn get_proc_address(module_name: &str, proc_name: &str) -> Result<FARPROC, Box<dyn Error>> {
  unsafe {
    let module = LoadLibraryA(into_pcstr(module_name))?;

    Ok(GetProcAddress(module, into_pcstr(proc_name)))
  }
}

pub fn wait_for_object(handle: HANDLE, millis: u32) -> WIN32_ERROR {
  unsafe { WaitForSingleObject(handle, millis) }
}

pub fn close_handle(handle: HANDLE) -> bool {
  unsafe { CloseHandle(handle).as_bool() }
}
