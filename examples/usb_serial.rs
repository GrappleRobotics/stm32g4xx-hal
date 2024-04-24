#![no_main]
#![no_std]

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use hal::{rcc::CrsExt, usb::configure_usb_clock_source};
use stm32g4xx_hal as hal;

use crate::hal::{
  prelude::*,
  pwr::PwrExt,
  rcc::{Config, RccExt, SysClockSrc, PllConfig, PLLSrc, PllNMul},
  stm32::Peripherals,
  time::RateExtU32,
  usb::{Peripheral, UsbBus},
};

use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[macro_use]
mod utils;

#[entry]
fn main() -> ! {
  let dp = Peripherals::take().unwrap();
  let _cp = cortex_m::Peripherals::take().expect("cannot take core peripherals");

  let mut pll_cfg = PllConfig::default();
  pll_cfg.mux = PLLSrc::HSE(8u32.MHz());
  pll_cfg.n = PllNMul::MUL_24;
  pll_cfg.r = Some(stm32g4xx_hal::rcc::PllRDiv::DIV_2);
  pll_cfg.q = Some(stm32g4xx_hal::rcc::PllQDiv::DIV_4);

  let pwr = dp.PWR.constrain().freeze();
  let mut rcc = dp.RCC.freeze(Config::new(SysClockSrc::PLL).pll_cfg(pll_cfg), pwr);

  let crs = dp.CRS.constrain();
  crs.configure(hal::rcc::CrsConfig {  }, &rcc);
  configure_usb_clock_source(hal::usb::ClockSource::Hsi48, &rcc);

  let gpioa = dp.GPIOA.split(&mut rcc);

  let mut usb_dp = gpioa.pa12.into_push_pull_output();
  usb_dp.set_low().ok();
  delay(rcc.clocks.sys_clk.raw() / 100);

  let usb = Peripheral {
      usb: dp.USB,
      pin_dm: gpioa.pa11,
      pin_dp: usb_dp.into_floating_input(),
  };
  let usb_bus = UsbBus::new(usb);

  // let mut serial = SerialPort::new(&usb_bus);
  let rx_buffer: [u8; 128] = [0; 128];
  let tx_buffer: [u8; 128] = [0; 128];

  let mut serial = SerialPort::new_with_store(&usb_bus, rx_buffer, tx_buffer);

  let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();

  loop {
    if !usb_dev.poll(&mut [&mut serial]) {
      continue;
    }

    let mut buf = [0u8; 64];

    match serial.read(&mut buf) {
      Ok(count) if count > 0 => {
        // Echo back in upper case
        for c in buf[0..count].iter_mut() {
          if 0x61 <= *c && *c <= 0x7a {
            *c &= !0x20;
          }
        }

        let mut write_offset = 0;
        while write_offset < count {
          match serial.write(&buf[write_offset..count]) {
            Ok(len) if len > 0 => {
              write_offset += len;
            }
            _ => {}
          }
        }
      }
      _ => {}
    }
  }
}