use core::ptr::null;

use crate::nds;

pub fn nitrofs_init() {
    unsafe {
        if !nds::nitroFSInit(null()) {
            panic!("Failed to initialize nitro fs")
        }
    }
}

use alloc::ffi::CString;
use core::fmt;
use core::result::Result;
use libnds_sys::eprintln;

use core::ffi::CStr;
use core::ptr;

pub fn ls(path: &CStr) {
    unsafe {
        let dir_ptr: *mut nds::DIR = nds::opendir(path.as_ptr());
        if dir_ptr.is_null() {
            eprintln!("open dir fail");
            return;
        }

        eprintln!("open dir ok");
        loop {
            let entry_ptr: *mut nds::dirent = nds::readdir(dir_ptr);
            if entry_ptr.is_null() {
                break;
            }

            let d_name = (*entry_ptr).d_name.as_ptr();
            let name_cstr = CStr::from_ptr(d_name);

            if let Ok(name) = name_cstr.to_str() {
                // Skip "." and ".."
                if name == "." || name == ".." {
                    continue;
                }

                eprintln!("{}", name);
            }
        }

        nds::closedir(dir_ptr);
    }
}

/// A safe wrapper around C's `FILE*` for reading data.
pub struct File {
    file: *mut nds::FILE,
}

/// An error type for file operations.
#[derive(Debug)]
pub struct FileError {
    msg: &'static str,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileError: {}", self.msg)
    }
}

impl core::error::Error for FileError {}
pub type FSResult<T> = Result<T, FileError>;
pub fn read(path: &str) -> FSResult<alloc::boxed::Box<[u8]>> {
    let mut f = File::open(path, "rb")?;
    let len = f.stat()?.st_size as usize;
    let mut buf = alloc::vec![0; len];
    f.read_exact(&mut buf)?;
    Ok(buf.into_boxed_slice())
}

impl File {
    /// Opens a file at the given `path` with the specified `mode`.
    ///
    /// For example, you might use `"rb"` to open a file for reading in binary mode.
    ///
    /// # Errors
    ///
    /// Returns a `FileError` if the path or mode strings are invalid or if `fopen` fails.
    pub fn open(path: &str, mode: &str) -> FSResult<File> {
        // Convert Rust strings into CStrings. These will be used to call `fopen`.
        let c_path = CString::new(path).map_err(|_| FileError {
            msg: "Invalid path string",
        })?;
        let c_mode = CString::new(mode).map_err(|_| FileError {
            msg: "Invalid mode string",
        })?;

        unsafe {
            let file = nds::fopen(c_path.as_ptr(), c_mode.as_ptr());
            if file.is_null() {
                return Err(FileError {
                    msg: "fopen failed",
                });
            }
            Ok(File { file })
        }
    }

    pub fn stat(&self) -> FSResult<nds::stat> {
        let mut st = core::mem::MaybeUninit::<nds::stat>::uninit();
        let res = unsafe { nds::fstat(nds::fileno(self.file), st.as_mut_ptr()) };
        if res == 0 {
            let st = unsafe { st.assume_init() };
            Ok(st)
        } else {
            Err(FileError { msg: "stat failed" })
        }
    }

    /// Reads up to `buf.len()` bytes from the file into `buf`.
    ///
    /// Returns the number of bytes read. If zero bytes are read and end-of-file is reached,
    /// it returns `Ok(0)`. On error, a `FileError` is returned.
    pub fn read(&mut self, buf: &mut [u8]) -> u32 {
        if buf.is_empty() {
            return 0;
        }
        unsafe { nds::fread(buf.as_mut_ptr() as *mut _, 1, buf.len() as _, self.file) }
    }

    /// Reads exactly `buf.len()` bytes into `buf`.
    ///
    /// Returns an error if end-of-file is encountered before reading enough bytes.
    pub fn read_exact(&mut self, mut buf: &mut [u8]) -> FSResult<()> {
        while !buf.is_empty() {
            let n = self.read(buf);
            if n == 0 {
                return Err(FileError {
                    msg: "EOF reached while reading",
                });
            }
            buf = &mut buf[(n as usize)..];
        }
        Ok(())
    }

    /// Reads a single byte from the file.
    ///
    /// Returns `Ok(Some(byte))` if a byte is read or `Ok(None)` if end-of-file is reached.
    pub fn read_byte(&mut self) -> FSResult<Option<u8>> {
        unsafe {
            let c = nds::fgetc(self.file);
            if c == nds::EOF {
                return Ok(None);
            }
            return Ok(Some(c as _));
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            if !self.file.is_null() {
                nds::fclose(self.file);
            }
        }
    }
}
