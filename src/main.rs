mod win32;

use std::{error::Error, path::Path, time::Duration};

use windows::Win32::{Foundation::WAIT_OBJECT_0, System::Threading::LPTHREAD_START_ROUTINE};

use crate::win32::kernel32::{close_handle, wait_for_object};

const DEFAULT_VDS_PATH: &str = "C:\\Program Files\\Virtual Desktop Streamer";
const MODULE_NAME_X86: &str = "VirtualDesktop.Injector32.dll";
const MODULE_NAME_X64: &str = "VirtualDesktop.Injector64.dll";

fn main() -> Result<(), Box<dyn Error>> {
  // Parse args
  let args = std::env::args().collect::<Vec<_>>();
  if args.len() < 2 {
    println!(
      "Usage: {} <process>",
      Path::new(std::env::current_exe()?.as_path().to_str().unwrap())
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
    );
    return Ok(());
  }

  // Get Virtual Desktop installation path
  let vds_path = if let Ok(path) = std::env::var("VDS_PATH") {
    path
  } else {
    DEFAULT_VDS_PATH.to_string()
  };

  let process_name = args[1].to_string();

  println!("Waiting for process '{}' to start...", process_name);

  // Wait for process to start
  let process_id = loop {
    let process_ids = win32::process::get_process_ids_by_name(&process_name)?;

    if process_ids.len() > 0 {
      break process_ids[0];
    }

    std::thread::sleep(Duration::from_millis(25));
  };

  println!("Found process '{}' with id {}", process_name, process_id);

  // Inject DLL

  let process = win32::process::Process::open(process_id)?;
  process.suspend();

  let dll_path = Path::new(&vds_path)
    .join(if process.is_wow64()? {
      MODULE_NAME_X86
    } else {
      MODULE_NAME_X64
    })
    .to_str()
    .unwrap()
    .to_string();

  println!(
    "Injecting Virtual Desktop helper into process '{}'",
    process_name
  );

  let loadlib: LPTHREAD_START_ROUTINE = unsafe {
    std::mem::transmute(win32::kernel32::get_proc_address(
      "kernel32.dll",
      "LoadLibraryA",
    )?)
  };
  let remote_dll_name = process.mem_allocate(dll_path.len() * 2);

  process.mem_write(
    remote_dll_name,
    dll_path.as_ptr() as *mut _,
    dll_path.len() * 2,
  )?;

  let thread = process.thread_create(loadlib, Some(remote_dll_name))?;

  loop {
    if wait_for_object(thread, 1000) == WAIT_OBJECT_0 {
      break;
    }

    // Give the process some time to load the DLL
    process.resume();
    std::thread::sleep(Duration::from_millis(250));
    process.suspend();
  }

  close_handle(thread);

  process.mem_free(remote_dll_name)?;
  process.resume();

  // Done

  println!(
    "Virtual Desktop helper injected into process '{}'\nIf your game is set up to use the Oculus SDK, it should not run without SteamVR.",
    process_name
  );

  Ok(())
}
