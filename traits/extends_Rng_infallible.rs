//! Independent traits
//! 
//! This implements `Rng` for any `CryptoRng` implicitly.
//! 
//! Note: this *only* considers the next_u32 member function
//! 
//! Thoughts: works okay. Implementing each trait requires only one impl block.
//! It's not possible to impl either trait with no function definition despite
//! the presence of an impl rule for both traits. The
//! `impl Rng for CryptoRng` rule is an implicit conversion which may panic.
//! (A `impl CryptoRng for &mut Rng` rule is also required, as in extends_CryptoRng2.)

// ——— traits ———

trait Rng {
    fn next_u32(&mut self) -> u32;
}

trait CryptoRng: Rng {}

// ——— impls ———

impl<'a, R: Rng+?Sized> Rng for &'a mut R {
    fn next_u32(&mut self) -> u32 {
        (*self).next_u32()
    }
}

impl<'a, R: CryptoRng+?Sized> CryptoRng for &'a mut R {}

// ——— adaptor ———
// Note: we *probably* don't need this, since the *only* reason to use
// `CryptoRng` is if we *need* a cryptographic generator. (Most of the other
// variations require the use of `CryptoRng` for users who want to handle errors.)

// Given `rng` of type `T` where `T: Rng`, this can consume
// `rng` (`as_rng(rng)`), and claim that the generator is good enough for crypto
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
impl<R: Rng+?Sized> CryptoRng for AsCRng<R> {}

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
impl CryptoRng for TestCRng {}

// ——— usage ———

fn main() {
    let mut t = TestRng(13);
    let mut c = TestCRng(42);
    println!("t: {:?} impls Rng", t);
    println!("c: {:?} impls CryptoRng", c);
    {
        // Do both traits support both functions via static dispatch?
        println!("t, static dispatch, using Rng: {:?}", t.next_u32());
        println!("c, static dispatch, using CryptoRng: {:?}", c.next_u32());
        println!("c, static dispatch, using Rng: {:?}", c.next_u32());
    }
    {
        // Can both types be used via CryptoRng with dynamic dispatch?
        let cr = &mut c as &mut CryptoRng;
        println!("c, dynamic dispatch, using CryptoRng: {:?}", cr.next_u32());
        let mut tr = as_crng(&mut t as &mut Rng);
        println!("t, dynamic dispatch, using CryptoRng: {:?}", tr.next_u32());
    }
    {
        // Can both types be used via Rng with dynamic dispatch?
        let cr = &mut c as &mut Rng;
        let tr = &mut t as &mut Rng;
        println!("c, dynamic dispatch, using Rng: {:?}", cr.next_u32());
        println!("t, dynamic dispatch, using Rng: {:?}", tr.next_u32());
    }
}
