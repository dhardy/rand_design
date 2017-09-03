//! Independent traits
//! 
//! This uses a wrapper, `AsRng`, implementing `Rng` for any `CryptoRng`.
//! (This wrapper can be used with other variants.)
//! 
//! Note: this *only* considers the next_u32 member function
//! 
//! Thoughts: works fine; relatively simple surprise-free code.
//! Adaptors needed in both directions. This (and `raw_rng` which also needs
//! two adaptors) is the only version which doesn't have implicit impls which
//! may panic.

// ——— traits ———

#[derive(Debug)]
struct CryptoError;

trait CryptoRng {
    fn try_next_u32(&mut self) -> Result<u32, CryptoError>;
}

trait Rng {
    fn next_u32(&mut self) -> u32;
}

// ——— impl ———

// Required for `as_rng(&mut rng)`
impl<'a, CR: CryptoRng+?Sized> CryptoRng for &'a mut CR {
    fn try_next_u32(&mut self) -> Result<u32, CryptoError> {
        (*self).try_next_u32()
    }
}

// Required for `as_crng(&mut rng)`
impl<'a, R: Rng+?Sized> Rng for &'a mut R {
    fn next_u32(&mut self) -> u32 {
        (*self).next_u32()
    }
}

// ——— adaptor 1 ———

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

// ——— adaptor 2 ———

// Given `rng` of type `T` where `T: Rng`, this can consume
// `rng` (`as_rng(rng)`)
fn as_crng<R: Rng>(rng: R) -> AsCRng<R> {
    AsCRng { rng }
}

struct AsCRng<R: Rng+?Sized> {
    rng: R
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

impl CryptoRng for TestCRng {
    fn try_next_u32(&mut self) -> Result<u32, CryptoError> {
        Ok(self.0)
    }
}

// A dual-purpose Rng
#[derive(Debug)]
struct TestBothRng(u32);

impl Rng for TestBothRng {
    fn next_u32(&mut self) -> u32 {
        self.0
    }
}

impl CryptoRng for TestBothRng {
    fn try_next_u32(&mut self) -> Result<u32, CryptoError> {
        Ok(self.0)
    }
}


// ——— usage ———

fn main() {
    let mut t = TestRng(13);
    let mut c = TestCRng(42);
    let mut b = TestBothRng(3651);
    println!("t: {:?} impls Rng", t);
    println!("c: {:?} impls CryptoRng", c);
    println!("b: {:?} impls both", b);
    {
        // Do both traits support both functions via static dispatch?
        println!("t, static dispatch, using CryptoRng: {:?}", as_crng(&mut t).try_next_u32());
        println!("t, static dispatch, using Rng: {:?}", t.next_u32());
        println!("c, static dispatch, using CryptoRng: {:?}", c.try_next_u32());
        println!("c, static dispatch, using Rng: {:?}", as_rng(&mut c).next_u32());
        println!("b, static dispatch, using CryptoRng: {:?}", b.try_next_u32());
        println!("b, static dispatch, using Rng: {:?}", b.next_u32());
    }
    {
        // Can both types be used via CryptoRng with dynamic dispatch?
        let cr = &mut c as &mut CryptoRng;
        println!("c, dynamic dispatch, using CryptoRng: {:?}", cr.try_next_u32());
        let mut tr = as_crng(&mut t as &mut Rng);
        println!("t, dynamic dispatch, using CryptoRng: {:?}", tr.try_next_u32());
        let br = &mut b as &mut CryptoRng;
        println!("b, dynamic dispatch, using CryptoRng: {:?}", br.try_next_u32());
    }
    {
        // Can both types be used via Rng with dynamic dispatch?
        let mut cr = as_rng(&mut c as &mut CryptoRng);
        let tr = &mut t as &mut Rng;
        let br = &mut b as &mut Rng;
        println!("c, dynamic dispatch, using Rng: {:?}", cr.next_u32());
        println!("t, dynamic dispatch, using Rng: {:?}", tr.next_u32());
        println!("b, dynamic dispatch, using Rng: {:?}", br.next_u32());
    }
}
