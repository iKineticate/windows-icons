use std::{ffi::OsString, os::windows::ffi::OsStringExt};

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        ProcessStatus::K32GetModuleFileNameExW,
        Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
};

pub fn get_process_path(process_id: u32) -> Result<String, windows::core::Error> {
    unsafe {
        let process_handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            process_id,
        )?;
        let mut buffer = vec![0u16; 1024];
        let size = K32GetModuleFileNameExW(Some(HANDLE(process_handle.0)), None, &mut buffer);
        CloseHandle(process_handle).map_err(|_| {
            windows::core::Error::new(
                windows::core::HRESULT(-1),
                "failed to close process handle.".to_string(),
            )
        })?;

        if size == 0 {
            return Err(windows::core::Error::from_win32());
        }

        buffer.truncate(size as usize);
        let path = OsString::from_wide(&buffer).into_string().map_err(|_| {
            windows::core::Error::new(
                windows::core::HRESULT(-1),
                "invalid Unicode in path.".to_string(),
            )
        })?;
        Ok(path)
    }
}
