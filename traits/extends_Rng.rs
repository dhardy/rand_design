//! Independent traits
//! 
//! This implements `Rng` for any `CryptoRng` implicitly.
//! 
//! Note: this *only* considers the next_u32 member function
//! 
//! Thoughts: we probably don't want CryptoRng to depend on Rng. CryptoRng
//! trait requires two implementations, one of which may have to panic.
//! None of the "lib code" can panic, but this is moot due to above (user forced
//! to write code which may panic).

// ——— traits ———

#[derive(Debug)]
struct CryptoError;

trait Rng {
    fn next_u32(&mut self) -> u32;
}

trait CryptoRng: Rng {
    fn try_next_u32(&mut self) -> Result<u32, CryptoError> {
        Ok(self.next_u32())
    }
}

// ——— impls ———

impl<'a, R: Rng+?Sized> Rng for &'a mut R {
    fn next_u32(&mut self) -> u32 {
        (*self).next_u32()
    }
}

// ——— adaptor ———

// Given `rng` of type `T` where `T: Rng`, this can consume
// `rng` (`as_rng(rng)`)
fn as_crng<R: Rng>(rng: R) -> AsCRng<R> {
    AsCRng { rng }
}

struct AsCRng<R: Rng+?Sized> {
    rng: R
}

impl<R: Rng+?Sized> Rng for AsCRng<R> {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }
}

impl<R: Rng+?Sized> CryptoRng for AsCRng<R> {
    fn try_next_u32(&mut self) -> Result<u32, CryptoError> {
        Ok(self.rng.next_u32())
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

impl Rng for TestCRng {
    fn next_u32(&mut self) -> u32 {
        self.0
    }
}

impl CryptoRng for TestCRng {
    // No function defintion needed *if* next_u32 version is sufficient
}

// ——— usage ———

fn main() {
    let mut t = TestRng(13);
    let mut c = TestCRng(42);
    println!("t: {:?} impls Rng", t);
    println!("c: {:?} impls CryptoRng", c);
    {
        // Do both traits support both functions via static dispatch?
        println!("t, static dispatch, using CryptoRng: {:?}", as_crng(&mut t).try_next_u32());
        println!("t, static dispatch, using Rng: {:?}", t.next_u32());
        println!("c, static dispatch, using CryptoRng: {:?}", c.try_next_u32());
        println!("c, static dispatch, using Rng: {:?}", c.next_u32());
    }
    {
        // Can both types be used via CryptoRng with dynamic dispatch?
        let cr = &mut c as &mut CryptoRng;
        println!("c, dynamic dispatch, using CryptoRng: {:?}", cr.try_next_u32());
        let mut tr = as_crng(&mut t as &mut Rng);
        println!("t, dynamic dispatch, using CryptoRng: {:?}", tr.try_next_u32());
    }
    {
        // Can both types be used via Rng with dynamic dispatch?
        let cr = &mut c as &mut Rng;
        let tr = &mut t as &mut Rng;
        println!("c, dynamic dispatch, using Rng: {:?}", cr.next_u32());
        println!("t, dynamic dispatch, using Rng: {:?}", tr.next_u32());
    }
}
