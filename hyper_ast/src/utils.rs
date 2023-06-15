use core::fmt;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{BuildHasher, Hash, Hasher},
};

pub fn hash<T: ?Sized + Hash>(x: &T) -> u64 {
    let mut state = DefaultHasher::default();
    x.hash(&mut state);
    state.finish()
}

/// Creates the `u64` hash value for the given value using the given hash builder.
pub fn make_hash<T>(builder: &impl BuildHasher, value: &T) -> u64
where
    T: ?Sized + Hash,
{
    let state = &mut builder.build_hasher();
    value.hash(state);
    state.finish()
}

pub fn clamp_u64_to_u32(x: &u64) -> u32 {
    (((x & 0xffff0000) >> 32) as u32) ^ ((x & 0xffff) as u32)
}

#[derive(Clone)]
pub struct MemoryUsage {
    allocated: Bytes,
}

impl fmt::Display for MemoryUsage {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.allocated)
    }
}

impl Into<Bytes> for MemoryUsage {
    fn into(self) -> Bytes {
        self.allocated
    }
}
impl Into<isize> for MemoryUsage {
    fn into(self) -> isize {
        self.allocated.bytes()
    }
}
impl Into<isize> for &MemoryUsage {
    fn into(self) -> isize {
        self.allocated.bytes()
    }
}

impl std::ops::Sub for MemoryUsage {
    type Output = MemoryUsage;
    fn sub(self, rhs: MemoryUsage) -> MemoryUsage {
        MemoryUsage {
            allocated: self.allocated - rhs.allocated,
        }
    }
}

impl std::ops::Add for MemoryUsage {
    type Output = MemoryUsage;
    fn add(self, rhs: MemoryUsage) -> MemoryUsage {
        MemoryUsage {
            allocated: self.allocated + rhs.allocated,
        }
    }
}

#[cfg(all(target_os = "linux", target_env = "gnu", not(feature = "jemalloc")))]
pub fn memusage_linux() -> MemoryUsage {
    // todo!()
    // // Linux/glibc has 2 APIs for allocator introspection that we can use: mallinfo and mallinfo2.
    // // mallinfo uses `int` fields and cannot handle memory usage exceeding 2 GB.
    // // mallinfo2 is very recent, so its presence needs to be detected at runtime.
    // // Both are abysmally slow.

    // use std::ffi::CStr;
    // use std::sync::atomic::{AtomicUsize, Ordering};

    // static MALLINFO2: AtomicUsize = AtomicUsize::new(1);

    // let mut mallinfo2 = MALLINFO2.load(Ordering::Relaxed);
    // if mallinfo2 == 1 {
    //     let cstr = CStr::from_bytes_with_nul(b"mallinfo2\0").unwrap();
    //     mallinfo2 = unsafe { libc::dlsym(libc::RTLD_DEFAULT, cstr.as_ptr()) } as usize;
    //     // NB: races don't matter here, since they'll always store the same value
    //     MALLINFO2.store(mallinfo2, Ordering::Relaxed);
    // }

    // if mallinfo2 == 0 {
    //     // mallinfo2 does not exist, use mallinfo.
    //     let alloc = unsafe { libc::mallinfo() }.uordblks as isize;
    //     MemoryUsage {
    //         allocated: Bytes(alloc),
    //     }
    // } else {
    //     let mallinfo2: fn() -> libc::mallinfo2 = unsafe { std::mem::transmute(mallinfo2) };
    //     let alloc = mallinfo2().uordblks as isize;
    //     MemoryUsage {
    //         allocated: Bytes(alloc),
    //     }
    // }
    log::debug!("no way of measuring heap precisely");
    let allocated = 0;
    MemoryUsage {
        allocated: Bytes(allocated as isize),
    }
}

#[cfg(all(target_os = "linux", target_env = "gnu", feature = "jemalloc"))]
pub fn memusage_linux() -> MemoryUsage {
    // #[cfg(not(target_env = "msvc"))]
    use jemalloc_ctl::{epoch, stats};
    // Obtain a MIB for the `epoch`, `stats.allocated`, and
    // `atats.resident` keys:
    let e = epoch::mib().unwrap();
    let allocated = stats::allocated::mib().unwrap();
    // let resident = stats::resident::mib().unwrap();
    e.advance().unwrap();

    // Read statistics using MIB key:
    let allocated = allocated.read().unwrap();
    // let allocated = 0;
    // let resident = resident.read().unwrap();
    // println!("{} bytes allocated/{} bytes resident", allocated, resident);
    MemoryUsage {
        allocated: Bytes(allocated as isize),
    }
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Bytes(isize);

impl Bytes {
    pub fn megabytes(self) -> isize {
        self.0 / 1024 / 1024
    }
    pub fn bytes(self) -> isize {
        self.0
    }
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.0;
        let mut value = bytes;
        let mut suffix = "b";
        if value.abs() > 4096 {
            value /= 1024;
            suffix = "kb";
            if value.abs() > 4096 {
                value /= 1024;
                suffix = "mb";
            }
        }
        f.pad(&format!("{}{}", value, suffix))
    }
}

impl Into<isize> for &Bytes {
    fn into(self) -> isize {
        self.0
    }
}

impl std::ops::AddAssign<usize> for Bytes {
    fn add_assign(&mut self, x: usize) {
        self.0 += x as isize;
    }
}

impl std::ops::AddAssign<Bytes> for Bytes {
    fn add_assign(&mut self, x: Bytes) {
        self.0 += x.0;
    }
}

impl std::ops::Sub for Bytes {
    type Output = Bytes;
    fn sub(self, rhs: Bytes) -> Bytes {
        Bytes(self.0 - rhs.0)
    }
}

impl std::ops::Add for Bytes {
    type Output = Bytes;
    fn add(self, rhs: Bytes) -> Bytes {
        Bytes(self.0 + rhs.0)
    }
}
