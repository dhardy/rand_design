//! Tries to unify the two traits via templating on the error type and using
//! the never type (still experimental). Tests have shown no performance
//! overhead.
//! 
//! Rng exists as a separate trait only so that users don't have to unwrap
//! the `Result<T, !>` type themselves.
//! 
//! Note: this *only* considers the next_u32 member function.
//! 
//! Thoughts: a common super-trait which is not object safe doesn't really help
//! anything(?). At the same time, it's no longer possible to make one version
//! implement the other, so IMO this is strictly worse than extends_CryptoRng2.
//! And don't forget, this also depends on an unstable language feature.

#![feature(never_type)]

// ——— traits ———

trait RawRng<Error> {
    fn try_next_u32(&mut self) -> Result<u32, Error>;
}

trait Rng: RawRng<!> {
    fn next_u32(&mut self) -> u32 {
        self.try_next_u32().unwrap_or_else(|e| e)
    }
}

#[derive(Debug)]
struct CryptoError;

type CryptoRng = RawRng<CryptoError>;

// ——— impls ———

impl<R: Rng+?Sized> RawRng<!> for R {
    fn try_next_u32(&mut self) -> Result<u32, !> {
        Ok(self.next_u32())
    }
}

// Required for `as_rng(&mut rng)` and `as_rng_ref` definition.
impl<'a, CR: RawRng<CryptoError>+?Sized> RawRng<CryptoError> for &'a mut CR {
    fn try_next_u32(&mut self) -> Result<u32, CryptoError> {
        (*self).try_next_u32()
    }
}

// ——— adaptor ———

// Given `rng` of type `T` where `T: CryptoRng`, this can consume
// `rng` (`as_rng(rng)`)
fn as_rng<CR: RawRng<CryptoError>>(rng: CR) -> AsRng<CR> {
    AsRng { rng }
}

struct AsRng<CR: RawRng<CryptoError>+?Sized> {
    rng: CR
}

impl<CR: RawRng<CryptoError>+?Sized> Rng for AsRng<CR> {
    fn next_u32(&mut self) -> u32 {
        self.rng.try_next_u32().unwrap()
    }
}

// ——— test RNGs ———

// A non-crypto Rng
#[derive(Debug)]
struct TestRng(u32);

impl Rng for TestRng {
    fn next_u32(&mut self) -> u32 {
        self.0
    }
}

// A CryptoRng
#[derive(Debug)]
struct TestCRng(u32);

impl RawRng<CryptoError> for TestCRng {
    fn try_next_u32(&mut self) -> Result<u32, CryptoError> {
        Ok(self.0)
    }
}

// ——— usage ———

fn main() {
    let mut t = TestRng(13);
    let mut c = TestCRng(42);
    println!("t: {:?} impls Rng", t);
    println!("c: {:?} impls CryptoRng", c);
    {
        // Do both traits support both functions via static dispatch?
        println!("t, static dispatch, using CryptoRng: {:?}", t.try_next_u32());
        println!("t, static dispatch, using Rng: {:?}", t.next_u32());
        println!("c, static dispatch, using CryptoRng: {:?}", c.try_next_u32());
        println!("c, static dispatch, using Rng: {:?}", as_rng(&mut c).next_u32());
    }
    {
        // Can both types be used via CryptoRng with dynamic dispatch?
        let cr = &mut c as &mut CryptoRng;
        println!("c, dynamic dispatch, using CryptoRng: {:?}", cr.try_next_u32());
        /* TODO: this would also need an adaptor, and it would be problematic
        (impl RawRng<!> for &mut RawRng<!> would conflict).
        let tr = &mut t as &mut CryptoRng;
        println!("t, dynamic dispatch, using CryptoRng: {:?}", tr.try_next_u32());
        */
    }
    {
        // Can both types be used via Rng with dynamic dispatch?
        let mut cr = as_rng(&mut c as &mut CryptoRng);
        let tr = &mut t as &mut Rng;
        println!("c, dynamic dispatch, using Rng: {:?}", cr.next_u32());
        println!("t, dynamic dispatch, using Rng: {:?}", tr.next_u32());
    }
}
