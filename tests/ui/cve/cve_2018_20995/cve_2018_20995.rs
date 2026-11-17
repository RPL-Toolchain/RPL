//@revisions: inline normal
//@check-pass: no pattern yet
//@ignore-target: windows
// Copied and modified from https://github.com/servo/rust-slice-deque/blob/621274a

#![allow(deprecated)]
#![allow(invalid_value)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(rpl::generic_function_marked_inline)]
#![allow(rpl::private_function_marked_inline)]
#![allow(rpl::ptr_offset_with_cast)]
#![allow(rpl::swap_ptr_to_ref)]
#![allow(rpl::unchecked_pointer_offset)]
#![allow(rpl::unchecked_pointer_offset_general)]

use buffer::Buffer;
use core::ptr::NonNull;
use core::{cmp, convert, fmt, hash, iter, mem, ops, ptr, slice, str};
use mirrored::*;

#[macro_use]
mod macros {
    //! Macros and utilities.

    /// Small Ascii String. Used to write errors in `no_std` environments.
    pub struct TinyAsciiString {
        /// A buffer for the ascii string
        buf: [u8; 512],
    }

    impl TinyAsciiString {
        /// Creates a new string initialized to zero.
        pub fn new() -> Self {
            Self { buf: [0_u8; 512] }
        }
        /// Converts the Tiny Ascii String to an UTF-8 string (unchecked).
        pub unsafe fn as_str(&self) -> &str {
            unsafe { std::str::from_utf8_unchecked(&self.buf) }
        }
    }

    impl std::fmt::Write for TinyAsciiString {
        fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
            for (idx, b) in s.bytes().enumerate() {
                if let Some(mut v) = self.buf.get_mut(idx) {
                    *v = b;
                } else {
                    return Err(std::fmt::Error);
                }
            }
            Ok(())
        }
    }

    macro_rules! tiny_str {
        ($($t:tt)*) => (
            {
                use std::fmt::Write;
                let mut s: $crate::macros::TinyAsciiString = $crate::macros::TinyAsciiString::new();
                write!(&mut s, $($t)*).unwrap();
                s
            }
        )
    }
}

/// Returns the size of a memory allocation unit.
///
/// In Linux-like systems this equals the page-size.
#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    not(feature = "unix_sysv")
))]
mod mirrored {
    use super::*;

    use libc::{
        _SC_PAGESIZE, ENOSYS, MAP_FAILED, MAP_FIXED, MAP_SHARED, PROT_READ, PROT_WRITE,
        SYS_memfd_create, c_char, c_int, c_long, c_uint, c_void, close, ftruncate, mkstemp, mmap,
        munmap, off_t, size_t, syscall, sysconf,
    };

    #[cfg(target_os = "android")]
    use libc::__errno;
    #[cfg(not(target_os = "android"))]
    use libc::__errno_location;

    use super::{AllocError, ptr};

    /// [`memfd_create`] - create an anonymous file
    ///
    /// [`memfd_create`]: http://man7.org/linux/man-pages/man2/memfd_create.2.html
    fn memfd_create(name: *const c_char, flags: c_uint) -> c_long {
        unsafe { syscall(SYS_memfd_create, name, flags) }
    }

    /// Returns the size of a memory allocation unit.
    ///
    /// In Linux-like systems this equals the page-size.
    pub fn allocation_granularity() -> usize {
        unsafe { sysconf(_SC_PAGESIZE) as usize }
    }

    /// Reads `errno`.
    fn errno() -> c_int {
        #[cfg(not(target_os = "android"))]
        unsafe {
            *__errno_location()
        }
        #[cfg(target_os = "android")]
        unsafe {
            *__errno()
        }
    }

    /// Allocates an uninitialzied buffer that holds `size` bytes, where
    /// the bytes in range `[0, size / 2)` are mirrored into the bytes in
    /// range `[size / 2, size)`.
    ///
    /// On Linux the algorithm is as follows:
    ///
    /// * 1. Allocate a memory-mapped file containing `size / 2` bytes.
    /// * 2. Map the file into `size` bytes of virtual memory.
    /// * 3. Map the file into the last `size / 2` bytes of the virtual memory
    /// region      obtained in step 2.
    ///
    /// This algorithm doesn't have any races.
    ///
    /// # Panics
    ///
    /// If `size` is zero or `size / 2` is not a multiple of the
    /// allocation granularity.
    pub fn allocate_mirrored(size: usize) -> Result<*mut u8, AllocError> {
        unsafe {
            let half_size = size / 2;
            assert!(size != 0);
            assert!(half_size % allocation_granularity() == 0);

            // create temporary file
            let mut fname = *b"/tmp/slice_deque_fileXXXXXX\0";
            let mut fd: c_long = memfd_create(fname.as_mut_ptr() as *mut c_char, 0);
            if fd == -1 && errno() == ENOSYS {
                // memfd_create is not implemented, use mkstemp instead:
                fd = c_long::from(mkstemp(fname.as_mut_ptr() as *mut c_char));
            }
            if fd == -1 {
                print_error("memfd_create failed");
                return Err(AllocError::Other);
            }
            let fd = fd as c_int;
            if ftruncate(fd, half_size as off_t) == -1 {
                print_error("ftruncate failed");
                if close(fd) == -1 {
                    print_error("@ftruncate: close failed");
                }
                return Err(AllocError::Oom);
            };

            // mmap memory
            let ptr = mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd,
                0,
            );
            if ptr == MAP_FAILED {
                print_error("@first: mmap failed");
                if close(fd) == -1 {
                    print_error("@first: close failed");
                }
                return Err(AllocError::Oom);
            }

            let ptr2 = mmap(
                (ptr as *mut u8).offset(half_size as isize) as *mut c_void,
                half_size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED | MAP_FIXED,
                fd,
                0,
            );
            if ptr2 == MAP_FAILED {
                print_error("@second: mmap failed");
                if munmap(ptr, size as size_t) == -1 {
                    print_error("@second: munmap failed");
                }
                if close(fd) == -1 {
                    print_error("@second: close failed");
                }
                return Err(AllocError::Other);
            }

            if close(fd) == -1 {
                print_error("@success: close failed");
            }
            Ok(ptr as *mut u8)
        }
    }

    /// Deallocates the mirrored memory region at `ptr` of `size` bytes.
    ///
    /// # Unsafe
    ///
    /// `ptr` must have been obtained from a call to `allocate_mirrored(size)`,
    /// otherwise the behavior is undefined.
    ///
    /// # Panics
    ///
    /// If `size` is zero or `size / 2` is not a multiple of the
    /// allocation granularity, or `ptr` is null.
    pub unsafe fn deallocate_mirrored(ptr: *mut u8, size: usize) {
        assert!(!ptr.is_null());
        assert!(size != 0);
        assert!(size % allocation_granularity() == 0);
        if munmap(ptr as *mut c_void, size as size_t) == -1 {
            print_error("deallocate munmap failed");
        }
    }

    /// Prints last os error at `location`.
    #[cfg(all(debug_assertions, feature = "use_std"))]
    fn print_error(location: &str) {
        eprintln!(
            "Error at {}: {}",
            location,
            ::std::io::Error::last_os_error()
        );
    }

    /// Prints last os error at `location`.
    #[cfg(not(all(debug_assertions, feature = "use_std")))]
    fn print_error(_location: &str) {}
}

#[cfg(all(
    any(target_os = "macos", target_os = "ios"),
    not(feature = "unix_sysv")
))]
mod mirrored {
    use super::*;
    use mach;
    use mach::boolean::boolean_t;
    use mach::kern_return::*;
    use mach::mach_types::mem_entry_name_port_t;
    use mach::memory_object_types::{memory_object_offset_t, memory_object_size_t};
    use mach::traps::mach_task_self;
    use mach::vm::{
        mach_make_memory_entry_64, mach_vm_allocate, mach_vm_deallocate, mach_vm_remap,
    };
    use mach::vm_inherit::VM_INHERIT_NONE;
    use mach::vm_prot::{VM_PROT_READ, VM_PROT_WRITE, vm_prot_t};
    use mach::vm_statistics::{VM_FLAGS_ANYWHERE, VM_FLAGS_FIXED};
    use mach::vm_types::mach_vm_address_t;

    /// TODO: not exposed by the mach crate
    const VM_FLAGS_OVERWRITE: ::libc::c_int = 0x4000_i32;

    /// Returns the size of an allocation unit.
    ///
    /// In `MacOSX` this equals the page size.
    pub fn allocation_granularity() -> usize {
        unsafe { mach::vm_page_size::vm_page_size as usize }
    }

    /// Allocates an uninitialzied buffer that holds `size` bytes, where
    /// the bytes in range `[0, size / 2)` are mirrored into the bytes in
    /// range `[size / 2, size)`.
    ///
    /// On Macos X the algorithm is as follows:
    ///
    /// * 1. Allocate twice the memory (`size` bytes)
    /// * 2. Deallocate the second half (bytes in range `[size / 2, 0)`)
    /// * 3. Race condition: mirror bytes of the first half into the second
    /// half.
    ///
    /// If we get a race (e.g. because some other process allocates to the
    /// second half) we release all the resources (we need to deallocate the
    /// memory) and try again (up to a maximum of `MAX_NO_ALLOC_ITERS` times).
    ///
    /// # Panics
    ///
    /// If `size` is zero or `size / 2` is not a multiple of the
    /// allocation granularity.
    pub fn allocate_mirrored(size: usize) -> Result<*mut u8, AllocError> {
        unsafe {
            assert!(size != 0);
            let half_size = size / 2;
            assert!(half_size % allocation_granularity() == 0);

            let task = mach_task_self();

            // Allocate memory to hold the whole buffer:
            let mut addr: mach_vm_address_t = 0;
            let r: kern_return_t = mach_vm_allocate(
                task,
                &mut addr as *mut mach_vm_address_t,
                size as u64,
                VM_FLAGS_ANYWHERE,
            );
            if r != KERN_SUCCESS {
                // If the first allocation fails, there is nothing to
                // deallocate and we can just fail to allocate:
                print_error("initial alloc", r);
                return Err(AllocError::Oom);
            }
            debug_assert!(addr != 0);

            // Set the size of the first half to size/2:
            let r: kern_return_t = mach_vm_allocate(
                task,
                &mut addr as *mut mach_vm_address_t,
                half_size as u64,
                VM_FLAGS_FIXED | VM_FLAGS_OVERWRITE,
            );
            if r != KERN_SUCCESS {
                // If the first allocation fails, there is nothing to
                // deallocate and we can just fail to allocate:
                print_error("first half alloc", r);
                return Err(AllocError::Other);
            }

            // Get an object handle to the first memory region:
            let mut memory_object_size = half_size as memory_object_size_t;
            let mut object_handle: mem_entry_name_port_t = mem::uninitialized();
            let parent_handle: mem_entry_name_port_t = 0;
            let r: kern_return_t = mach_make_memory_entry_64(
                task,
                &mut memory_object_size as *mut memory_object_size_t,
                addr as memory_object_offset_t,
                VM_PROT_READ | VM_PROT_WRITE,
                &mut object_handle as *mut mem_entry_name_port_t,
                parent_handle,
            );

            if r != KERN_SUCCESS {
                // If making the memory entry fails we should deallocate the first
                // allocation:
                print_error("make memory entry", r);
                if dealloc(addr as *mut u8, size).is_err() {
                    panic!("failed to deallocate after error");
                }
                return Err(AllocError::Other);
            }

            // Map the first half to the second half using the object handle:
            let mut to = (addr as *mut u8).add(half_size) as mach_vm_address_t;
            let mut current_prot: vm_prot_t = mem::uninitialized();
            let mut out_prot: vm_prot_t = mem::uninitialized();
            let r: kern_return_t = mach_vm_remap(
                task,
                &mut to as *mut mach_vm_address_t,
                half_size as u64,
                /* mask: */ 0,
                VM_FLAGS_FIXED | VM_FLAGS_OVERWRITE,
                task,
                addr,
                /* copy: */ 0 as boolean_t,
                &mut current_prot as *mut vm_prot_t,
                &mut out_prot as *mut vm_prot_t,
                VM_INHERIT_NONE,
            );

            if r != KERN_SUCCESS {
                print_error("map first to second half", r);
                // If making the memory entry fails we deallocate all the memory
                if dealloc(addr as *mut u8, size).is_err() {
                    panic!("failed to deallocate after error");
                }
                return Err(AllocError::Other);
            }

            // TODO: object_handle is leaked here. Investigate whether this is ok.

            Ok(addr as *mut u8)
        }
    }

