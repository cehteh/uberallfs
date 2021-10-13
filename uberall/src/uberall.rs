use rand::distributions::Standard;
use rand::prelude::*;
use rand_core::OsRng;
use rand_hc::Hc128Rng;

use crate::prelude::*;

/// Shared Application state
#[derive(Debug)]
pub struct UberAll {
    rng: Hc128Rng,
}

impl UberAll {
    pub fn new() -> Result<Self> {
        Ok(UberAll {
            rng: Hc128Rng::from_rng(OsRng)?,
        })
    }

    pub fn rng_gen<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        self.rng.gen()
    }

    // PLANNED: provide multiple mutex<queues> of u8 filled with randoms by a thread
    // which get woken up when any queue hits lowwater. trylock these round
    // robin to acquire randoms. keep start index for roundrobin in a atomic
    // counter
}
