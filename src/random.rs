//! this file controls the random number generator used
//!
//! By default the `csprng` feature is enabled, which enables
//! the ChaCha8 CSPRNG. It's not the most secure CSPRNG, but it's
//! mostly fast and we don't use it for cryptography.
//!
//! If the csprng feature is not enabled, we use the xoshiro PRNG.
//! It's faster in theory, but not appreciably in our measurements.

use std::{cell::UnsafeCell, rc::Rc};

// ChaCha8
//type Core = rand_chacha::ChaCha8Core;

/// BELOW WAS PARTIALLY COPIED FROM rand
// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "csprng")]
mod blockrngbased {
    use super::*;
    use rand::rngs::{adapter::ReseedingRng, OsRng};
    use rand::SeedableRng;

    // source from rand crate:
    // > Number of generated bytes after which to reseed `ThreadRng`.
    // > According to benchmarks, reseeding has a noticable impact with thresholds
    // > of 32 kB and less. We choose 64 kB to avoid significant overhead.
    // LPN crate: increase a lot: *1000
    const THREAD_RNG_RESEED_THRESHOLD: u64 = 1024 * 64 * 1000;

    type Core = rand_chacha::ChaCha8Core;
    thread_local!(
        // We require Rc<..> to avoid premature freeing when thread_rng is used
        // within thread-local destructors. See #968.
        static THREAD_RNG_KEY: Rc<UnsafeCell<ReseedingRng<Core, OsRng>>> = {
            let r = Core::from_rng(OsRng).unwrap_or_else(|err|
                    panic!("could not initialize thread_rng: {}", err));
            let rng = ReseedingRng::new(r,
                                        THREAD_RNG_RESEED_THRESHOLD,
                                        OsRng);
            Rc::new(UnsafeCell::new(rng))
        }
    );

    pub struct ThreadRng {
        // Rc is explictly !Send and !Sync
        pub(super) rng: Rc<UnsafeCell<ReseedingRng<Core, OsRng>>>,
    }

    pub fn lpn_thread_rng() -> ThreadRng {
        // contents of this function also borrowed from rand
        let rng = THREAD_RNG_KEY.with(|t| t.clone());
        ThreadRng { rng }
    }
}
#[cfg(feature = "csprng")]
pub use blockrngbased::{lpn_thread_rng, ThreadRng};

#[cfg(not(feature = "csprng"))]
mod xoshiro {
    use super::*;
    use rand::SeedableRng;
    use rand_xoshiro::Xoshiro256PlusPlus;

    thread_local! {
        static THREAD_RNG_KEY: Rc<UnsafeCell<Xoshiro256PlusPlus>> = {
            Rc::new(UnsafeCell::new(Xoshiro256PlusPlus::from_entropy()))
        }
    }

    pub struct ThreadRng {
        pub(super) rng: Rc<UnsafeCell<Xoshiro256PlusPlus>>,
    }

    pub fn lpn_thread_rng() -> ThreadRng {
        let rng = THREAD_RNG_KEY.with(|t| t.clone());
        ThreadRng { rng }
    }
}
#[cfg(not(feature = "csprng"))]
pub use xoshiro::{lpn_thread_rng, ThreadRng};

impl rand::RngCore for ThreadRng {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        // SAFETY: We must make sure to stop using `rng` before anyone else
        // creates another mutable reference
        let rng = unsafe { &mut *self.rng.get() };
        rng.next_u32()
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        // SAFETY: We must make sure to stop using `rng` before anyone else
        // creates another mutable reference
        let rng = unsafe { &mut *self.rng.get() };
        rng.next_u64()
    }

    #[inline(always)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // SAFETY: We must make sure to stop using `rng` before anyone else
        // creates another mutable reference
        let rng = unsafe { &mut *self.rng.get() };
        rng.fill_bytes(dest)
    }

    #[inline(always)]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        // SAFETY: We must make sure to stop using `rng` before anyone else
        // creates another mutable reference
        let rng = unsafe { &mut *self.rng.get() };
        rng.try_fill_bytes(dest)
    }
}

impl rand::CryptoRng for ThreadRng {}