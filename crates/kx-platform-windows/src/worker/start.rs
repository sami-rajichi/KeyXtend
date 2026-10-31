//! One start of the worker: its job, its pipes, the reduced token, the list of the only handles it inherits, the
//! process made suspended, put in the job and its token checked, and only then let run.

use std::io::{PipeReader, PipeWriter};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::path::Path;

use kx_platform::pictures::{Launch, SpawnError};
use windows::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, HANDLE, HANDLE_FLAG_INHERIT, SetHandleInformation,
    WIN32_ERROR,
};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    CREATE_SUSPENDED, CreateProcessAsUserW, DETACHED_PROCESS, DeleteProcThreadAttributeList,
    EXTENDED_STARTUPINFO_PRESENT, InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROCESS_INFORMATION, ResumeThread, STARTF_USESTDHANDLES,
    STARTUPINFOEXW, STARTUPINFOW, TerminateProcess, UpdateProcThreadAttribute,
};
use windows::core::{PCWSTR, PWSTR};

use super::job::{ENDED, Job};
use super::{WinChild, access, line, raw, refused, size32, token};

/// What `ResumeThread` returns when it fails.
const RESUME_FAILED: u32 = u32::MAX;

/// The new process and its first thread, still suspended.
struct Made {
    process: OwnedHandle,
    thread: OwnedHandle,
}

/// Starts `exe` as `launch` says and hands back its stdin, its stdout and its process.
pub(super) fn start(
    exe: &Path,
    launch: &Launch,
) -> Result<(PipeWriter, PipeReader, WinChild), SpawnError> {
    let job = Job::new(launch.memory_mb)?;
    let (theirs_in, ours_in) = pipe()?;
    let (ours_out, theirs_out) = pipe()?;
    let made = {
        let token = token::reduced()?;
        let std = (inherit(&theirs_in)?, inherit(&theirs_out)?);
        let mut list = Inherit::new(vec![std.0, std.1])?;
        create(exe, launch, &token, &mut list, std)?
    };
    // Our copies of the worker's ends close here, so its exit ends our reads.
    drop((theirs_in, theirs_out));
    let child = WinChild {
        process: made.process,
        job,
    };
    if let Err(e) = child.job.assign(raw(&child.process)) {
        terminate(&child.process);
        return Err(e);
    }
    // A worker in its job ends with `child` from here on.
    if !access::rights(raw(&child.process))?.reduced() {
        return Err(SpawnError::Rights);
    }
    resume(&made.thread)?;
    Ok((PipeWriter::from(ours_in), PipeReader::from(ours_out), child))
}

/// An anonymous pipe's read and write ends, neither inherited yet.
#[allow(unsafe_code, reason = "Makes a pipe into handles we own.")]
fn pipe() -> Result<(OwnedHandle, OwnedHandle), SpawnError> {
    let (mut read, mut write) = (HANDLE::default(), HANDLE::default());
    // SAFETY: two locals for Windows to fill; with no attributes, neither end is inherited.
    unsafe { CreatePipe(&raw mut read, &raw mut write, None, 0) }
        .map_err(refused("make a pipe"))?;
    // SAFETY: Windows just made both handles, and only their owners close them.
    Ok(unsafe {
        (
            OwnedHandle::from_raw_handle(read.0),
            OwnedHandle::from_raw_handle(write.0),
        )
    })
}

/// Marks `end` inherited, which the handle list needs, and hands back its raw handle.
#[allow(unsafe_code, reason = "Changes a flag of a handle we own.")]
fn inherit(end: &OwnedHandle) -> Result<HANDLE, SpawnError> {
    // SAFETY: our live handle; only its inherit flag changes.
    unsafe { SetHandleInformation(raw(end), HANDLE_FLAG_INHERIT.0, HANDLE_FLAG_INHERIT) }
        .map_err(refused("let the worker inherit its pipe"))?;
    Ok(raw(end))
}

