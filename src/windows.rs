use compio_log::error;

type DWORD = u32;

/// Win32 BOOL type (0 = false, non-zero = true).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct BOOL(i32);

pub type WinResult<T> = Result<T, std::io::Error>;

impl BOOL {
    /// Convert to Result. Returns error if value is 0 (false).
    fn or_error(self, case: &str) -> WinResult<()> {
        if self.0 == 0 {
            let error = std::io::Error::last_os_error();
            error!("Failed to {case}: {error}");
            Err(error)
        } else {
            Ok(())
        }
    }
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn AttachConsole(dwProcessId: DWORD) -> BOOL;
}

/// Try to attach to the parent process's console.
///
/// On Windows GUI apps, there is no console by default.
/// Calling this lets us see log output when launched from a terminal.
pub fn try_attach_console() -> WinResult<()> {
    unsafe { AttachConsole(-1_i32 as u32) }.or_error("attach console")
}