    /// Deallocates the mirrored memory region at `ptr` of `size` bytes.
    ///
    /// # Unsafe
    ///
    /// `ptr` must have been obtained from a call to `allocate_mirrored(size)`,
    /// otherwise the behavior is undefined.
    ///
    /// # Panics
    ///
    /// If `size` is zero or `size / 2` is not a multiple of the
    /// allocation granularity, or `ptr` is null.
    pub unsafe fn deallocate_mirrored(ptr: *mut u8, size: usize) {
        assert!(!ptr.is_null());
        assert!(size != 0);
        assert!(size % allocation_granularity() == 0);
        unsafe { dealloc(ptr, size).expect("deallocating mirrored buffer failed") };
    }

    /// Tries to deallocates `size` bytes of memory starting at `ptr`.
    ///
    /// # Unsafety
    ///
    /// The `ptr` must have been obtained from a previous call to `alloc` and point
    /// to a memory region containing at least `size` bytes.
    ///
    /// # Panics
    ///
    /// If `size` is zero or not a multiple of the `allocation_granularity`, or if
    /// `ptr` is null.
    unsafe fn dealloc(ptr: *mut u8, size: usize) -> Result<(), ()> {
        assert!(size != 0);
        assert!(size % allocation_granularity() == 0);
        assert!(!ptr.is_null());
        let addr = ptr as mach_vm_address_t;
        let r: kern_return_t = unsafe { mach_vm_deallocate(mach_task_self(), addr, size as u64) };
        if r != KERN_SUCCESS {
            print_error("dealloc", r);
            return Err(());
        }
        Ok(())
    }

    fn print_error(msg: &str, code: kern_return_t) {
        eprintln!("ERROR at \"{}\": {}", msg, report_error(code));
    }

