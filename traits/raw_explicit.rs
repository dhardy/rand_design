//! Tries to unify the two traits via templating on the error type and using
//! the never type (still experimental). Tests have shown no performance
//! overhead.
//! 
//! Rng exists as a separate trait only so that users don't have to unwrap
//! the `Result<T, !>` type themselves.
//! 
//! Note: this *only* considers the next_u32 member function.
//! 
//! Thoughts: better than I had expected. A little complex. Might be workable.
#![feature(never_type)]

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

impl<R: CryptoRng<!>+?Sized> CryptoRng<Error> for R {}

// ——— adaptor ———

// Given `rng` of type `T` where `T: CryptoRng`, this can consume
// `&mut rng` (`as_rng(rng)`)
fn as_rng_ref<'a, CR: RawRng<Error>+?Sized+'a>(rng: &'a mut CR) -> AsRng<'a, CR> {
    AsRng { rng }
}

struct AsRng<'a, CR: RawRng<Error>+?Sized+'a> {
    rng: &'a mut CR
}

impl<'a, CR: RawRng<Error>+?Sized> Rng for AsRng<'a, CR> {
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

// An infallible CryptoRng
#[derive(Debug)]
struct TestICRng(u32);

impl Rng for TestICRng {
    fn next_u32(&mut self) -> u32 {
        self.0
    }
}

impl CryptoRng<!> for TestICRng {}

// A fallible CryptoRng
#[derive(Debug)]
struct TestFCRng(u32);

impl RawRng<Error> for TestFCRng {
    fn try_next_u32(&mut self) -> Result<u32, Error> {
        Ok(self.0)
    }
}

impl CryptoRng<Error> for TestFCRng {}

// ——— usage ———

fn main() {
    let mut t = TestRng(13);
    let mut ic = TestICRng(17);
    let mut fc = TestFCRng(42);
    println!("t: {:?} impls Rng", t);
    println!("ic: {:?} impls CryptoRng", ic);
    println!("fc: {:?} impls CryptoRng", fc);
    {
        // Do all traits support all functions via static dispatch?
        println!("t, static dispatch, using RawRng<Error>: {:?}", RawRng::<Error>::try_next_u32(&mut t));
        println!("t, static dispatch, using Rng: {:?}", t.next_u32());
        println!("ic, static dispatch, using RawRng<Error>: {:?}", RawRng::<Error>::try_next_u32(&mut ic));
        println!("ic, static dispatch, using Rng: {:?}", as_rng_ref(&mut ic).next_u32());
        println!("fc, static dispatch, using RawRng<Error>: {:?}", fc.try_next_u32());
        println!("fc, static dispatch, using Rng: {:?}", as_rng_ref(&mut fc).next_u32());
    }
    {
        // Can all types be used via RawRng<Error> with dynamic dispatch?
        let ir = &mut ic as &mut RawRng<Error>;
        println!("ic, dynamic dispatch, using RawRng<Error>: {:?}", ir.try_next_u32());
        let cr = &mut fc as &mut RawRng<Error>;
        println!("fc, dynamic dispatch, using RawRng<Error>: {:?}", cr.try_next_u32());
        let tr = &mut t as &mut RawRng<Error>;
        println!("t, dynamic dispatch, using RawRng<Error>: {:?}", tr.try_next_u32());
    }
    {
        // Can all types be used via RawRng<!> with dynamic dispatch?
        let ir = &mut ic as &mut Rng;
        let mut cr = as_rng_ref(&mut fc as &mut CryptoRng<Error>);
        let tr = &mut t as &mut Rng;
        println!("ic, dynamic dispatch, using Rng: {:?}", ir.next_u32());
        println!("fc, dynamic dispatch, using Rng: {:?}", cr.next_u32());
        println!("t, dynamic dispatch, using Rng: {:?}", tr.next_u32());
    }
    {
        // Can both crypto RNGs be used via CryptoRng<Error> with dynamic dispatch?
        let ir = &mut ic as &mut CryptoRng<Error>;
        let fr = &mut fc as &mut CryptoRng<Error>;
        println!("ic, dynamic dispatch, using CryptoRng: {:?}", ir.try_next_u32());
        println!("fc, dynamic dispatch, using CryptoRng: {:?}", fr.try_next_u32());
        
    }
}
