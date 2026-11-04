//! Windows: an AppContainer, inside a Job Object.
//!
//! The AppContainer is applied by the app when it starts the worker (see
//! [`crate::client`]): a token with no capabilities cannot reach the user's
//! files or the network. The worker puts itself in a Job Object that caps its
//! memory and forbids child processes, then checks that its token really is
//! an AppContainer's. A worker started any other way finds that it is not and
//! refuses to run.

use std::mem::{size_of, zeroed};

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TokenIsAppContainer};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use super::{Allow, ContainError, MEMORY_BUDGET};

fn last_error(what: &str) -> ContainError {
    ContainError(format!("{what}: {}", std::io::Error::last_os_error()))
}

pub(super) fn apply(_allow: &Allow) -> Result<(), ContainError> {
    // SAFETY: plain Win32 calls with valid arguments; the job handle is kept
    // open on purpose, so the limits last as long as the process.
    unsafe {
        let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job.is_null() {
            return Err(last_error("couldn't create the job object"));
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_PROCESS_MEMORY
            | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
            | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // Only the worker itself: it may not start another program.
        info.BasicLimitInformation.ActiveProcessLimit = 1;
        info.ProcessMemoryLimit = MEMORY_BUDGET as usize;
        let set = SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );
        if set == 0 {
            return Err(last_error("couldn't set the job object's limits"));
        }
        if AssignProcessToJobObject(job, GetCurrentProcess()) == 0 {
            return Err(last_error("couldn't join the job object"));
        }
    }
    Ok(())
}

pub(super) fn verify() -> Result<(), ContainError> {
    // SAFETY: the token handle is opened, read and closed here.
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(last_error("couldn't read the process token"));
        }
        let mut is_container: u32 = 0;
        let mut returned = 0u32;
        let read = GetTokenInformation(
            token,
            TokenIsAppContainer,
            (&mut is_container as *mut u32).cast(),
            size_of::<u32>() as u32,
            &mut returned,
        );
        CloseHandle(token);
        if read == 0 {
            return Err(last_error("couldn't read the process token"));
        }
        if is_container == 0 {
            return Err(ContainError(
                "the PDF reader is not running in an AppContainer".into(),
            ));
        }
    }
    Ok(())
}