    fn report_error(error: kern_return_t) -> &'static str {
        use mach::kern_return::*;
        match error {
            KERN_ABORTED => "KERN_ABORTED",
            KERN_ALREADY_IN_SET => "KERN_ALREADY_IN_SET",
            KERN_ALREADY_WAITING => "KERN_ALREADY_WAITING",
            KERN_CODESIGN_ERROR => "KERN_CODESIGN_ERROR",
            KERN_DEFAULT_SET => "KERN_DEFAULT_SET",
            KERN_EXCEPTION_PROTECTED => "KERN_EXCEPTION_PROTECTED",
            KERN_FAILURE => "KERN_FAILURE",
            KERN_INVALID_ADDRESS => "KERN_INVALID_ADDRESS",
            KERN_INVALID_ARGUMENT => "KERN_INVALID_ARGUMENT",
            KERN_INVALID_CAPABILITY => "KERN_INVALID_CAPABILITY",
            KERN_INVALID_HOST => "KERN_INVALID_HOST",
            KERN_INVALID_LEDGER => "KERN_INVALID_LEDGER",
            KERN_INVALID_MEMORY_CONTROL => "KERN_INVALID_MEMORY_CONTROL",
            KERN_INVALID_NAME => "KERN_INVALID_NAME",
            KERN_INVALID_OBJECT => "KERN_INVALID_OBJECT",
            KERN_INVALID_POLICY => "KERN_INVALID_POLICY",
            KERN_INVALID_PROCESSOR_SET => "KERN_INVALID_PROCESSOR_SET",
            KERN_INVALID_RIGHT => "KERN_INVALID_RIGHT",
            KERN_INVALID_SECURITY => "KERN_INVALID_SECURITY",
            KERN_INVALID_TASK => "KERN_INVALID_TASK",
            KERN_INVALID_VALUE => "KERN_INVALID_VALUE",
            KERN_LOCK_OWNED => "KERN_LOCK_OWNED",
            KERN_LOCK_OWNED_SELF => "KERN_LOCK_OWNED_SELF",
            KERN_LOCK_SET_DESTROYED => "KERN_LOCK_SET_DESTROYED",
            KERN_LOCK_UNSTABLE => "KERN_LOCK_UNSTABLE",
            KERN_MEMORY_DATA_MOVED => "KERN_MEMORY_DATA_MOVED",
            KERN_MEMORY_ERROR => "KERN_MEMORY_ERROR",
            KERN_MEMORY_FAILURE => "KERN_MEMORY_FAILURE",
            KERN_MEMORY_PRESENT => "KERN_MEMORY_PRESENT",
            KERN_MEMORY_RESTART_COPY => "KERN_MEMORY_RESTART_COPY",
            KERN_NAME_EXISTS => "KERN_NAME_EXISTS",
            KERN_NODE_DOWN => "KERN_NODE_DOWN",
            KERN_NOT_DEPRESSED => "KERN_NOT_DEPRESSED",
            KERN_NOT_IN_SET => "KERN_NOT_IN_SET",
            KERN_NOT_RECEIVER => "KERN_NOT_RECEIVER",
            KERN_NOT_SUPPORTED => "KERN_NOT_SUPPORTED",
            KERN_NOT_WAITING => "KERN_NOT_WAITING",
            KERN_NO_ACCESS => "KERN_NO_ACCESS",
            KERN_NO_SPACE => "KERN_NO_SPACE",
            KERN_OPERATION_TIMED_OUT => "KERN_OPERATION_TIMED_OUT",
            KERN_POLICY_LIMIT => "KERN_POLICY_LIMIT",
            KERN_POLICY_STATIC => "KERN_POLICY_STATIC",
            KERN_PROTECTION_FAILURE => "KERN_PROTECTION_FAILURE",
            KERN_RESOURCE_SHORTAGE => "KERN_RESOURCE_SHORTAGE",
            KERN_RETURN_MAX => "KERN_RETURN_MAX",
            KERN_RIGHT_EXISTS => "KERN_RIGHT_EXISTS",
            KERN_RPC_CONTINUE_ORPHAN => "KERN_RPC_CONTINUE_ORPHAN",
            KERN_RPC_SERVER_TERMINATED => "KERN_RPC_SERVER_TERMINATED",
            KERN_RPC_TERMINATE_ORPHAN => "KERN_RPC_TERMINATE_ORPHAN",
            KERN_SEMAPHORE_DESTROYED => "KERN_SEMAPHORE_DESTROYED",
            KERN_SUCCESS => "KERN_SUCCESS",
            KERN_TERMINATED => "KERN_TERMINATED",
            KERN_UREFS_OVERFLOW => "KERN_UREFS_OVERFLOW",
            v => {
                eprintln!("unknown kernel error: {}", v);
                "UNKNOWN_KERN_ERROR"
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod mirrored {
    use mem;

    use winapi::shared::basetsd::SIZE_T;
    use winapi::shared::minwindef::{BOOL, DWORD, LPCVOID, LPVOID};
    use winapi::shared::ntdef::LPCWSTR;
    use winapi::um::memoryapi::{
        CreateFileMappingW, FILE_MAP_ALL_ACCESS, MapViewOfFileEx, UnmapViewOfFile, VirtualAlloc,
        VirtualFree,
    };
    use winapi::um::winnt::{MEM_RELEASE, MEM_RESERVE, PAGE_NOACCESS, PAGE_READWRITE, SEC_COMMIT};

    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::minwinbase::LPSECURITY_ATTRIBUTES;
    use winapi::um::sysinfoapi::{GetSystemInfo, LPSYSTEM_INFO, SYSTEM_INFO};

    pub use winapi::shared::ntdef::HANDLE;

    use AllocError;

    /// Returns the size of an allocation unit in bytes.
    ///
    /// In Windows calls to `VirtualAlloc` must specify a multiple of
    /// `SYSTEM_INFO::dwAllocationGranularity` bytes.
    ///
    /// FIXME: the allocation granularity should always be larger than the page
    /// size (64k vs 4k), so determining the page size here is not necessary.
    pub fn allocation_granularity() -> usize {
        unsafe {
            let mut system_info: SYSTEM_INFO = mem::uninitialized();
            GetSystemInfo(&mut system_info as LPSYSTEM_INFO);
            let allocation_granularity = system_info.dwAllocationGranularity as usize;
            let page_size = system_info.dwPageSize as usize;
            page_size.max(allocation_granularity)
        }
    }

    /// Allocates an uninitialzied buffer that holds `size` bytes, where
    /// the bytes in range `[0, size / 2)` are mirrored into the bytes in
    /// range `[size / 2, size)`.
    ///
    /// On Windows the algorithm is as follows:
    ///
    /// * 1. Allocate physical memory to hold `size / 2` bytes using a   memory
    ///   mapped file.
    /// * 2. Find a region of virtual memory large enough to hold `size`
    /// bytes (by allocating memory with `VirtualAlloc` and immediately
    /// freeing   it with `VirtualFree`).
    /// * 3. Race condition: map the physical memory to the two halves of the
    ///   virtual memory region.
    ///
    /// If we get a race (e.g. because some other process obtains memory in the
    /// memory region where we wanted to map our physical memory) we release
    /// the first portion of virtual memory if mapping succeeded and try
    /// again (up to a maximum of `MAX_NO_ALLOC_ITERS` times).
    ///
    /// # Panics
    ///
    /// If `size` is zero or `size / 2` is not a multiple of the
    /// allocation granularity.
    pub fn allocate_mirrored(size: usize) -> Result<*mut u8, AllocError> {
        /// Maximum number of attempts to allocate in case of a race condition.
        const MAX_NO_ALLOC_ITERS: usize = 10;
        unsafe {
            let half_size = size / 2;
            assert!(size != 0);
            assert!(half_size % allocation_granularity() == 0);

            let file_mapping = create_file_mapping(half_size)?;

            let mut no_iters = 0;
            let virt_ptr = loop {
                if no_iters > MAX_NO_ALLOC_ITERS {
                    // If we exceeded the number of iterations try to close the
                    // handle and error:
                    close_file_mapping(file_mapping).expect("freeing physical memory failed");
                    return Err(AllocError::Other);
                }

                // Find large enough virtual memory region (if this fails we are
                // done):
                let virt_ptr = reserve_virtual_memory(size)?;

                // Map the physical memory to the first half:
                if map_view_of_file(file_mapping, half_size, virt_ptr).is_err() {
                    // If this fails, there is nothing to free and we try again:
                    no_iters += 1;
                    continue;
                }

                // Map physical memory to the second half:
                if map_view_of_file(file_mapping, half_size, virt_ptr.offset(half_size as isize))
                    .is_err()
                {
                    // If this fails, we release the map of the first half and try
                    // again:
                    no_iters += 1;
                    if unmap_view_of_file(virt_ptr).is_err() {
                        // If unmapping fails try to close the handle and
                        // panic:
                        close_file_mapping(file_mapping).expect("freeing physical memory failed");
                        panic!("unmapping first half of memory failed")
                    }
                    continue;
                }

                // We are done
                break virt_ptr;
            };

            // Close the file handle, it will be released when all the memory is
            // unmapped:
            close_file_mapping(file_mapping).expect("closing file handle failed");

            Ok(virt_ptr)
        }
    }

    /// Deallocates the mirrored memory region at `ptr` of `size` bytes.
    ///
    /// # Unsafe
    ///
    /// `ptr` must have been obtained from a call to `allocate_mirrored(size)`,
    /// otherwise the behavior is undefined.
    ///
    /// # Panics
    ///
    /// If `size` is zero or `size / 2` is not a multiple of the
    /// allocation granularity, or `ptr` is null.
    pub unsafe fn deallocate_mirrored(ptr: *mut u8, size: usize) {
        assert!(!ptr.is_null());
        assert!(size != 0);
        assert!(size % allocation_granularity() == 0);
        // On "windows" we unmap the memory.
        let half_size = size / 2;
        unmap_view_of_file(ptr).expect("unmapping first buffer half failed");
        let second_half_ptr = ptr.offset(half_size as isize);
        unmap_view_of_file(second_half_ptr).expect("unmapping second buffer half failed");
    }

    /// Creates a file mapping able to hold `size` bytes.
    ///
    /// # Panics
    ///
    /// If `size` is zero or not a multiple of the `allocation_granularity`.
    fn create_file_mapping(size: usize) -> Result<HANDLE, AllocError> {
        unsafe {
            assert!(size != 0);
            assert!(size % allocation_granularity() == 0);
            let dw_maximum_size_low: DWORD = size as DWORD;
            let dw_maximum_size_high: DWORD =
                match (mem::size_of::<DWORD>(), mem::size_of::<usize>()) {
                    // If both sizes are equal, the size is passed in the lower
                    // half, so the higher 32-bits are zero
                    (4, 4) | (8, 8) => 0,
                    // If DWORD is 32 bit but usize is 64-bit, we pass the higher
                    // 32-bit of size:
                    (4, 8) => (size >> 32) as DWORD,
                    _ => unimplemented!(),
                };

            let h: HANDLE = CreateFileMappingW(
                /* hFile: */ INVALID_HANDLE_VALUE as HANDLE,
                /* lpAttributes: */ 0 as LPSECURITY_ATTRIBUTES,
                /* flProtect: */ PAGE_READWRITE | SEC_COMMIT as DWORD,
                /* dwMaximumSizeHigh: */ dw_maximum_size_high,
                /* dwMaximumSizeLow: */ dw_maximum_size_low,
                /* lpName: */ 0 as LPCWSTR,
            );

            if h.is_null() {
                let s = tiny_str!("create_file_mapping (with size: {})", size);
                print_error(s.as_str());
                return Err(AllocError::Oom);
            }
            Ok(h)
        }
    }

    /// Closes a file mapping.
    ///
    /// # Unsafety
    ///
    /// `file_mapping` must point to a valid file mapping created with
    /// `create_file_mapping`.
    ///
    /// # Panics
    ///
    /// If `file_mapping` is null.
    unsafe fn close_file_mapping(file_mapping: HANDLE) -> Result<(), ()> {
        assert!(!file_mapping.is_null());

        let r: BOOL = CloseHandle(file_mapping);
        if r == 0 {
            print_error("close_file_mapping");
            return Err(());
        }
        Ok(())
    }

    /// Reserves a virtual memory region able to hold `size` bytes.
    ///
    /// The Windows API has no way to do this, so... we allocate a `size`-ed region
    /// with `VirtualAlloc`, immediately free it afterwards with `VirtualFree` and
    /// hope that the region is still available when we try to map into it.
    ///
    /// # Panics
    ///
    /// If `size` is not a multiple of the `allocation_granularity`.
    fn reserve_virtual_memory(size: usize) -> Result<(*mut u8), AllocError> {
        unsafe {
            assert!(size != 0);
            assert!(size % allocation_granularity() == 0);

            let r: LPVOID = VirtualAlloc(
                /* lpAddress: */ 0 as LPVOID,
                /* dwSize: */ size as SIZE_T,
                /* flAllocationType: */ MEM_RESERVE,
                /* flProtect: */ PAGE_NOACCESS,
            );

            if r.is_null() {
                print_error("reserve_virtual_memory(alloc failed)");
                return Err(AllocError::Oom);
            }

            let fr = VirtualFree(
                /* lpAddress: */ r,
                /* dwSize: */ 0 as SIZE_T,
                /* dwFreeType: */ MEM_RELEASE as DWORD,
            );
            if fr == 0 {
                print_error("reserve_virtual_memory(free failed)");
                return Err(AllocError::Other);
            }

            Ok(r as *mut u8)
        }
    }

    /// Maps `size` bytes of `file_mapping` to `address`.
    ///
    /// # Unsafety
    ///
    /// `file_mapping` must point to a valid file-mapping created with
    /// `create_file_mapping`.
    ///
    /// # Panics
    ///
    /// If `file_mapping` or `address` are null, or if `size` is zero or not a
    /// multiple of the allocation granularity of the system.
    unsafe fn map_view_of_file(
        file_mapping: HANDLE,
        size: usize,
        address: *mut u8,
    ) -> Result<(), ()> {
        assert!(!file_mapping.is_null());
        assert!(!address.is_null());
        assert!(size != 0);
        assert!(size % allocation_granularity() == 0);

        let r: LPVOID = MapViewOfFileEx(
            /* hFileMappingObject: */ file_mapping,
            /* dwDesiredAccess: */ FILE_MAP_ALL_ACCESS,
            /* dwFileOffsetHigh: */ 0 as DWORD,
            /* dwFileOffsetLow: */ 0 as DWORD,
            /* dwNumberOfBytesToMap: */ size as SIZE_T,
            /* lpBaseAddress: */ address as LPVOID,
        );
        if r.is_null() {
            print_error("map_view_of_file");
            return Err(());
        }
        debug_assert!(r == address as LPVOID);
        Ok(())
    }

    /// Unmaps the memory at `address`.
    ///
    /// # Unsafety
    ///
    /// If address does not point to a valid memory address previously mapped with
    /// `map_view_of_file`.
    ///
    /// # Panics
    ///
    /// If `address` is null.
    unsafe fn unmap_view_of_file(address: *mut u8) -> Result<(), ()> {
        assert!(!address.is_null());

        let r = UnmapViewOfFile(/* lpBaseAddress: */ address as LPCVOID);
        if r == 0 {
            print_error("unmap_view_of_file");
            return Err(());
        }
        Ok(())
    }

    /// Prints last os error at `location`.
    fn print_error(location: &str) {
        eprintln!(
            "Error at {}: {}",
            location,
            ::std::io::Error::last_os_error()
        );
    }
}

mod buffer {
    use super::*;

    /// Number of required memory allocation units to hold `bytes`.
    fn no_required_allocation_units(bytes: usize) -> usize {
        let ag = allocation_granularity();
        let r = ((bytes + ag - 1) / ag).max(1);
        let r = if r % 2 == 0 { r } else { r + 1 };
        debug_assert!(r * ag >= bytes);
        debug_assert!(r % 2 == 0);
        r
    }

    /// Mirrored memory buffer of length `len`.
    ///
    /// The buffer elements in range `[0, len/2)` are mirrored into the range
    /// `[len/2, len)`.
    pub struct Buffer<T> {
        /// Pointer to the first element in the buffer.
        ptr: NonNull<T>,
        /// Length of the buffer:
        ///
        /// * it is NOT always a multiple of 2
        /// * the elements in range `[0, len/2)` are mirrored into the range
        /// `[len/2, len)`.
        len: usize,
    }

    impl<T> Buffer<T> {
        /// Number of elements in the buffer.
        pub fn len(&self) -> usize {
            self.len
        }

        /// Is the buffer empty?
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        /// Pointer to the first element in the buffer.
        pub unsafe fn ptr(&self) -> *mut T {
            self.ptr.as_ptr()
        }

        /// Interprets contents as a slice.
        ///
        /// Warning: Some memory might be uninitialized.
        pub unsafe fn as_slice(&self) -> &[T] {
            unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len()) }
        }

        /// Interprets contents as a mut slice.
        ///
        /// Warning: Some memory might be uninitialized.
        pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
            unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len()) }
        }

        /// Interprets content as a slice and access the `i`-th element.
        ///
        /// Warning: The memory of the `i`-th element might be uninitialized.
        pub unsafe fn get(&self, i: usize) -> &T {
            unsafe { &self.as_slice()[i] }
        }

        /// Interprets content as a mut slice and access the `i`-th element.
        ///
        /// Warning: The memory of the `i`-th element might be uninitialized.
        pub unsafe fn get_mut(&mut self, i: usize) -> &mut T {
            unsafe { &mut self.as_mut_slice()[i] }
        }

        /// Creates a new empty `Buffer`.
        pub fn new() -> Self {
            // Zero-sized elements are not supported yet:
            assert!(mem::size_of::<T>() > 0);
            // Here `ptr` is initialized to a magic value but `len == 0`
            // will ensure that it is never dereferenced in this state.
            unsafe {
                Self {
                    ptr: NonNull::new_unchecked(mem::align_of::<T>() as *mut T),
                    len: 0,
                }
            }
        }

        /// Creates a new empty `Buffer` from a `ptr` and a `len`.
        ///
        /// # Panics
        ///
        /// If `ptr` is null.
        pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> Self {
            // Zero-sized types are not supported yet:
            assert!(mem::size_of::<T>() > 0);
            assert!(len % 2 == 0);
            assert!(!ptr.is_null());
            Self {
                ptr: unsafe { NonNull::new_unchecked(ptr) },
                len,
            }
        }

        /// Total number of bytes in the buffer (including mirrored memory).
        fn size_in_bytes(len: usize) -> usize {
            let v =
                no_required_allocation_units(len * mem::size_of::<T>()) * allocation_granularity();
            debug_assert!(
                v >= len * mem::size_of::<T>(),
                "len: {}, so<T>: {}, v: {}",
                len,
                mem::size_of::<T>(),
                v
            );
            v
        }

        /// Create a mirrored buffer containing `len` `T`s where the first half of
        /// the buffer is mirrored into the second half.
        pub fn uninitialized(len: usize) -> Result<Self, AllocError> {
            // Zero-sized types are not supported yet:
            assert!(mem::size_of::<T>() > 0);
            // The alignment requirements of `T` must be smaller than the
            // allocation granularity.
            assert!(mem::align_of::<T>() <= allocation_granularity());
            // To split the buffer in two halfs the number of elements must be a
            // multiple of two, and greater than zero to be able to mirror
            // something.
            if len == 0 {
                return Ok(Self::new());
            }
            assert!(len % 2 == 0);

            // How much memory we need:
            let alloc_size = Self::size_in_bytes(len);
            debug_assert!(alloc_size > 0);
            debug_assert!(alloc_size % 2 == 0);
            debug_assert!(alloc_size % allocation_granularity() == 0);
            debug_assert!(alloc_size >= len * mem::size_of::<T>());

            let ptr = allocate_mirrored(alloc_size)?;
            Ok(Self {
                ptr: unsafe { NonNull::new_unchecked(ptr as *mut T) },
                len: alloc_size / mem::size_of::<T>(),
                // Note: len is not a multiple of two: debug_assert!(len % 2 == 0);
            })
        }
    }

    impl<T> Drop for Buffer<T> {
        fn drop(&mut self) {
            if self.is_empty() {
                return;
            }

            let buffer_size_in_bytes = Self::size_in_bytes(self.len());
            let first_half_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { deallocate_mirrored(first_half_ptr, buffer_size_in_bytes) };
        }
    }

    impl<T> Clone for Buffer<T>
    where
        T: Clone,
    {
        fn clone(&self) -> Self {
            unsafe {
                let mid = self.len() / 2;
                let mut c = Self::uninitialized(self.len())
                    .expect("allocating a new mirrored buffer failed");
                let (from, _) = self.as_slice().split_at(mid);
                {
                    let (to, _) = c.as_mut_slice().split_at_mut(mid);
                    to[..mid].clone_from_slice(&from[..mid]);
                }
                c
            }
        }
    }

    impl<T> Default for Buffer<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    // Safe because it is possible to free this from a different thread
    unsafe impl<T> Send for Buffer<T> where T: Send {}
    // Safe because this doesn't use any kind of interior mutability.
    unsafe impl<T> Sync for Buffer<T> where T: Sync {}
}

extern crate core;

#[cfg(all(
    any(target_os = "macos", target_os = "ios"),
    not(feature = "unix_sysv")
))]
extern crate mach;

#[cfg(unix)]
extern crate libc;

#[cfg(target_os = "windows")]
extern crate winapi;

/// Allocation error.
pub enum AllocError {
    /// The system is Out-of-memory.
    Oom,
    /// Other allocation errors (not out-of-memory).
    ///
    /// Race conditions, exhausted file descriptors, etc.
    Other,
}

impl fmt::Debug for AllocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AllocError::Oom => write!(f, "out-of-memory"),
            AllocError::Other => write!(f, "other (not out-of-memory)"),
        }
    }
}

/// A stable version of the `core::intrinsics` module.
mod intrinsics {
    /// Like `core::intrinsics::unlikely` but does nothing.
    #[inline(always)]
    pub unsafe fn unlikely<T>(x: T) -> T {
        x
    }

    /// Like `core::intrinsics::assume` but does nothing.
    #[inline(always)]
    pub unsafe fn assume<T>(x: T) -> T {
        x
    }

    /// Like `core::intrinsics::arith_offset` but doing pointer to integer
    /// conversions.
    #[inline(always)]
    pub unsafe fn arith_offset<T>(dst: *const T, offset: isize) -> *const T {
        let r = if offset >= 0 {
            (dst as usize).wrapping_add(offset as usize)
        } else {
            (dst as usize).wrapping_sub((-offset) as usize)
        };
        r as *const T
    }
}

/// Stable implementation of `.offset_to` for pointers.
trait OffsetTo {
    /// Stable implementation of `.offset_to` for pointers.
    fn offset_to_(self, other: Self) -> Option<isize>;
}

