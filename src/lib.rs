//! # Use ws2812 leds with timers
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWrite` trait
//!
//! The `new` method needs a periodic timer running at 3 MHz
//!
//! If it's too slow (e.g.  e.g. all/some leds are white or display the wrong color)
//! you may want to try the `slow` feature.

#![no_std]

use embedded_hal as hal;

use crate::hal::digital::v2::OutputPin;
use crate::hal::timer::{CountDown, Periodic};
use smart_leds_trait::{SmartLedsWrite, RGB8};

use nb;
use nb::block;

pub struct Ws2812<TIMER, PIN> {
    timer: TIMER,
    pin: PIN,
}

impl<TIMER, PIN> Ws2812<TIMER, PIN>
where
    TIMER: CountDown + Periodic,
    PIN: OutputPin,
{
    /// The timer has to already run at with a frequency of 3 MHz
    pub fn new(timer: TIMER, mut pin: PIN) -> Ws2812<TIMER, PIN> {
        pin.set_low().ok();
        Self { timer, pin }
    }

    /// Write a single color for ws2812 devices
    #[cfg(feature = "slow")]
    #[inline]
    fn write_byte(&mut self, mut data: u8) {
        for _ in 0..8 {
            if (data & 0x80) != 0 {
                block!(self.timer.wait()).ok();
                self.pin.set_high().ok();
                block!(self.timer.wait()).ok();
                block!(self.timer.wait()).ok();
                self.pin.set_low().ok();
            } else {
                block!(self.timer.wait()).ok();
                self.pin.set_high().ok();
                self.pin.set_low().ok();
                block!(self.timer.wait()).ok();
                block!(self.timer.wait()).ok();
            }
            data <<= 1;
        }
    }

    /// Write a single color for ws2812 devices
    #[cfg(not(feature = "slow"))]
    #[inline]
    fn write_byte(&mut self, mut data: u8) {
        for _ in 0..8 {
            // Timer period is 333 nanoseconds.  Need TH+TL = 650-1850
            // nanoseconds.  Unknown how long GPIO changes take.
            if (data & 0x80) != 0 {
                block!(self.timer.wait()).ok();
                self.pin.set_high().ok(); // T1H = 550-850 nanoseconds
                block!(self.timer.wait()).ok();
                block!(self.timer.wait()).ok();
                self.pin.set_low().ok(); // T1L = 450-750 nanoseconds
                block!(self.timer.wait()).ok();
            } else {
                block!(self.timer.wait()).ok();
                self.pin.set_high().ok(); // T0H = 200 - 500 nanoseconds
                block!(self.timer.wait()).ok();
                self.pin.set_low().ok(); // T0L = 650 - 950 nanoseconds
                block!(self.timer.wait()).ok();
                block!(self.timer.wait()).ok();
            }
            data <<= 1;
        }
    }
}

impl<TIMER, PIN> SmartLedsWrite for Ws2812<TIMER, PIN>
where
    TIMER: CountDown + Periodic,
    PIN: OutputPin,
{
    type Error = ();
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        for item in iterator {
            let item = item.into();
            // We don't know how long until the timer will expire, so wait for
            // the end of a period to get consistent timing of first bit.
            block!(self.timer.wait()).ok();
            self.write_byte(item.g);
            self.write_byte(item.r);
            self.write_byte(item.b);
        }
        // WS2812 datasheet wants >50us
        // 1/(3MHz) = 333ns
        for _ in 0..(3 * 50 + 10) {
            block!(self.timer.wait()).ok();
        }
        Ok(())
    }
}
