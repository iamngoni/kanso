//! Windows: starts the worker in an AppContainer with no capabilities.
//!
//! An AppContainer process runs with a token that has no access to the
//! user's files or the network unless granted. The app creates (or finds)
//! a container profile, starts the worker with that container's SID in the
//! process's security capabilities, and gives it only its two pipes.
//! `std::process::Command` cannot do this, so the process is made with
//! `CreateProcessW` directly. The worker's folder must be readable by "all
//! application packages", as Program Files is.

use std::fs::File;
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::path::Path;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, HANDLE, HANDLE_FLAG_INHERIT, LocalFree, SetHandleInformation,
};
use windows_sys::Win32::Security::Isolation::{
    CreateAppContainerProfile, DeriveAppContainerSidFromAppContainerName,
};
use windows_sys::Win32::Security::{FreeSid, SECURITY_ATTRIBUTES, SECURITY_CAPABILITIES};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    OPEN_EXISTING,
};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CREATE_NO_WINDOW, CreateProcessW, DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT,
    InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES,
    PROCESS_INFORMATION, STARTF_USESTDHANDLES, STARTUPINFOEXW, TerminateProcess,
    UpdateProcThreadAttribute, WaitForSingleObject,
};

const CONTAINER: &str = "app.kanso.pdfworker";

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn last_error(what: &str) -> String {
    format!("{what}: {}", std::io::Error::last_os_error())
}

/// The started worker.
pub struct Launched {
    process: HANDLE,
}

// SAFETY: a process handle may be used from any thread.
unsafe impl Send for Launched {}

impl Launched {
    pub fn kill(&mut self) {
        // SAFETY: `process` is a live handle owned by `self`.
        unsafe { TerminateProcess(self.process, 1) };
    }

    pub fn wait(&mut self) {
        // SAFETY: as above.
        unsafe { WaitForSingleObject(self.process, 5_000) };
    }
}

impl Drop for Launched {
    fn drop(&mut self) {
        // SAFETY: closed once.
        unsafe { CloseHandle(self.process) };
    }
}

/// The container's SID, creating the profile on first use.
unsafe fn container_sid() -> Result<*mut core::ffi::c_void, String> {
    let name = wide(CONTAINER);
    let display = wide("Kanso PDF reader");
    let mut sid = null_mut();
    let made = unsafe {
        CreateAppContainerProfile(
            name.as_ptr(),
            display.as_ptr(),
            display.as_ptr(),
            null(),
            0,
            &mut sid,
        )
    };
    if made >= 0 {
        return Ok(sid);
    }
    // Already there from an earlier run (HRESULT of ERROR_ALREADY_EXISTS).
    if made as u32 == 0x8007_0000 | ERROR_ALREADY_EXISTS {
        let found = unsafe { DeriveAppContainerSidFromAppContainerName(name.as_ptr(), &mut sid) };
        if found >= 0 {
            return Ok(sid);
        }
    }
    Err(format!(
        "couldn't create the PDF reader's container ({made:#x})"
    ))
}

/// A pipe whose `parent` end is not inherited; returns (read, write).
unsafe fn pipe(parent_reads: bool) -> Result<(HANDLE, HANDLE), String> {
    let attrs = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: 1,
    };
    let (mut read, mut write) = (null_mut(), null_mut());
    if unsafe { CreatePipe(&mut read, &mut write, &attrs, 0) } == 0 {
        return Err(last_error("couldn't make a pipe"));
    }
    let parent_end = if parent_reads { read } else { write };
    unsafe { SetHandleInformation(parent_end, HANDLE_FLAG_INHERIT, 0) };
    Ok((read, write))
}

pub fn launch(exe: &Path, library: Option<&Path>) -> Result<(Launched, File, File), String> {
    // SAFETY: Win32 calls with valid, live arguments; every handle made here
    // is either handed to a `File`/`Launched` or closed before returning.
    unsafe {
        let sid = container_sid()?;
        let (child_in, parent_out) = pipe(false)?;
        let (parent_in, child_out) = pipe(true)?;
        let nul_name = wide("NUL");
        let attrs = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: 1,
        };
        let nul = CreateFileW(
            nul_name.as_ptr(),
            FILE_GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            &attrs,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        );

        let mut capabilities: SECURITY_CAPABILITIES = std::mem::zeroed();
        capabilities.AppContainerSid = sid;
        let mut inherited = [child_in, child_out, nul];

        let mut size = 0usize;
        InitializeProcThreadAttributeList(null_mut(), 2, 0, &mut size);
        let mut storage = vec![0u8; size];
        let list: LPPROC_THREAD_ATTRIBUTE_LIST = storage.as_mut_ptr().cast();
        if InitializeProcThreadAttributeList(list, 2, 0, &mut size) == 0 {
            return Err(last_error("couldn't prepare the PDF reader"));
        }
        let set_container = UpdateProcThreadAttribute(
            list,
            0,
            PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
            (&mut capabilities as *mut SECURITY_CAPABILITIES).cast(),
            size_of::<SECURITY_CAPABILITIES>(),
            null_mut(),
            null(),
        );
        let set_handles = UpdateProcThreadAttribute(
            list,
            0,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
            inherited.as_mut_ptr().cast(),
            size_of::<HANDLE>() * inherited.len(),
            null_mut(),
            null(),
        );

        let mut startup: STARTUPINFOEXW = std::mem::zeroed();
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup.StartupInfo.hStdInput = child_in;
        startup.StartupInfo.hStdOutput = child_out;
        startup.StartupInfo.hStdError = nul;
        startup.lpAttributeList = list;

        let mut command = format!("\"{}\"", exe.display());
        if let Some(library) = library {
            command.push_str(&format!(" --library \"{}\"", library.display()));
        }
        let mut command = wide(&command);
        let exe_wide = wide(&exe.to_string_lossy());
        // An empty environment: just the terminating double NUL.
        let environment: [u16; 2] = [0, 0];
        let dir = wide(
            &exe.parent()
                .map_or_else(String::new, |p| p.to_string_lossy().into_owned()),
        );
        let mut info: PROCESS_INFORMATION = std::mem::zeroed();
        let created = if set_container != 0 && set_handles != 0 {
            CreateProcessW(
                exe_wide.as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                1,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW | 0x0000_0400, // CREATE_UNICODE_ENVIRONMENT
                environment.as_ptr().cast(),
                dir.as_ptr(),
                &startup.StartupInfo,
                &mut info,
            )
        } else {
            0
        };
        let failure = (created == 0).then(|| last_error("couldn't start the PDF reader"));

        DeleteProcThreadAttributeList(list);
        FreeSid(sid);
        // The child's ends are the child's now.
        CloseHandle(child_in);
        CloseHandle(child_out);
        if !nul.is_null() {
            CloseHandle(nul);
        }
        if let Some(failure) = failure {
            CloseHandle(parent_in);
            CloseHandle(parent_out);
            let _ = LocalFree;
            return Err(failure);
        }
        CloseHandle(info.hThread);
        Ok((
            Launched {
                process: info.hProcess,
            },
            File::from_raw_handle(parent_out as RawHandle),
            File::from_raw_handle(parent_in as RawHandle),
        ))
    }
}