/// Stable implementation of `.offset_to` for pointers.
trait OffsetToMut {
    /// A const pointer type.
    type Other;
    /// Stable implementation of `.offset_to` for pointers.
    fn offset_to_(self, other: Self::Other) -> Option<isize>;
}

impl<T: Sized> OffsetTo for *const T {
    #[inline(always)]
    fn offset_to_(self, other: Self) -> Option<isize>
    where
        T: Sized,
    {
        let size = mem::size_of::<T>();
        if size == 0 {
            None
        } else {
            let diff = (other as isize).wrapping_sub(self as isize);
            Some(diff / size as isize)
        }
    }
}

impl<T: Sized> OffsetToMut for *mut T {
    type Other = *const T;
    #[inline(always)]
    fn offset_to_(self, other: Self::Other) -> Option<isize>
    where
        T: Sized,
    {
        let size = mem::size_of::<T>();
        if size == 0 {
            None
        } else {
            let diff = (other as isize).wrapping_sub(self as isize);
            Some(diff / size as isize)
        }
    }
}

/// A double-ended queue that derefs into a slice.
///
/// It is implemented with a growable virtual ring buffer.
pub struct SliceDeque<T> {
    /// Index of the first element in the queue.
    head_: usize,
    /// Index of one past the last element in the queue.
    tail_: usize,
    /// Mirrored memory buffer.
    buf: Buffer<T>,
}

/// Implementation detail of the sdeq! macro.
#[doc(hidden)]
pub use mem::forget as __mem_forget;

/// Creates a [`SliceDeque`] containing the arguments.
///
/// `sdeq!` allows `SliceDeque`s to be defined with the same syntax as array
/// expressions. There are two forms of this macro:
///
/// - Create a [`SliceDeque`] containing a given list of elements:
///
/// ```
/// # #[macro_use] extern crate slice_deque;
/// # use slice_deque::SliceDeque;
/// # fn main() {
/// let v: SliceDeque<i32> = sdeq![1, 2, 3];
/// assert_eq!(v[0], 1);
/// assert_eq!(v[1], 2);
/// assert_eq!(v[2], 3);
/// # }
/// ```
///
/// - Create a [`SliceDeque`] from a given element and size:
///
/// ```
/// # #[macro_use] extern crate slice_deque;
/// # use slice_deque::SliceDeque;
/// # fn main() {
/// let v = sdeq![7; 3];
/// assert_eq!(v, [7, 7, 7]);
/// # }
/// ```
///
/// Note that unlike array expressions this syntax supports all elements
/// which implement `Clone` and the number of elements doesn't have to be
/// a constant.
///
/// This will use `clone` to duplicate an expression, so one should be careful
/// using this with types having a nonstandard `Clone` implementation. For
/// example, `sdeq![Rc::new(1); 5]` will create a deque of five references
/// to the same boxed integer value, not five references pointing to
/// independently boxed integers.
///
/// ```
/// # #[macro_use] extern crate slice_deque;
/// # use slice_deque::SliceDeque;
/// # use std::rc::Rc;
/// # fn main() {
/// let v = sdeq![Rc::new(1_i32); 5];
/// let ptr: *const i32 = &*v[0] as *const i32;
/// for i in v.iter() {
///     assert_eq!(Rc::into_raw(i.clone()), ptr);
/// }
/// # }
/// ```
///
/// [`SliceDeque`]: struct.SliceDeque.html
#[macro_export]
macro_rules! sdeq {
    ($elem:expr; $n:expr) => (
        {
            let mut deq = $crate::SliceDeque::with_capacity($n);
            deq.resize($n, $elem);
            deq
        }
    );
    () => ( $crate::SliceDeque::new() );
    ($($x:expr),*) => (
        {
            unsafe {
                let array = [$($x),*];
                let deq = $crate::SliceDeque::steal_from_slice(&array);
                #[cfg_attr(feature = "cargo-clippy", allow(clippy::forget_copy))]
                $crate::__mem_forget(array);
                deq
            }
        }
    );
    ($($x:expr,)*) => (sdeq![$($x),*])
}

