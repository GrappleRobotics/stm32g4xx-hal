// From PR #50

use crate::stm32::CRS;

use super::{Rcc, Enable};

pub struct CrsConfig {}

pub struct Crs {
    rb: CRS,
}

impl Crs {
    /// Sets up the clock recovery system for the HSI48 oscillator.
    pub fn configure(self, crs_config: CrsConfig, rcc: &Rcc) -> Self {
      rcc.enable_hsi48();

      // Enable the clock recovery system
      CRS::enable(&rcc.rb);

      // Set to b10 for USB SOF as source
      self.rb
          .cfgr
          .modify(|_, w| unsafe { w.syncsrc().bits(0b10) });

      self.rb.cr.modify(|_, w| {
          // Set autotrim enabled.
          w.autotrimen().set_bit();
          // Enable CRS
          w.cen().set_bit()
      });

      self
    }
}

pub trait CrsExt {
    fn constrain(self) -> Crs;
}

impl CrsExt for CRS {
    fn constrain(self) -> Crs {
        Crs { rb: self }
    }
}