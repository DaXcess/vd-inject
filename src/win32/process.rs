use std::error::Error;

use ntapi::ntpsapi::{NtResumeProcess, NtSuspendProcess};
use windows::Win32::{
  Foundation::{BOOL, HANDLE},
  System::{
    Diagnostics::{
      Debug::WriteProcessMemory,
      ToolHelp::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
      },
    },
    Memory::{VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE},
    Threading::{
      CreateRemoteThread, IsWow64Process, OpenProcess, LPTHREAD_START_ROUTINE, PROCESS_ALL_ACCESS,
    },
  },
};

use super::get_last_error;

pub struct Process {
  handle: HANDLE,
}

impl Process {
  pub fn open(id: u32) -> Result<Self, Box<dyn Error>> {
    unsafe {
      let handle = OpenProcess(PROCESS_ALL_ACCESS, false, id)?;

      Ok(Self { handle })
    }
  }

  pub fn suspend(&self) {
    unsafe {
      NtSuspendProcess(self.handle.0 as *mut ntapi::winapi::ctypes::c_void);
    };
  }

  pub fn resume(&self) {
    unsafe {
      NtResumeProcess(self.handle.0 as *mut ntapi::winapi::ctypes::c_void);
    }
  }

  pub fn is_wow64(&self) -> Result<bool, Box<dyn Error>> {
    let mut is_wow64 = BOOL::default();

    unsafe {
      if !IsWow64Process(self.handle, &mut is_wow64).as_bool() {
        return Err(get_last_error());
      }
    }

    Ok(is_wow64.as_bool())
  }

  pub fn mem_allocate(&self, size: usize) -> *mut core::ffi::c_void {
    unsafe {
      VirtualAllocEx(
        self.handle,
        None,
        size,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_READWRITE,
      )
    }
  }

  pub fn mem_free(&self, address: *mut core::ffi::c_void) -> Result<(), Box<dyn Error>> {
    unsafe {
      if !VirtualFreeEx(self.handle, address, 0, MEM_RELEASE).as_bool() {
        return Err(get_last_error());
      }
    }

    Ok(())
  }

  pub fn mem_write(
    &self,
    address: *mut core::ffi::c_void,
    buffer: *mut core::ffi::c_void,
    size: usize,
  ) -> Result<(), Box<dyn Error>> {
    unsafe {
      if !WriteProcessMemory(self.handle, address, buffer, size, None).as_bool() {
        return Err(get_last_error());
      }
    }

    Ok(())
  }

  pub fn thread_create(
    &self,
    startaddress: LPTHREAD_START_ROUTINE,
    parameter: core::option::Option<*const core::ffi::c_void>,
  ) -> Result<HANDLE, Box<dyn Error>> {
    unsafe {
      Ok(CreateRemoteThread(
        self.handle,
        None,
        0,
        startaddress,
        parameter,
        0,
        None,
      )?)
    }
  }
}

pub fn get_process_ids_by_name(name: &str) -> Result<Vec<u32>, Box<dyn Error>> {
  let mut entry = PROCESSENTRY32 {
    dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
    ..Default::default()
  };

  let mut processes = Vec::new();

  unsafe {
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;

    if Process32First(snapshot, &mut entry).as_bool() {
      while Process32Next(snapshot, &mut entry).as_bool() {
        let range_end = entry.szExeFile.iter().position(|&c| c.0 == 0).unwrap_or(0);
        let exe_file = String::from_utf8(entry.szExeFile.map(|e| e.0)[..range_end].to_vec())?;

        if exe_file == name {
          processes.push(entry.th32ProcessID);
        }
      }
    } else {
      return Err(get_last_error());
    }
  }

  Ok(processes)
}