impl<T> SliceDeque<T> {
    /// Creates a new empty deque.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let deq = SliceDeque::new();
    /// # let o: SliceDeque<u32> = deq;
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            head_: 0,
            tail_: 0,
            buf: Buffer::new(),
        }
    }

    /// Creates a SliceDeque from its raw components.
    ///
    /// The `ptr` must be a pointer to the beginning of the memory buffer from
    /// another `SliceDeque`, and `capacity` the capacity of this `SliceDeque`.
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut T, capacity: usize, head: usize, tail: usize) -> Self {
        debug_assert!(head <= tail);

        let d = Self {
            head_: head,
            tail_: tail,
            buf: unsafe { Buffer::from_raw_parts(ptr, capacity * 2) },
        };

        debug_assert!(d.tail() <= d.tail_upper_bound());
        debug_assert!(d.head() <= d.head_upper_bound());

        d
    }

    /// Create an empty deque with capacity to hold `n` elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let deq = SliceDeque::with_capacity(10);
    /// # let o: SliceDeque<u32> = deq;
    /// ```
    #[inline]
    pub fn with_capacity(n: usize) -> Self {
        unsafe {
            Self {
                head_: 0,
                tail_: 0,
                buf: Buffer::uninitialized(2 * n).unwrap_or_else(|e| {
                    let s = tiny_str!(
                        "failed to allocate a buffer with capacity \"{}\" due to \"{:?}\"",
                        n,
                        e
                    );
                    panic!("{}", s.as_str())
                }),
            }
        }
    }

    /// Returns the number of elements that the deque can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let deq = SliceDeque::with_capacity(10);
    /// assert!(deq.capacity() >= 10);
    /// # let o: SliceDeque<u32> = deq;
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        // Note: the buffer length is not necessarily a power of two
        // debug_assert!(self.buf.len() % 2 == 0);
        self.buf.len() / 2
    }

    /// Largest tail value
    #[inline]
    fn tail_upper_bound(&self) -> usize {
        self.capacity() * 2
    }

    /// Largest head value
    #[inline]
    fn head_upper_bound(&self) -> usize {
        self.capacity()
    }

    /// Get index to the head
    #[inline]
    fn head(&self) -> usize {
        self.head_
    }

    /// Get index to the tail
    #[inline]
    fn tail(&self) -> usize {
        self.tail_
    }

    /// Number of elements in the ring buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::with_capacity(10);
    /// assert!(deq.len() == 0);
    /// deq.push_back(3);
    /// assert!(deq.len() == 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        let l = self.tail() - self.head();
        debug_assert!(self.tail() >= self.head());
        debug_assert!(l <= self.capacity());
        l
    }

    /// Is the ring buffer full ?
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::with_capacity(10);
    /// assert!(!deq.is_full());
    /// # let o: SliceDeque<u32> = deq;
    /// ```
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Extracts a slice containing the entire deque.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let ptr = self.buf.ptr();
            let ptr = ptr.add(self.head());
            slice::from_raw_parts(ptr, self.len())
        }
    }

    /// Extracts a mutable slice containing the entire deque.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let ptr = self.buf.ptr();
            let ptr = ptr.add(self.head());
            slice::from_raw_parts_mut(ptr, self.len())
        }
    }

    /// Returns a pair of slices, where the first slice contains the contents
    /// of the deque and the second one is empty.
    #[inline]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        unsafe {
            let left = self.as_slice();
            let right = slice::from_raw_parts(usize::max_value() as *const _, 0);
            (left, right)
        }
    }

    /// Returns a pair of slices, where the first slice contains the contents
    /// of the deque and the second one is empty.
    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        unsafe {
            let left = self.as_mut_slice();
            let right = slice::from_raw_parts_mut(usize::max_value() as *mut _, 0);
            (left, right)
        }
    }

    /// Returns the slice of uninitialized memory between the `tail` and the
    /// `head`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # fn main() {
    /// let mut d = sdeq![1, 2, 3];
    /// let cap = d.capacity();
    /// let len = d.len();
    /// unsafe {
    ///     {
    ///         // This slice contains the uninitialized elements in
    ///         // the deque:
    ///         let mut s = d.tail_head_slice();
    ///         assert_eq!(s.len(), cap - len);
    ///         // We can write to them and for example bump the tail of
    ///         // the deque:
    ///         s[0] = 4;
    ///         s[1] = 5;
    ///     }
    ///     d.move_tail(2);
    /// }
    /// assert_eq!(d, sdeq![1, 2, 3, 4, 5]);
    /// # }
    /// ```
    pub unsafe fn tail_head_slice(&mut self) -> &mut [T] {
        let ptr = unsafe { self.buf.ptr() };
        let ptr = unsafe { ptr.add(self.tail()) };
        unsafe { slice::from_raw_parts_mut(ptr, self.capacity() - self.len()) }
    }

    /// Attempts to reserve capacity for inserting at least `additional`
    /// elements without reallocating. Does nothing if the capacity is already
    /// sufficient.
    ///
    /// The collection always reserves memory in multiples of the page size.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        let old_len = self.len();
        let new_cap = self.grow_policy(additional);
        self.reserve_capacity(new_cap)?;
        debug_assert!(self.capacity() >= old_len + additional);
        Ok(())
    }

    /// Reserves capacity for inserting at least `additional` elements without
    /// reallocating. Does nothing if the capacity is already sufficient.
    ///
    /// The collection always reserves memory in multiples of the page size.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize` or on OOM.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.try_reserve(additional).unwrap();
    }

    /// Attempts to reserve capacity for `new_capacity` elements. Does nothing
    /// if the capacity is already sufficient.
    #[inline]
    fn reserve_capacity(&mut self, new_capacity: usize) -> Result<(), AllocError> {
        unsafe {
            if new_capacity <= self.capacity() {
                return Ok(());
            }

            let mut new_buffer = Buffer::uninitialized(2 * new_capacity)?;
            debug_assert!(new_buffer.len() >= 2 * new_capacity);

            let len = self.len();
            // Move the elements from the current buffer
            // to the beginning of the new buffer:
            {
                let from_ptr = self.as_mut_ptr();
                let to_ptr = new_buffer.as_mut_slice().as_mut_ptr();
                ::core::ptr::copy_nonoverlapping(from_ptr, to_ptr, len);
            }

            // Exchange buffers
            mem::swap(&mut self.buf, &mut new_buffer);

            // Correct head and tail (we copied to the
            // beginning of the of the new buffer)
            self.head_ = 0;
            self.tail_ = len;

            Ok(())
        }
    }

    /// Reserves the minimum capacity for exactly `additional` more elements to
    /// be inserted in the given `SliceDeq<T>`. After calling `reserve_exact`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore capacity can not be relied upon to be precisely
    /// minimal. Prefer `reserve` if future insertions are expected.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # fn main() {
    /// let mut deq = sdeq![1];
    /// deq.reserve_exact(10);
    /// assert!(deq.capacity() >= 11);
    /// # }
    /// ```
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        let old_len = self.len();
        let new_cap = old_len.checked_add(additional).expect("overflow");
        self.reserve_capacity(new_cap).unwrap();
        debug_assert!(self.capacity() >= old_len + additional);
    }

    /// Growth policy of the deque. The capacity is going to be a multiple of
    /// the page-size anyways, so we just double capacity when needed.
    #[inline]
    fn grow_policy(&self, additional: usize) -> usize {
        let cur_cap = self.capacity();
        let old_len = self.len();
        let req_cap = old_len.checked_add(additional).expect("overflow");
        if req_cap > cur_cap {
            let dbl_cap = cur_cap.saturating_mul(2);
            cmp::max(req_cap, dbl_cap)
        } else {
            req_cap
        }
    }

    /// Moves the deque head by `x`.
    ///
    /// # Panics
    ///
    /// If the `head` wraps over the `tail` the behavior is undefined, that is,
    /// if `x` is out-of-range `[-(capacity() - len()), len()]`.
    ///
    /// If `-C debug-assertions=1` violating this pre-condition `panic!`s.
    ///
    /// # Unsafe
    ///
    /// It does not `drop` nor initialize elements, it just moves where the
    /// tail of the deque points to within the allocated buffer.
    #[inline]
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::cyclomatic_complexity))]
    pub unsafe fn move_head_unchecked(&mut self, x: isize) {
        // Make sure that the head does not wrap over the tail:
        debug_assert!(x >= -((self.capacity() - self.len()) as isize));
        debug_assert!(x <= self.len() as isize);
        let head = self.head() as isize;
        let mut new_head = head + x;
        let tail = self.tail() as isize;
        let cap = self.capacity();
        debug_assert!(new_head <= tail);
        debug_assert!(tail - new_head <= cap as isize);

        if unsafe { intrinsics::unlikely(new_head < 0) } {
            // If the new head is negative we shift the range by capacity to
            // move it towards the second mirrored memory region.
            debug_assert!(tail < cap as isize);
            new_head += cap as isize;
            debug_assert!(new_head >= 0);
            self.tail_ += cap;
        } else if new_head as usize >= cap {
            // cannot panic because new_head >= 0
            // If the new head is larger than the capacity, we shift the range
            // by -capacity to move it towards the first mirrored
            // memory region.
            debug_assert!(tail >= cap as isize);
            new_head -= cap as isize;
            debug_assert!(new_head >= 0);
            self.tail_ -= cap;
        }

        self.head_ = new_head as usize;
        debug_assert!(self.len() as isize == (tail - head) - x);
        debug_assert!(self.head() <= self.tail());

        debug_assert!(self.tail() <= self.tail_upper_bound());
        debug_assert!(self.head() <= self.head_upper_bound());

        debug_assert!(self.head() != self.capacity());
    }

    /// Moves the deque head by `x`.
    ///
    /// # Panics
    ///
    /// If the `head` wraps over the `tail`, that is, if `x` is out-of-range
    /// `[-(capacity() - len()), len()]`.
    ///
    /// # Unsafe
    ///
    /// It does not `drop` nor initialize elements, it just moves where the
    /// tail of the deque points to within the allocated buffer.
    #[inline]
    pub unsafe fn move_head(&mut self, x: isize) {
        assert!(x >= -((self.capacity() - self.len()) as isize) && x <= self.len() as isize);
        unsafe { self.move_head_unchecked(x) }
    }

    /// Moves the deque tail by `x`.
    ///
    /// # Panics
    ///
    /// If the `tail` wraps over the `head` the behavior is undefined, that is,
    /// if `x` is out-of-range `[-len(), capacity() - len()]`.
    ///
    /// If `-C debug-assertions=1` violating this pre-condition `panic!`s.
    ///
    /// # Unsafe
    ///
    /// It does not `drop` nor initialize elements, it just moves where the
    /// tail of the deque points to within the allocated buffer.
    #[inline]
    pub unsafe fn move_tail_unchecked(&mut self, x: isize) {
        // Make sure that the tail does not wrap over the head:
        debug_assert!(x >= -(self.len() as isize));
        debug_assert!(
            x <= (self.capacity() - self.len()) as isize,
            "x = {}, len = {}, cap = {}",
            x,
            self.len(),
            self.capacity()
        );
        let head = self.head() as isize;
        let tail = self.tail() as isize;
        let cap = self.capacity() as isize;
        let mut new_tail = tail + x;
        debug_assert!(new_tail >= 0);
        debug_assert!(head <= new_tail);
        debug_assert!(new_tail - head <= cap);

        // If the new tail falls of the mirrored region of virtual memory we
        // shift the range by -capacity to move it towards the first mirrored
        // memory region.

        if unsafe { intrinsics::unlikely(new_tail >= 2 * cap) } {
            debug_assert!(head >= cap);
            self.head_ -= cap as usize;
            new_tail -= cap as isize;
            debug_assert!(new_tail <= cap);
        }

        self.tail_ = new_tail as usize;
        debug_assert!(self.len() as isize == (tail - head) + x);

        debug_assert!(self.tail() <= self.tail_upper_bound());
        debug_assert!(self.head() <= self.head_upper_bound());
    }

    /// Moves the deque tail by `x`.
    ///
    /// # Panics
    ///
    /// If the `tail` wraps over the `head`, that is, if `x` is out-of-range
    /// `[-len(), capacity() - len()]`.
    ///
    /// # Unsafe
    ///
    /// It does not `drop` nor initialize elements, it just moves where the
    /// tail of the deque points to within the allocated buffer.
    #[inline]
    pub unsafe fn move_tail(&mut self, x: isize) {
        assert!(x >= -(self.len() as isize) && x <= (self.capacity() - self.len()) as isize);
        unsafe { self.move_tail_unchecked(x) };
    }

    /// Appends elements to `self` from `other`.
    #[inline]
    unsafe fn append_elements(&mut self, other: *const [T]) {
        let count = unsafe { (*other).len() };
        self.reserve(count);
        let len = self.len();
        unsafe { ptr::copy_nonoverlapping(other as *const T, self.get_unchecked_mut(len), count) };
        unsafe { self.move_tail_unchecked(count as isize) };
    }

    /// Steal the elements from the slice `s`. You should `mem::forget` the
    /// slice afterwards.
    pub unsafe fn steal_from_slice(s: &[T]) -> Self {
        let mut deq = Self::new();
        unsafe { deq.append_elements(s as *const _) };
        deq
    }

    /// Moves all the elements of `other` into `Self`, leaving `other` empty.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements in the deque overflows a `isize`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![1, 2, 3];
    /// let mut deq2 = sdeq![4, 5, 6];
    /// deq.append(&mut deq2);
    /// assert_eq!(deq, [1, 2, 3, 4, 5, 6]);
    /// assert_eq!(deq2, []);
    /// # }
    /// ```
    #[inline]
    pub fn append(&mut self, other: &mut Self) {
        unsafe {
            self.append_elements(other.as_slice() as _);
            other.head_ = 0;
            other.tail_ = 0;
        }
    }

    /// Provides a reference to the first element, or `None` if the deque is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// assert_eq!(deq.front(), None);
    ///
    /// deq.push_back(1);
    /// deq.push_back(2);
    /// assert_eq!(deq.front(), Some(&1));
    /// deq.push_front(3);
    /// assert_eq!(deq.front(), Some(&3));
    /// ```
    #[inline]
    pub fn front(&self) -> Option<&T> {
        self.get(0)
    }

    /// Provides a mutable reference to the first element, or `None` if the
    /// deque is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// assert_eq!(deq.front(), None);
    ///
    /// deq.push_back(1);
    /// deq.push_back(2);
    /// assert_eq!(deq.front(), Some(&1));
    /// (*deq.front_mut().unwrap()) = 3;
    /// assert_eq!(deq.front(), Some(&3));
    /// ```
    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }

    /// Provides a reference to the last element, or `None` if the deque is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// assert_eq!(deq.back(), None);
    ///
    /// deq.push_back(1);
    /// deq.push_back(2);
    /// assert_eq!(deq.back(), Some(&2));
    /// deq.push_front(3);
    /// assert_eq!(deq.back(), Some(&2));
    /// ```
    #[inline]
    pub fn back(&self) -> Option<&T> {
        let last_idx = self.len().wrapping_sub(1);
        self.get(last_idx)
    }

    /// Provides a mutable reference to the last element, or `None` if the
    /// deque is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// assert_eq!(deq.front(), None);
    ///
    /// deq.push_back(1);
    /// deq.push_back(2);
    /// assert_eq!(deq.back(), Some(&2));
    /// (*deq.back_mut().unwrap()) = 3;
    /// assert_eq!(deq.back(), Some(&3));
    /// ```
    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        let last_idx = self.len().wrapping_sub(1);
        self.get_mut(last_idx)
    }

    /// Attempts to prepend `value` to the deque.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// deq.try_push_front(1).unwrap();
    /// deq.try_push_front(2).unwrap();
    /// assert_eq!(deq.front(), Some(&2));
    /// ```
    #[inline]
    pub fn try_push_front(&mut self, value: T) -> Result<(), (T, AllocError)> {
        unsafe {
            if intrinsics::unlikely(self.is_full()) {
                if let Err(e) = self.try_reserve(1) {
                    return Err((value, e));
                }
            }

            self.move_head_unchecked(-1);
            ptr::write(self.get_mut(0).unwrap(), value);
            Ok(())
        }
    }

    /// Prepends `value` to the deque.
    ///
    /// # Panics
    ///
    /// On OOM.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// deq.push_front(1);
    /// deq.push_front(2);
    /// assert_eq!(deq.front(), Some(&2));
    /// ```
    #[inline]
    pub fn push_front(&mut self, value: T) {
        if let Err(e) = self.try_push_front(value) {
            panic!("{:?}", e.1);
        }
    }

    /// Attempts to appends `value` to the deque.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// deq.try_push_back(1).unwrap();
    /// deq.try_push_back(3).unwrap();
    /// assert_eq!(deq.back(), Some(&3));
    /// ```
    #[inline]
    pub fn try_push_back(&mut self, value: T) -> Result<(), (T, AllocError)> {
        unsafe {
            if intrinsics::unlikely(self.is_full()) {
                if let Err(e) = self.try_reserve(1) {
                    return Err((value, e));
                }
            }
            self.move_tail_unchecked(1);
            let len = self.len();
            ptr::write(self.get_mut(len - 1).unwrap(), value);
            Ok(())
        }
    }

    /// Appends `value` to the deque.
    ///
    /// # Panics
    ///
    /// On OOM.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// deq.push_back(1);
    /// deq.push_back(3);
    /// assert_eq!(deq.back(), Some(&3));
    /// ```
    #[inline]
    pub fn push_back(&mut self, value: T) {
        if let Err(e) = self.try_push_back(value) {
            panic!("{:?}", e.1);
        }
    }

    /// Removes the first element and returns it, or `None` if the deque is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// assert_eq!(deq.pop_front(), None);
    ///
    /// deq.push_back(1);
    /// deq.push_back(2);
    ///
    /// assert_eq!(deq.pop_front(), Some(1));
    /// assert_eq!(deq.pop_front(), Some(2));
    /// assert_eq!(deq.pop_front(), None);
    /// ```
    #[inline]
    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            let v = match self.get_mut(0) {
                None => return None,
                Some(v) => ptr::read(v),
            };
            self.move_head_unchecked(1);
            Some(v)
        }
    }

    /// Removes the last element from the deque and returns it, or `None` if it
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// assert_eq!(deq.pop_back(), None);
    ///
    /// deq.push_back(1);
    /// deq.push_back(3);
    ///
    /// assert_eq!(deq.pop_back(), Some(3));
    /// assert_eq!(deq.pop_back(), Some(1));
    /// assert_eq!(deq.pop_back(), None);
    /// ```
    #[inline]
    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            let len = self.len();
            let v = match self.get_mut(len.wrapping_sub(1)) {
                None => return None,
                Some(v) => ptr::read(v),
            };
            self.move_tail_unchecked(-1);
            Some(v)
        }
    }

    /// Shrinks the capacity of the deque as much as possible.
    ///
    /// It will drop down as close as possible to the length, but because
    /// `SliceDeque` allocates memory in multiples of the page size the deque
    /// might still have capacity for inserting new elements without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::with_capacity(15);
    /// deq.extend(0..4);
    /// assert!(deq.capacity() >= 15);
    /// deq.shrink_to_fit();
    /// assert!(deq.capacity() >= 4);
    /// # let o: SliceDeque<u32> = deq;
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        if unsafe { intrinsics::unlikely(self.is_empty()) } {
            return;
        }

        let mut new_vd = Self::with_capacity(self.len());
        if new_vd.capacity() < self.capacity() {
            unsafe {
                ::core::ptr::copy_nonoverlapping(
                    self.as_mut_ptr(),
                    new_vd.as_mut_ptr(),
                    self.len(),
                );
            }
            new_vd.tail_ = self.len();
            mem::swap(self, &mut new_vd);
        }
    }

    /// Shortens the deque by removing excess elements from the back.
    ///
    /// If `len` is greater than the SliceDeque's current length, this has no
    /// effect.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![5, 10, 15];
    /// assert_eq!(deq, [5, 10, 15]);
    /// deq.truncate_back(1);
    /// assert_eq!(deq, [5]);
    /// # }
    /// ```
    #[inline]
    pub fn truncate_back(&mut self, len: usize) {
        unsafe {
            while len < self.len() {
                // decrement tail before the drop_in_place(), so a panic on
                // Drop doesn't re-drop the just-failed value.
                self.move_tail(-1);
                let len = self.len();
                core::ptr::drop_in_place(self.get_unchecked_mut(len));
            }
        }
    }

    /// Shortens the deque by removing excess elements from the back.
    ///
    /// If `len` is greater than the SliceDeque's current length, this has no
    /// effect. See `truncate_back` for examples.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.truncate_back(len);
    }

    /// Shortens the deque by removing excess elements from the front.
    ///
    /// If `len` is greater than the SliceDeque's current length, this has no
    /// effect.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![5, 10, 15];
    /// assert_eq!(deq, [5, 10, 15]);
    /// deq.truncate_front(1);
    /// assert_eq!(deq, [15]);
    /// # }
    /// ```
    #[inline]
    pub fn truncate_front(&mut self, len: usize) {
        unsafe {
            while len < self.len() {
                let head: *mut T = self.get_unchecked_mut(0) as *mut _;
                // increment head before the drop_in_place(), so a panic on
                // Drop doesn't re-drop the just-failed value.
                self.move_head(1);
                core::ptr::drop_in_place(head);
            }
        }
    }

    /// Removes all values from the deque.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![1];
    /// assert!(!deq.is_empty());
    /// deq.clear();
    /// assert!(deq.is_empty());
    /// # }
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Removes the element at `index` and return it in `O(1)` by swapping the
    /// last element into its place.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// assert_eq!(deq.swap_remove_back(0), None);
    /// deq.extend(1..4);
    /// assert_eq!(deq, [1, 2, 3]);
    ///
    /// assert_eq!(deq.swap_remove_back(0), Some(1));
    /// assert_eq!(deq, [3, 2]);
    /// ```
    #[inline]
    pub fn swap_remove_back(&mut self, index: usize) -> Option<T> {
        let len = self.len();
        if self.is_empty() {
            None
        } else {
            self.swap(index, len - 1);
            self.pop_back()
        }
    }

    /// Removes the element at `index` and returns it in `O(1)` by swapping the
    /// first element into its place.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// assert_eq!(deq.swap_remove_front(0), None);
    /// deq.extend(1..4);
    /// assert_eq!(deq, [1, 2, 3]);
    ///
    /// assert_eq!(deq.swap_remove_front(2), Some(3));
    /// assert_eq!(deq, [2, 1]);
    /// ```
    #[inline]
    pub fn swap_remove_front(&mut self, index: usize) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.swap(index, 0);
            self.pop_front()
        }
    }

    /// Inserts an `element` at `index` within the deque, shifting all elements
    /// with indices greater than or equal to `index` towards the back.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if `index` is greater than deque's length
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq!['a', 'b', 'c'];
    /// assert_eq!(deq, &['a', 'b', 'c']);
    ///
    /// deq.insert(1, 'd');
    /// assert_eq!(deq, &['a', 'd', 'b', 'c']);
    /// # }
    /// ```
    #[inline]
    pub fn insert(&mut self, index: usize, element: T) {
        unsafe {
            let len = self.len();
            assert!(index <= len);

            if intrinsics::unlikely(self.is_full()) {
                self.reserve(1);
                // TODO: when the deque needs to grow, reserve should
                // copy the memory to the new storage leaving a whole
                // at the index where the new elements are to be inserted
                // to avoid having to copy the memory again
            }

            let p = if index > self.len() / 2 {
                let p = unsafe { self.as_mut_ptr().add(index) };
                // Shift elements towards the back
                ptr::copy(p, p.add(1), len - index);
                self.move_tail_unchecked(1);
                p
            } else {
                // Shift elements towards the front
                self.move_head_unchecked(-1);
                let p = unsafe { self.as_mut_ptr().add(index) };
                ptr::copy(p, p.sub(1), index);
                p
            };
            ptr::write(p, element); // Overwritte
        }
    }

    /// Removes and returns the element at position `index` within the deque.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![1, 2, 3, 4, 5];
    /// assert_eq!(deq.remove(1), 2);
    /// assert_eq!(deq, [1, 3, 4, 5]);
    /// # }
    /// ```
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        let len = self.len();
        assert!(index < len);
        unsafe {
            // copy element at pointer:
            let ptr = unsafe { self.as_mut_ptr().add(index) };
            let ret = ptr::read(ptr);
            if index > self.len() / 2 {
                // If the index is close to the back, shift elements from the
                // back towards the front
                ptr::copy(ptr.add(1), ptr, len - index - 1);
                self.move_tail_unchecked(-1);
            } else {
                // If the index is close to the front, shift elements from the
                // front towards the back
                let ptr = self.as_mut_ptr();
                ptr::copy(ptr, ptr.add(1), index);
                self.move_head_unchecked(1);
            }

            ret
        }
    }

    /// Splits the collection into two at the given index.
    ///
    /// Returns a newly allocated `Self`. `self` contains elements `[0, at)`,
    /// and the returned `Self` contains elements `[at, len)`.
    ///
    /// Note that the capacity of `self` does not change.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![1, 2, 3];
    /// let deq2 = deq.split_off(1);
    /// assert_eq!(deq, [1]);
    /// assert_eq!(deq2, [2, 3]);
    /// # }
    /// ```
    #[inline]
    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(at <= self.len(), "`at` out of bounds");

        let other_len = self.len() - at;
        let mut other = Self::with_capacity(other_len);

        unsafe {
            self.move_tail_unchecked(-(other_len as isize));
            other.move_tail_unchecked(other_len as isize);

            ptr::copy_nonoverlapping(self.as_ptr().add(at), other.as_mut_ptr(), other.len());
        }
        other
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// That is, remove all elements `e` such that `f(&e)` returns `false`.
    /// This method operates in place and preserves the order of the
    /// retained elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![1, 2, 3, 4];
    /// deq.retain(|&x| x % 2 == 0);
    /// assert_eq!(deq, [2, 4]);
    /// # }
    /// ```
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;

            for i in 0..len {
                if !f(&v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        if del > 0 {
            self.truncate(len - del);
        }
    }

    /// Removes all but the first of consecutive elements in the deque that
    /// resolve to the same key.
    ///
    /// If the deque is sorted, this removes all duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![10, 20, 21, 30, 20];
    ///
    /// deq.dedup_by_key(|i| *i / 10);
    /// assert_eq!(deq, [10, 20, 30, 20]);
    /// # }
    /// ```
    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.dedup_by(|a, b| key(a) == key(b))
    }

    /// Removes all but the first of consecutive elements in the deque
    /// satisfying a given equality relation.
    ///
    /// The `same_bucket` function is passed references to two elements from
    /// the deque, and returns `true` if the elements compare equal, or
    /// `false` if they do not. The elements are passed in opposite order
    /// from their order in the deque, so if `same_bucket(a, b)` returns
    /// `true`, `a` is removed.
    ///
    /// If the deque is sorted, this removes all duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq!["foo", "bar", "Bar", "baz", "bar"];
    ///
    /// deq.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    ///
    /// assert_eq!(deq, ["foo", "bar", "baz", "bar"]);
    /// # }
    /// ```
    #[inline]
    pub fn dedup_by<F>(&mut self, mut same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        unsafe {
            // Although we have a mutable reference to `self`, we cannot make
            // *arbitrary* changes. The `same_bucket` calls could panic, so we
            // must ensure that the deque is in a valid state at all time.
            //
            // The way that we handle this is by using swaps; we iterate
            // over all the elements, swapping as we go so that at the end
            // the elements we wish to keep are in the front, and those we
            // wish to reject are at the back. We can then truncate the
            // deque. This operation is still O(n).
            //
            // Example: We start in this state, where `r` represents "next
            // read" and `w` represents "next_write`.
            //
            //           r
            //     +---+---+---+---+---+---+
            //     | 0 | 1 | 1 | 2 | 3 | 3 |
            //     +---+---+---+---+---+---+
            //           w
            //
            // Comparing self[r] against self[w-1], this is not a duplicate, so
            // we swap self[r] and self[w] (no effect as r==w) and then
            // increment both r and w, leaving us with:
            //
            //               r
            //     +---+---+---+---+---+---+
            //     | 0 | 1 | 1 | 2 | 3 | 3 |
            //     +---+---+---+---+---+---+
            //               w
            //
            // Comparing self[r] against self[w-1], this value is a duplicate,
            // so we increment `r` but leave everything else unchanged:
            //
            //                   r
            //     +---+---+---+---+---+---+
            //     | 0 | 1 | 1 | 2 | 3 | 3 |
            //     +---+---+---+---+---+---+
            //               w
            //
            // Comparing self[r] against self[w-1], this is not a duplicate,
            // so swap self[r] and self[w] and advance r and w:
            //
            //                       r
            //     +---+---+---+---+---+---+
            //     | 0 | 1 | 2 | 1 | 3 | 3 |
            //     +---+---+---+---+---+---+
            //                   w
            //
            // Not a duplicate, repeat:
            //
            //                           r
            //     +---+---+---+---+---+---+
            //     | 0 | 1 | 2 | 3 | 1 | 3 |
            //     +---+---+---+---+---+---+
            //                       w
            //
            // Duplicate, advance r. End of deque. Truncate to w.

            let ln = self.len();
            if intrinsics::unlikely(ln <= 1) {
                return;
            }

            // Avoid bounds checks by using raw pointers.
            let p = self.as_mut_ptr();
            let mut r: usize = 1;
            let mut w: usize = 1;

            while r < ln {
                let p_r = p.add(r);
                let p_wm1 = p.add(w - 1);
                if !same_bucket(&mut *p_r, &mut *p_wm1) {
                    if r != w {
                        let p_w = p_wm1.add(1);
                        unsafe { mem::swap(&mut *p_r, &mut *p_w) };
                    }
                    w += 1;
                }
                r += 1;
            }

            self.truncate(w);
        }
    }

    /// Extend the `SliceDeque` by `n` values, using the given generator.
    #[inline]
    fn extend_with<E: ExtendWith<T>>(&mut self, n: usize, value: E) {
        self.reserve(n);

        unsafe {
            let mut ptr = self.as_mut_ptr().add(self.len());

            // Write all elements except the last one
            for _ in 1..n {
                ptr::write(ptr, value.next());
                ptr = ptr.add(1);
                // Increment the length in every step in case next() panics
                self.move_tail_unchecked(1);
            }

            if n > 0 {
                // We can write the last element directly without cloning
                // needlessly
                ptr::write(ptr, value.last());
                self.move_tail_unchecked(1);
            }

            // len set by scope guard
        }
    }

    /// Extend for a general iterator.
    ///
    /// This function should be the moral equivalent of:
    ///
    /// >  for item in iterator {
    /// >      self.push_back(item);
    /// >  }
    #[inline]
    fn extend_desugared<I: Iterator<Item = T>>(&mut self, mut iterator: I) {
        #[cfg_attr(feature = "cargo-clippy", allow(clippy::while_let_on_iterator))]
        while let Some(element) = iterator.next() {
            let len = self.len();
            let cap = self.capacity();
            if len == cap {
                let (lower, upper) = iterator.size_hint();
                let additional_cap = if let Some(upper) = upper {
                    upper
                } else {
                    lower
                }
                .checked_add(1)
                .expect("overflow");
                self.reserve(additional_cap);
            }
            debug_assert!(self.len() < self.capacity());
            unsafe {
                ptr::write(self.get_unchecked_mut(len), element);
                // NB can't overflow since we would have had to alloc the
                // address space
                self.move_tail_unchecked(1);
            }
        }
    }

    /// Creates an iterator which uses a closure to determine if an element
    /// should be removed.
    ///
    /// If the closure returns `true`, then the element is removed and yielded.
    /// If the closure returns `false`, it will try again, and call the closure
    /// on the next element, seeing if it passes the test.
    ///
    /// Using this method is equivalent to the following code:
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// # let some_predicate = |x: &mut i32| { *x == 2 || *x == 3 || *x == 6
    /// # };
    /// let mut deq = SliceDeque::new();
    /// deq.extend(1..7);
    /// let mut i = 0;
    /// while i != deq.len() {
    ///     if some_predicate(&mut deq[i]) {
    ///         let val = deq.remove(i);
    ///     // your code here
    ///     } else {
    ///         i += 1;
    ///     }
    /// }
    /// # let mut expected = sdeq![1, 4, 5];
    /// # assert_eq!(deq, expected);
    /// # }
    /// ```
    ///
    /// But `drain_filter` is easier to use. `drain_filter` is also more
    /// efficient, because it can backshift the elements of the deque in
    /// bulk.
    ///
    /// Note that `drain_filter` also lets you mutate every element in the
    /// filter closure, regardless of whether you choose to keep or remove
    /// it.
    ///
    ///
    /// # Examples
    ///
    /// Splitting a deque into evens and odds, reusing the original allocation:
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut numbers = sdeq![1, 2, 3, 4, 5, 6, 8, 9, 11, 13, 14, 15];
    ///
    /// let evens = numbers
    ///     .drain_filter(|x| *x % 2 == 0)
    ///     .collect::<SliceDeque<_>>();
    /// let odds = numbers;
    ///
    /// assert_eq!(sdeq![2, 4, 6, 8, 14], evens);
    /// assert_eq!(odds, sdeq![1, 3, 5, 9, 11, 13, 15]);
    /// # }
    /// ```
    #[inline]
    pub fn drain_filter<F>(&mut self, filter: F) -> DrainFilter<T, F>
    where
        F: FnMut(&mut T) -> bool,
    {
        let old_len = self.len();

        // Guard against us getting leaked (leak amplification)
        unsafe {
            self.move_tail_unchecked(-(old_len as isize));
        }

        DrainFilter {
            deq: self,
            idx: 0,
            del: 0,
            old_len,
            pred: filter,
        }
    }

    // TODO: fn place_back(&mut self) -> PlaceBack<T>
    // TODO: fn place_front(&mut self) -> PlaceFront<T>
}

