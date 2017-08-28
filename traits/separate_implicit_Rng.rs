//! Independent traits
//! 
//! This implements `Rng` for any `CryptoRng` implicitly.
//! 
//! Note: this *only* considers the next_u32 member function
//! 
//! Thoughts: this is basically equivalent to extends_CryptoRng2.

// ——— traits ———

#[derive(Debug)]
struct CryptoError;

trait CryptoRng {
    fn try_next_u32(&mut self) -> Result<u32, CryptoError>;
}

trait Rng {
    fn next_u32(&mut self) -> u32;
}

// ——— impls ———

impl<R: Rng+?Sized> CryptoRng for R {
    fn try_next_u32(&mut self) -> Result<u32, CryptoError> {
        Ok(self.next_u32())
    }
}

// Required for `as_rng(&mut rng)` and `as_rng_ref` definition.
impl<'a, CR: CryptoRng+?Sized> Rng for &'a mut CR {
    fn next_u32(&mut self) -> u32 {
        (*self).try_next_u32().unwrap()
    }
}

// ——— adaptor ———

// Given `rng` of type `T` where `T: CryptoRng`, this can consume
// `rng` (`as_rng(rng)`) or use a reference (`as_rng(&mut rng)`).
fn as_rng<CR: CryptoRng>(rng: CR) -> AsRng<CR> {
    AsRng { rng }
}

struct AsRng<CR: CryptoRng+?Sized> {
    rng: CR
}

impl<CR: CryptoRng+?Sized> Rng for AsRng<CR> {
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

impl CryptoRng for TestCRng {
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
        println!("c, static dispatch, using Rng: {:?}", (&mut c).next_u32());
    }
    {
        // Can both types be used via CryptoRng with dynamic dispatch?
        let cr = &mut c as &mut CryptoRng;
        println!("c, dynamic dispatch, using CryptoRng: {:?}", cr.try_next_u32());
        let tr = &mut t as &mut CryptoRng;
        println!("t, dynamic dispatch, using CryptoRng: {:?}", tr.try_next_u32());
    }
    {
        // Can both types be used via Rng with dynamic dispatch?
        let mut cr = as_rng(&mut c as &mut CryptoRng);
        let tr = &mut t as &mut Rng;
        println!("c, dynamic dispatch, using Rng: {:?}", cr.next_u32());
        println!("t, dynamic dispatch, using Rng: {:?}", tr.next_u32());
    }
}
