//! Tries to unify the two traits via templating on the error type and using
//! the never type (still experimental). Tests have shown no performance
//! overhead.
//! 
//! Rng exists as a separate trait only so that users don't have to unwrap
//! the `Result<T, !>` type themselves.
//! 
//! Note: this *only* considers the next_u32 member function.
//! 
//! Thoughts: a bit messy. This only considers the two cases covered by other
//! examples: infallible non-crypto RNG and fallible crypto RNG, but it should
//! also consider at least CryptoRng<!>.
#![feature(never_type)]

use std::marker::PhantomData;
use std::fmt::Debug;

// ——— traits ———

#[derive(Debug)]
struct Error;

trait RawRng<E> {
    fn try_next_u32(&mut self) -> Result<u32, E>;
}

trait Rng: RawRng<!> {
    fn next_u32(&mut self) -> u32 {
        self.try_next_u32().unwrap_or_else(|e| e)
    }
}

trait CryptoRng<E>: RawRng<E> {}

// ——— impls ———

impl<R: Rng+?Sized> RawRng<!> for R {
    fn try_next_u32(&mut self) -> Result<u32, !> {
        Ok(self.next_u32())
    }
}

// This impl allows infallible generators to use the fallible trait, but means
// `t.try_next_u32()` resolves to two functions (conflict)
impl<R: RawRng<!>+?Sized> RawRng<Error> for R {
    fn try_next_u32(&mut self) -> Result<u32, Error> {
        self.try_next_u32().map_err(|e| e)
    }
}

// ——— adaptor ———

// Given `rng` of type `T` where `T: CryptoRng`, this can consume
// `&mut rng` (`as_rng(rng)`)
fn as_rng_ref<'a, E: Debug, CR: RawRng<E>+?Sized+'a>(rng: &'a mut CR) -> AsRng<'a, E, CR> {
    AsRng { _phantom: PhantomData {}, rng }
}

struct AsRng<'a, E: Debug, CR: RawRng<E>+?Sized+'a> {
    _phantom: PhantomData<E>,
    rng: &'a mut CR
}

impl<'a, E: Debug, CR: RawRng<E>+?Sized> Rng for AsRng<'a, E, CR> {
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

impl RawRng<Error> for TestCRng {
    fn try_next_u32(&mut self) -> Result<u32, Error> {
        Ok(self.0)
    }
}

impl CryptoRng<Error> for TestCRng {}

// ——— usage ———

fn main() {
    let mut t = TestRng(13);
    let mut c = TestCRng(42);
    println!("t: {:?} impls Rng", t);
    println!("c: {:?} impls CryptoRng", c);
    {
        // Do both traits support both functions via static dispatch?
        println!("t, static dispatch, using RawRng<Error>: {:?}", RawRng::<Error>::try_next_u32(&mut t));
        println!("t, static dispatch, using Rng: {:?}", t.next_u32());
        println!("c, static dispatch, using RawRng<Error>: {:?}", c.try_next_u32());
        println!("c, static dispatch, using Rng: {:?}", as_rng_ref(&mut c).next_u32());
    }
    {
        // Can both types be used via RawRng<Error> with dynamic dispatch?
        let cr = &mut c as &mut RawRng<Error>;
        println!("c, dynamic dispatch, using RawRng<Error>: {:?}", cr.try_next_u32());
        let tr = &mut t as &mut RawRng<Error>;
        println!("t, dynamic dispatch, using RawRng<Error>: {:?}", tr.try_next_u32());
    }
    {
        // Can both types be used via RawRng<!> with dynamic dispatch?
        let mut cr = as_rng_ref(&mut c as &mut CryptoRng<Error>);
        let tr = &mut t as &mut Rng;
        println!("c, dynamic dispatch, using Rng: {:?}", cr.next_u32());
        println!("t, dynamic dispatch, using Rng: {:?}", tr.next_u32());
    }
}