impl<T> SliceDeque<T>
where
    T: Clone,
{
    /// Clones and appends all elements in a slice to the `SliceDeque`.
    ///
    /// Iterates over the slice `other`, clones each element, and then appends
    /// it to this `SliceDeque`. The `other` slice is traversed in-order.
    ///
    /// Note that this function is same as `extend` except that it is
    /// specialized to work with slices instead. If and when Rust gets
    /// specialization this function will likely be deprecated (but still
    /// available).
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice_deque::SliceDeque;
    /// let mut deq = SliceDeque::new();
    /// deq.push_back(1);
    /// deq.extend_from_slice(&[2, 3, 4]);
    /// assert_eq!(deq, [1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[T]) {
        {
            self.reserve(other.len());
            unsafe {
                let len = self.len();
                self.move_tail_unchecked(other.len() as isize);
                self.get_unchecked_mut(len..).clone_from_slice(other);
            }
        }
    }

    /// Modifies the `SliceDeque` in-place so that `len()` is equal to
    /// `new_len`, either by removing excess elements or by appending clones of
    /// `value` to the back.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![5, 10, 15];
    /// assert_eq!(deq, [5, 10, 15]);
    ///
    /// deq.resize(2, 0);
    /// assert_eq!(deq, [5, 10]);
    ///
    /// deq.resize(5, 20);
    /// assert_eq!(deq, [5, 10, 20, 20, 20]);
    /// # }
    /// ```
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T) {
        let len = self.len();

        if new_len > len {
            self.reserve(new_len - len);
            while self.len() < new_len {
                self.push_back(value.clone());
            }
        } else {
            self.truncate(new_len);
        }
        debug_assert!(self.len() == new_len);
    }
}

