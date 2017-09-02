use std::cmp::min;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping;

#[derive(Debug)]
pub struct Error;

/// Trait governing random number generation.
/// 
/// Generators may be infallible (never failing) or fallible. In the latter
/// case, only `try_fill` allows error handling; other functions may panic.
pub trait Rng {
    /// Fill dest with random bytes.
    /// 
    /// Panics if the underlying generator has an error; use `try_fill` if you
    /// wish to handle errors.
    fn fill(&mut self, dest: &mut [u8]);
    
    /// Fill dest with random bytes.
    /// 
    /// In infallible generators this is identical to `fill`; in fallible
    /// generators this function may return an `Error`, and `fill` simply
    /// unwraps the `Result`.
    fn try_fill(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        // we asume infallable generator in default impl
        self.fill(dest);
        Ok(())
    }
    
    /// Generate a random number.
    /// 
    /// Panics if the underlying generator has an error.
    fn next_u64(&mut self) -> u64;
    
    // also next_u32, next_u128
}

/// Extension trait marking a generator as "cryptographically secure".
/// 
/// This exists so that code taking a generator as a parameter and requiring
/// *cryptographically-secure random numbers* may mark this requirement and
/// make use of type safety to enforce it.
/// 
/// Generators which should be secure when correctly initialised should
/// implement this trait (it is noted that no algorithmic generator can be
/// secure without a secret seed, and such seeding cannot be checked at
/// compile time, therefore this trait is more of a guideline than a guarantee).
pub trait CryptoRng: Rng {}

// ——— utility functions ———

/// Convenient implementation for `fill` in terms of `next_u64`.
// TODO: Also for u32, u128 via macro internals.
pub fn impl_fill_from_u64<R: Rng+?Sized>(rng: &mut R, dest: &mut [u8]) {
    let mut p: *mut u8 = &mut dest[0] as *mut u8;
    let mut len = dest.len();
    while len > 0 {
        let x = rng.next_u64().to_le();
        let xp = &x as *const u64 as *const u8;
        let n = min(len, size_of::<u64>());
        unsafe{ copy_nonoverlapping(xp, p, n); }
        unsafe{ p = p.offset(n as isize); }
        len -= n;
    }
}

macro_rules! impl_uint_from_fill {
    ($ty:ty, $N:expr, $rng:expr) => ({
        assert_eq!($N, size_of::<$ty>());
        let mut buf = [0u8; $N];
        $rng.fill(&mut buf);
        unsafe{ *(&buf[0] as *const u8 as *const $ty) }.to_le()
    });
}

/// Convenient implementation of `next_u64` in terms of `fill`.
/// 
/// High-performance generators will probably need to implement some `next_*`
/// variant directly and others in terms of that. But this provides a convenient
/// solution for other generators focussed mainly on byte streams.
pub fn impl_next_u64_from_fill<R: Rng+?Sized>(rng: &mut R) -> u64 {
    impl_uint_from_fill!(u64, 8, rng)
}

// ——— test RNGs ———

// A non-crypto Rng
#[derive(Debug)]
struct TestRng(u64);

impl Rng for TestRng {
    fn fill(&mut self, dest: &mut [u8]) {
        impl_fill_from_u64(self, dest)
    }
    
    fn next_u64(&mut self) -> u64 {
        self.0
    }
}

// A CryptoRng
#[derive(Debug)]
struct TestCRng(u64);

impl Rng for TestCRng {
    fn fill(&mut self, dest: &mut [u8]) {
        impl_fill_from_u64(self, dest)
    }
    
    fn next_u64(&mut self) -> u64 {
        self.0
    }
}

impl CryptoRng for TestCRng {}

// ——— usage ———

fn main() {
    let mut t = TestRng(0x20216F6C6C6548);
    let mut c = TestCRng(0x3F556572416F6857);
    let mut buf = [0u8; 16];
    t.fill(&mut buf);
    println!("t: {:?} says: {}", t, String::from_utf8_lossy(&buf));
    c.fill(&mut buf);
    println!("c: {:?} says: {}", c, String::from_utf8_lossy(&buf));
    {
        // Static dispatch
        println!("t, static dispatch, using Rng: {:?}", t.next_u64());
        println!("c, static dispatch, using Rng: {:?}", c.next_u64());
    }
    {
        // Dynamic dispatch using CryptoRng
        let cr = &mut c as &mut CryptoRng;
        println!("c, dynamic dispatch, using CryptoRng: {:?}", cr.next_u64());
    }
    {
        // Dynamic dispatch using Rng
        let tr = &mut t as &mut Rng;
        println!("t, dynamic dispatch, using Rng: {:?}", tr.next_u64());
        let cr = &mut c as &mut Rng;
        println!("c, dynamic dispatch, using Rng: {:?}", cr.next_u64());
    }
}