/// Makes the worker's process suspended, with stdin and stdout from `std`, no stderr and no console, whose host would
/// load `user32` under the worker's token and fail where that token reaches no desktop.
#[allow(unsafe_code, reason = "Starts a process with our reduced token.")]
fn create(
    exe: &Path,
    launch: &Launch,
    token: &OwnedHandle,
    list: &mut Inherit,
    std: (HANDLE, HANDLE),
) -> Result<Made, SpawnError> {
    let mut info = STARTUPINFOEXW::default();
    info.StartupInfo.cb = size32::<STARTUPINFOEXW>();
    info.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    // No stderr: the worker never fills a pipe nobody reads, and Rust drops writes to a missing handle.
    (info.StartupInfo.hStdInput, info.StartupInfo.hStdOutput) = std;
    info.lpAttributeList = list.at();
    let app: Vec<u16> = exe.as_os_str().encode_wide().chain([0]).collect();
    let mut line = line::command_line(exe.as_os_str(), &launch.args);
    let flags = EXTENDED_STARTUPINFO_PRESENT | DETACHED_PROCESS | CREATE_SUSPENDED;
    let mut made = PROCESS_INFORMATION::default();
    let startup = (&raw const info).cast::<STARTUPINFOW>();
    // SAFETY: every pointer is to a live local or to `list`, which outlive the call; `app` and `line` end with a null
    // and `line` is writable, as the call wants; the token is open with the rights a start needs.
    unsafe {
        CreateProcessAsUserW(
            Some(raw(token)),
            PCWSTR(app.as_ptr()),
            Some(PWSTR(line.as_mut_ptr())),
            None,
            None,
            true,
            flags,
            None,
            PCWSTR::null(),
            startup,
            &raw mut made,
        )
    }
    .map_err(not_found)?;
    // SAFETY: Windows just made both handles for us alone.
    Ok(unsafe {
        Made {
            process: OwnedHandle::from_raw_handle(made.hProcess.0),
            thread: OwnedHandle::from_raw_handle(made.hThread.0),
        }
    })
}

/// `Missing` for a program that is not there, else a refused start.
fn not_found(e: windows::core::Error) -> SpawnError {
    let gone = [ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND].map(WIN32_ERROR::to_hresult);
    if gone.contains(&e.code()) {
        SpawnError::Missing
    } else {
        refused("start the worker")(e)
    }
}

/// Lets the worker's first thread run.
#[allow(unsafe_code, reason = "Resumes a thread of ours.")]
fn resume(thread: &OwnedHandle) -> Result<(), SpawnError> {
    // SAFETY: our live thread handle from the start.
    if unsafe { ResumeThread(raw(thread)) } == RESUME_FAILED {
        return Err(refused("let the worker run")(
            windows::core::Error::from_thread(),
        ));
    }
    Ok(())
}

/// Ends a worker outside its job.
#[allow(unsafe_code, reason = "Ends a process of ours.")]
fn terminate(process: &OwnedHandle) {
    // SAFETY: our live process handle; a process that already ended makes the call fail harmlessly.
    let _ = unsafe { TerminateProcess(raw(process), ENDED) };
}

/// A thread attribute list naming the only handles the worker inherits; deleted once when dropped.
struct Inherit {
    /// The list's memory, pointer-aligned.
    room: Vec<usize>,
    /// The handles it names, alive and in place until the list is deleted.
    handles: Vec<HANDLE>,
}

impl Inherit {
    /// A list that names `handles` only.
    #[allow(
        unsafe_code,
        reason = "Fills a thread attribute list in memory we own."
    )]
    fn new(handles: Vec<HANDLE>) -> Result<Self, SpawnError> {
        let mut size = 0;
        // SAFETY: a size query with no list; it fails by design and only writes `size`.
        let _ = unsafe { InitializeProcThreadAttributeList(None, 1, None, &raw mut size) };
        let mut room = vec![0; size.div_ceil(size_of::<usize>())];
        let at = LPPROC_THREAD_ATTRIBUTE_LIST(room.as_mut_ptr().cast());
        // SAFETY: `room` holds at least `size` bytes, and its heap block stays put as it moves into the list below.
        unsafe { InitializeProcThreadAttributeList(Some(at), 1, None, &raw mut size) }
            .map_err(refused("make a handle list"))?;
        let mut list = Self { room, handles };
        let value = list.handles.as_ptr().cast();
        let len = size_of_val(list.handles.as_slice());
        let kind = usize::try_from(PROC_THREAD_ATTRIBUTE_HANDLE_LIST).unwrap_or_default();
        // SAFETY: the handles live in `list` until it deletes the attribute list, and their heap block never moves.
        unsafe { UpdateProcThreadAttribute(list.at(), 0, kind, Some(value), len, None, None) }
            .map_err(refused("name the inherited handles"))?;
        Ok(list)
    }

    /// The list, as the start takes it.
    fn at(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        LPPROC_THREAD_ATTRIBUTE_LIST(self.room.as_mut_ptr().cast())
    }
}

impl Drop for Inherit {
    #[allow(unsafe_code, reason = "Deletes the attribute list we made.")]
    fn drop(&mut self) {
        // SAFETY: the list was made in `room` by `new`, and is deleted once, here.
        unsafe { DeleteProcThreadAttributeList(self.at()) };
    }
}