impl<T: Default> SliceDeque<T> {
    /// Resizes the `SliceDeque` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `SliceDeque` is extended by the
    /// difference, with each additional slot filled with `Default::default()`.
    /// If `new_len` is less than `len`, the `SliceDeque` is simply truncated.
    ///
    /// This method uses `Default` to create new values on every push. If
    /// you'd rather `Clone` a given value, use [`resize`].
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![1, 2, 3];
    /// deq.resize_default(5);
    /// assert_eq!(deq, [1, 2, 3, 0, 0]);
    ///
    /// deq.resize_default(2);
    /// assert_eq!(deq, [1, 2]);
    /// # }
    /// ```
    ///
    /// [`resize`]: #method.resize
    #[inline]
    pub fn resize_default(&mut self, new_len: usize) {
        let len = self.len();

        if new_len > len {
            self.extend_with(new_len - len, ExtendDefault);
        } else {
            self.truncate(new_len);
        }
    }
}

impl<T: PartialEq> SliceDeque<T> {
    /// Removes consecutive repeated elements in the deque.
    ///
    /// If the deque is sorted, this removes all duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![1, 2, 2, 3, 2];
    ///
    /// deq.dedup();
    /// assert_eq!(deq, [1, 2, 3, 2]);
    ///
    /// deq.sort();
    /// assert_eq!(deq, [1, 2, 2, 3]);
    ///
    /// deq.dedup();
    /// assert_eq!(deq, [1, 2, 3]);
    /// # }
    /// ```
    #[inline]
    pub fn dedup(&mut self) {
        self.dedup_by(|a, b| a == b)
    }

    /// Removes the first instance of `item` from the deque if the item exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq![1, 2, 3, 1];
    ///
    /// deq.remove_item(&1);
    /// assert_eq!(deq, &[2, 3, 1]);
    /// deq.remove_item(&1);
    /// assert_eq!(deq, &[2, 3]);
    /// # }
    /// ```
    #[inline]
    pub fn remove_item(&mut self, item: &T) -> Option<T> {
        let pos = match self.iter().position(|x| *x == *item) {
            Some(x) => x,
            None => return None,
        };
        Some(self.remove(pos))
    }
}

impl<T: fmt::Debug> fmt::Debug for SliceDeque<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.as_slice())
        /*
         write!(
             f,
             // TODO: "SliceDeque({:?})",
             "SliceDeque(len: {}, cap: {}, head: {}, tail: {}, elems: {:?})",
             self.len(),
             self.capacity(),
             self.head(),
             self.tail(),
             self.as_slice()
         )
        */
    }
}

impl<T> Drop for SliceDeque<T> {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> ops::Deref for SliceDeque<T> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> ops::DerefMut for SliceDeque<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T> Default for SliceDeque<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for SliceDeque<T> {
    #[inline]
    fn clone(&self) -> Self {
        let mut new = Self::with_capacity(self.len());
        for i in self.iter() {
            new.push_back(i.clone());
        }
        new
    }
    #[inline]
    fn clone_from(&mut self, other: &Self) {
        self.clear();
        for i in other.iter() {
            self.push_back(i.clone());
        }
    }
}

impl<'a, T: Clone> From<&'a [T]> for SliceDeque<T> {
    #[inline]
    fn from(s: &'a [T]) -> Self {
        let mut new = Self::with_capacity(s.len());
        for i in s {
            new.push_back(i.clone());
        }
        new
    }
}

impl<'a, T: Clone> From<&'a mut [T]> for SliceDeque<T> {
    #[inline]
    fn from(s: &'a mut [T]) -> Self {
        let mut new = Self::with_capacity(s.len());
        for i in s {
            new.push_back(i.clone());
        }
        new
    }
}

impl<T: hash::Hash> hash::Hash for SliceDeque<T> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(&**self, state)
    }
}

///////////////////////////////////////////////////////////////////////////////
// PartialEq implementations:

macro_rules! __impl_slice_eq1 {
    ($Lhs:ty, $Rhs:ty) => {
        __impl_slice_eq1! { $Lhs, $Rhs, Sized }
    };
    ($Lhs:ty, $Rhs:ty, $Bound:ident) => {
        impl<'a, 'b, A: $Bound, B> PartialEq<$Rhs> for $Lhs
        where
            A: PartialEq<B>,
        {
            #[inline]
            fn eq(&self, other: &$Rhs) -> bool {
                self[..] == other[..]
            }
        }
    };
}

__impl_slice_eq1! { SliceDeque<A>, SliceDeque<B> }
__impl_slice_eq1! { SliceDeque<A>, &'b [B] }
__impl_slice_eq1! { SliceDeque<A>, &'b mut [B] }

#[cfg(feature = "use_std")]
__impl_slice_eq1! { SliceDeque<A>, Vec<B> }

macro_rules! array_impls {
    ($($N: expr)+) => {
        $(
            // NOTE: some less important impls are omitted to reduce code bloat
            __impl_slice_eq1! { SliceDeque<A>, [B; $N] }
            __impl_slice_eq1! { SliceDeque<A>, &'b [B; $N] }
        )+
    }
}

array_impls! {
    0  1  2  3  4  5  6  7  8  9
        10 11 12 13 14 15 16 17 18 19
        20 21 22 23 24 25 26 27 28 29
        30 31 32
}

///////////////////////////////////////////////////////////////////////////////

impl<T: Eq> Eq for SliceDeque<T> {}

impl<T: PartialOrd> PartialOrd for SliceDeque<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<'a, T: PartialOrd> PartialOrd<&'a [T]> for SliceDeque<T> {
    #[inline]
    fn partial_cmp(&self, other: &&'a [T]) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, other)
    }
}

/// A draining iterator for `SliceDeque<T>`.
///
/// This `struct` is created by the [`drain`] method on [`SliceDeque`].
///
/// [`drain`]: struct.SliceDeque.html#method.drain
/// [`SliceDeque`]: struct.SliceDeque.html
pub struct Drain<'a, T: 'a> {
    /// Index of tail to preserve
    tail_start: usize,
    /// Length of tail
    tail_len: usize,
    /// Current remaining range to remove
    iter: slice::Iter<'a, T>,
    /// A shared mutable pointer to the deque (with shared ownership).
    deq: NonNull<SliceDeque<T>>,
}

impl<'a, T: 'a + fmt::Debug> fmt::Debug for Drain<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Drain").field(&self.iter.as_slice()).finish()
    }
}

unsafe impl<'a, T: Sync> Sync for Drain<'a, T> {}
unsafe impl<'a, T: Send> Send for Drain<'a, T> {}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.iter
            .next()
            .map(|elt| unsafe { ptr::read(elt as *const _) })
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.iter
            .next_back()
            .map(|elt| unsafe { ptr::read(elt as *const _) })
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    #[inline]
    fn drop(&mut self) {
        // exhaust self first
        self.for_each(|_| {});

        if self.tail_len > 0 {
            unsafe {
                let source_deq = self.deq.as_mut();
                // memmove back untouched tail, update to new length
                let start = source_deq.len();
                let tail = self.tail_start;
                let src = source_deq.as_ptr().add(tail);
                let dst = source_deq.as_mut_ptr().add(start);
                ptr::copy(src, dst, self.tail_len);
                source_deq.move_tail_unchecked(self.tail_len as isize);
            }
        }
    }
}

/// An iterator that moves out of a deque.
///
/// This `struct` is created by the `into_iter` method on
/// [`SliceDeque`][`SliceDeque`] (provided by the [`IntoIterator`] trait).
///
/// [`SliceDeque`]: struct.SliceDeque.html
/// [`IntoIterator`]: ../../std/iter/trait.IntoIterator.html
pub struct IntoIter<T> {
    /// NonNull pointer to the buffer
    buf: NonNull<T>,
    /// Capacity of the buffer.
    cap: usize,
    /// Pointer to the first element.
    ptr: *const T,
    /// Pointer to one-past-the-end.
    end: *const T,
}

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
    }
}

impl<T> IntoIter<T> {
    /// Returns the remaining items of this iterator as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq!['a', 'b', 'c'];
    /// let mut into_iter = deq.into_iter();
    /// assert_eq!(into_iter.as_slice(), ['a', 'b', 'c']);
    /// let _ = into_iter.next().unwrap();
    /// assert_eq!(into_iter.as_slice(), ['b', 'c']);
    /// # }
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.size_hint().0) }
    }

    /// Returns the remaining items of this iterator as a mutable slice.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq!['a', 'b', 'c'];
    /// let mut into_iter = deq.into_iter();
    /// assert_eq!(into_iter.as_slice(), ['a', 'b', 'c']);
    /// into_iter.as_mut_slice()[2] = 'z';
    /// assert_eq!(into_iter.next().unwrap(), 'a');
    /// assert_eq!(into_iter.next().unwrap(), 'b');
    /// assert_eq!(into_iter.next().unwrap(), 'z');
    /// # }
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr as *mut T, self.size_hint().0) }
    }
}

unsafe impl<T: Send> Send for IntoIter<T> {}
unsafe impl<T: Sync> Sync for IntoIter<T> {}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        unsafe {
            if self.ptr as *const _ == self.end {
                None
            } else if mem::size_of::<T>() == 0 {
                // purposefully don't use 'ptr.offset' because for
                // deques with 0-size elements this would return the
                // same pointer.
                self.ptr = intrinsics::arith_offset(self.ptr as *const i8, 1) as *mut T;

                // Use a non-null pointer value
                // (self.ptr might be null because of wrapping)
                Some(ptr::read(1 as *mut T))
            } else {
                let old = self.ptr;
                self.ptr = self.ptr.add(1);

                Some(ptr::read(old))
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = match self.ptr.offset_to_(self.end) {
            Some(x) => x as usize,
            None => (self.end as usize).wrapping_sub(self.ptr as usize),
        };
        (exact, Some(exact))
    }

