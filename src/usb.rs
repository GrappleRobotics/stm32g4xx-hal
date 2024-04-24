use stm32_usbd::UsbPeripheral;
pub use stm32_usbd::UsbBus;

use crate::{gpio::{gpioa::{PA11, PA12}, Floating, Input}, rcc::{Enable, Rcc, Reset}, stm32::{RCC, USB}};

pub struct Peripheral {
  pub usb: USB,
  pub pin_dm: PA11<Input<Floating>>,
  pub pin_dp: PA12<Input<Floating>>,
}

unsafe impl Sync for Peripheral {}

unsafe impl UsbPeripheral for Peripheral {
  const REGISTERS: *const () = USB::ptr() as *const ();
  const DP_PULL_UP_FEATURE: bool = false;
  const EP_MEMORY: *const () = 0x4000_6000 as _;
  const EP_MEMORY_SIZE: usize = 1024;
  const EP_MEMORY_ACCESS_2X16: bool = true;

  fn enable() {
      let rcc = unsafe {
        &*RCC::ptr()
      };

      cortex_m::interrupt::free(|_| {
          // Enable USB peripheral
          rcc.apb1enr1.modify(|_, w| w.usben().enabled());

          // Reset USB peripheral
          rcc.apb1rstr1.modify(|_, w| w.usbrst().reset());
          rcc.apb1rstr1.modify(|_, w| w.usbrst().clear_bit());
      });
  }

  fn startup_delay() {
      // The datasheet doesn't actually specify tstartup, so we're using the same one as the stm32f1xx-hal.
      cortex_m::asm::delay(170);
  }
}

pub enum ClockSource {
    Hsi48,
    PllQ,
}

// From PR #50
#[inline(always)]
pub fn configure_usb_clock_source(cs: ClockSource, rcc: &Rcc) {
    rcc.rb.ccipr.modify(|_, w| match cs {
        ClockSource::Hsi48 => w.clk48sel().hsi48(),
        ClockSource::PllQ => w.clk48sel().pllq(),
    });
}

pub type UsbBusType = UsbBus<Peripheral>;