    #[inline]
    fn count(self) -> usize {
        self.size_hint().0
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        unsafe {
            if self.end == self.ptr {
                None
            } else if mem::size_of::<T>() == 0 {
                // See above for why 'ptr.offset' isn't used
                self.end = intrinsics::arith_offset(self.end as *const i8, -1) as *mut T;

                // Use a non-null pointer value
                // (self.end might be null because of wrapping)
                Some(ptr::read(1 as *mut T))
            } else {
                self.end = self.end.offset(-1);

                Some(ptr::read(self.end))
            }
        }
    }
}

impl<T: Clone> Clone for IntoIter<T> {
    #[inline]
    fn clone(&self) -> Self {
        let mut deq = SliceDeque::<T>::with_capacity(self.size_hint().0);
        unsafe {
            deq.append_elements(self.as_slice());
        }
        deq.into_iter()
    }
}

impl<T> Drop for IntoIter<T> {
    #[inline]
    fn drop(&mut self) {
        // destroy the remaining elements
        for _x in self.by_ref() {}

        // Buffer handles deallocation
        let _ = unsafe { Buffer::from_raw_parts(self.buf.as_ptr(), 2 * self.cap) };
    }
}

impl<T> IntoIterator for SliceDeque<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Creates a consuming iterator, that is, one that moves each value out of
    /// the deque (from start to end). The deque cannot be used after calling
    /// this.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate slice_deque;
    /// # use slice_deque::SliceDeque;
    /// # fn main() {
    /// let mut deq = sdeq!["a".to_string(), "b".to_string()];
    /// let expected = ["a".to_string(), "b".to_string()];
    /// for (i, s) in deq.into_iter().enumerate() {
    ///     // s has type String, not &String
    ///     println!("{}", s);
    ///     assert_eq!(s, expected[i]);
    /// }
    /// # }
    /// ```
    #[inline]
    fn into_iter(self) -> IntoIter<T> {
        unsafe {
            let buf_ptr = self.buf.ptr();
            intrinsics::assume(!buf_ptr.is_null());
            assert!(mem::size_of::<T>() != 0); // TODO: zero-sized types
            let begin = buf_ptr.add(self.head()) as *const T;
            let end = buf_ptr.add(self.tail()) as *const T;
            assert!(begin as usize <= end as usize);
            let it = IntoIter {
                buf: NonNull::new_unchecked(buf_ptr),
                cap: self.capacity(),
                ptr: begin,
                end,
            };
            debug_assert!(self.len() == it.size_hint().0);
            #[cfg_attr(feature = "cargo-clippy", allow(clippy::mem_forget))]
            mem::forget(self);
            it
        }
    }
}

impl<'a, T> IntoIterator for &'a SliceDeque<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;
    #[inline]
    fn into_iter(self) -> slice::Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SliceDeque<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;
    #[inline]
    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T> Extend<T> for SliceDeque<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        <Self as SpecExtend<T, I::IntoIter>>::spec_extend(self, iter.into_iter())
    }
}

/// Specialization trait used for `SliceDeque::from_iter` and
/// `SliceDeque::extend`.
trait SpecExtend<T, I> {
    /// Specialization for `SliceDeque::from_iter`.
    fn from_iter(iter: I) -> Self;
    /// Specialization for `SliceDeque::extend`.
    fn spec_extend(&mut self, iter: I);
}

/// Default implementation of `SpecExtend::from_iter`.
#[inline(always)]
fn from_iter_default<T, I: Iterator<Item = T>>(mut iterator: I) -> SliceDeque<T> {
    // Unroll the first iteration, as the deque is going to be
    // expanded on this iteration in every case when the iterable is not
    // empty, but the loop in extend_desugared() is not going to see the
    // deque being full in the few subsequent loop iterations.
    // So we get better branch prediction.
    let mut deque = match iterator.next() {
        None => return SliceDeque::<T>::new(),
        Some(element) => {
            let (lower, _) = iterator.size_hint();
            let mut deque = SliceDeque::<T>::with_capacity(lower.saturating_add(1));
            unsafe {
                ptr::write(deque.get_unchecked_mut(0), element);
                deque.move_tail_unchecked(1);
            }
            deque
        }
    };
    <SliceDeque<T> as SpecExtend<T, I>>::spec_extend(&mut deque, iterator);
    deque
}

impl<T, I> SpecExtend<T, I> for SliceDeque<T>
where
    I: Iterator<Item = T>,
{
    fn from_iter(iterator: I) -> Self {
        from_iter_default(iterator)
    }

    fn spec_extend(&mut self, iter: I) {
        self.extend_desugared(iter)
    }
}

impl<'a, T: 'a, I> SpecExtend<&'a T, I> for SliceDeque<T>
where
    I: Iterator<Item = &'a T>,
    T: Clone,
{
    fn from_iter(iterator: I) -> Self {
        SpecExtend::from_iter(iterator.cloned())
    }

    fn spec_extend(&mut self, iterator: I) {
        self.spec_extend(iterator.cloned())
    }
}

impl<T> iter::FromIterator<T> for SliceDeque<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        unsafe { <Self as SpecExtend<T, I::IntoIter>>::from_iter(iter.into_iter()) }
    }
}

/// This code generalises `extend_with_{element,default}`.
trait ExtendWith<T> {
    /// TODO: docs
    fn next(&self) -> T;
    /// TODO: docs
    fn last(self) -> T;
}

/// TODO: docs
struct ExtendElement<T>(T);
impl<T: Clone> ExtendWith<T> for ExtendElement<T> {
    fn next(&self) -> T {
        self.0.clone()
    }
    fn last(self) -> T {
        self.0
    }
}

/// TODO: docs
struct ExtendDefault;
impl<T: Default> ExtendWith<T> for ExtendDefault {
    fn next(&self) -> T {
        Default::default()
    }
    fn last(self) -> T {
        Default::default()
    }
}

/// TODO: docs
/// FIXME: not used, this should be used by the sdeq! macro? Remove this maybe.
#[doc(hidden)]
pub fn from_elem<T: Clone>(elem: T, n: usize) -> SliceDeque<T> {
    <T as SpecFromElem>::from_elem(elem, n)
}

/// Specialization trait used for `SliceDeque::from_elem`.
trait SpecFromElem: Sized {
    /// TODO: docs
    fn from_elem(elem: Self, n: usize) -> SliceDeque<Self>;
}

impl<T: Clone> SpecFromElem for T {
    fn from_elem(elem: Self, n: usize) -> SliceDeque<Self> {
        let mut v = SliceDeque::with_capacity(n);
        v.extend_with(n, ExtendElement(elem));
        v
    }
}

/// Extend implementation that copies elements out of references before
/// pushing them onto the `SliceDeque`.
///
/// This implementation is specialized for slice iterators, where it uses
/// [`copy_from_slice`] to append the entire slice at once.
///
/// [`copy_from_slice`]: ../../std/primitive.slice.html#method.copy_from_slice
impl<'a, T: 'a + Copy> Extend<&'a T> for SliceDeque<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.spec_extend(iter.into_iter())
    }
}

/// A splicing iterator for `SliceDeque`.
///
/// This struct is created by the [`splice()`] method on [`SliceDeque`]. See
/// its documentation for more.
///
/// [`splice()`]: struct.SliceDeque.html#method.splice
/// [`SliceDeque`]: struct.SliceDeque.html
#[derive(Debug)]
pub struct Splice<'a, I: Iterator + 'a> {
    /// TODO: docs
    drain: Drain<'a, I::Item>,
    /// TODO: docs
    replace_with: I,
}

impl<'a, I: Iterator> Iterator for Splice<'a, I> {
    type Item = I::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.drain.size_hint()
    }
}

impl<'a, I: Iterator> DoubleEndedIterator for Splice<'a, I> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.drain.next_back()
    }
}

impl<'a, I: Iterator> Drop for Splice<'a, I> {
    fn drop(&mut self) {
        // exhaust drain first
        while let Some(_) = self.drain.next() {}

        unsafe {
            if self.drain.tail_len == 0 {
                self.drain.deq.as_mut().extend(self.replace_with.by_ref());
                return;
            }

            // First fill the range left by drain().
            if !self.drain.fill(&mut self.replace_with) {
                return;
            }

            // There may be more elements. Use the lower bound as an estimate.
            // FIXME: Is the upper bound a better guess? Or something else?
            let (lower_bound, _upper_bound) = self.replace_with.size_hint();
            if lower_bound > 0 {
                self.drain.move_tail_unchecked(lower_bound);
                if !self.drain.fill(&mut self.replace_with) {
                    return;
                }
            }

            // Collect any remaining elements.
            // This is a zero-length deque which does not allocate if
            // `lower_bound` was exact.
            let mut collected = self
                .replace_with
                .by_ref()
                .collect::<SliceDeque<I::Item>>()
                .into_iter();
            // Now we have an exact count.
            if collected.size_hint().0 > 0 {
                self.drain.move_tail_unchecked(collected.size_hint().0);
                let filled = self.drain.fill(&mut collected);
                debug_assert!(filled);
                debug_assert_eq!(collected.size_hint().0, 0);
            }
        }
        // Let `Drain::drop` move the tail back if necessary and restore
        // `deq.tail`.
    }
}

/// Private helper methods for `Splice::drop`
impl<'a, T> Drain<'a, T> {
    /// The range from `self.deq.tail` to `self.tail()_start` contains elements
    /// that have been moved out.
    /// Fill that range as much as possible with new elements from the
    /// `replace_with` iterator. Return whether we filled the entire
    /// range. (`replace_with.next()` didn’t return `None`.)
    unsafe fn fill<I: Iterator<Item = T>>(&mut self, replace_with: &mut I) -> bool {
        let deq = unsafe { self.deq.as_mut() };
        let range_start = deq.len();
        let range_end = self.tail_start;
        let range_slice = unsafe {
            slice::from_raw_parts_mut(deq.as_mut_ptr().add(range_start), range_end - range_start)
        };

        for place in range_slice {
            if let Some(new_item) = replace_with.next() {
                unsafe { ptr::write(place, new_item) };
                unsafe { deq.move_tail_unchecked(1) };
            } else {
                return false;
            }
        }
        true
    }

    /// Make room for inserting more elements before the tail.
    unsafe fn move_tail_unchecked(&mut self, extra_capacity: usize) {
        let deq = unsafe { self.deq.as_mut() };
        let used_capacity = self.tail_start + self.tail_len;
        deq.reserve_capacity(used_capacity + extra_capacity)
            .expect("oom");

        let new_tail_start = self.tail_start + extra_capacity;
        let src = unsafe { deq.as_ptr().add(self.tail_start) };
        let dst = unsafe { deq.as_mut_ptr().add(new_tail_start) };
        unsafe { ptr::copy(src, dst, self.tail_len) };
        self.tail_start = new_tail_start;
    }
}

/// An iterator produced by calling `drain_filter` on `SliceDeque`.
#[derive(Debug)]
pub struct DrainFilter<'a, T: 'a, F>
where
    F: FnMut(&mut T) -> bool,
{
    /// TODO: docs
    deq: &'a mut SliceDeque<T>,
    /// TODO: docs
    idx: usize,
    /// TODO: docs
    del: usize,
    /// TODO: docs
    old_len: usize,
    /// TODO: docs
    pred: F,
}

impl<'a, T, F> Iterator for DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        unsafe {
            while self.idx != self.old_len {
                let i = self.idx;
                self.idx += 1;
                let v = slice::from_raw_parts_mut(self.deq.as_mut_ptr(), self.old_len);
                if (self.pred)(&mut v[i]) {
                    self.del += 1;
                    return Some(ptr::read(&v[i]));
                } else if self.del > 0 {
                    let del = self.del;
                    let src: *const T = &v[i];
                    let dst: *mut T = &mut v[i - del];
                    // This is safe because self.deq has length 0
                    // thus its elements will not have Drop::drop
                    // called on them in the event of a panic.
                    ptr::copy_nonoverlapping(src, dst, 1);
                }
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.old_len - self.idx))
    }
}

impl<'a, T, F> Drop for DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    fn drop(&mut self) {
        for _ in self.by_ref() {}

        unsafe {
            let new_len = self.old_len - self.del;
            let new_tail = self.deq.head() + new_len;
            let old_tail = self.deq.tail();
            self.deq
                .move_tail_unchecked(new_tail as isize - old_tail as isize);
        }
    }
}

impl<T> convert::AsRef<[T]> for SliceDeque<T> {
    fn as_ref(&self) -> &[T] {
        &*self
    }
}

impl<T> convert::AsMut<[T]> for SliceDeque<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut *self
    }
}

fn main() {}
