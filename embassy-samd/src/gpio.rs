use core::{convert::Infallible, marker::PhantomData};

use embassy::util::Unborrow;
use embassy_hal_common::{unborrow, unsafe_impl_unborrow};
use embedded_hal::digital::v2 as digital;

/// Represents a digital input or output level.
#[derive(Debug, Eq, PartialEq)]
pub enum Level {
    Low,
    High,
}

/// Represents a pull setting for an input.
#[derive(Debug, Eq, PartialEq)]
pub enum Pull {
    None,
    Up,
    Down,
}

/// A GPIO bank with up to 32 pins.
#[derive(Debug, Eq, PartialEq)]
pub enum Bank {
    Bank0 = 0,
    Qspi = 1,
}

// pub struct Input<'d, T: Pin> {
//     pin: T,
//     phantom: PhantomData<&'d mut T>,
// }

// impl<'d, T: Pin> Input<'d, T> {
//     pub fn new(pin: impl Unborrow<Target = T> + 'd, pull: Pull) -> Self {
//         unborrow!(pin);

//         unsafe {
//             pin.pad_ctrl().write(|w| {
//                 w.set_ie(true);
//                 match pull {
//                     Pull::Up => w.set_pue(true),
//                     Pull::Down => w.set_pde(true),
//                     Pull::None => {}
//                 }
//             });

//             // disable output in SIO, to use it as input
//             pin.sio_oe().value_clr().write_value(1 << pin.pin());

//             pin.io().ctrl().write(|w| {
//                 w.set_funcsel(pac::io::vals::Gpio0CtrlFuncsel::SIO_0.0);
//             });
//         }

//         Self {
//             pin,
//             phantom: PhantomData,
//         }
//     }

//     pub fn is_high(&self) -> bool {
//         !self.is_low()
//     }

//     pub fn is_low(&self) -> bool {
//         let val = 1 << self.pin.pin();
//         unsafe { self.pin.sio_in().read() & val == 0 }
//     }
// }

// impl<'d, T: Pin> Drop for Input<'d, T> {
//     fn drop(&mut self) {
//         // todo
//     }
// }

// impl<'d, T: Pin> digital::InputPin for Input<'d, T> {
//     type Error = Infallible;

//     fn is_high(&self) -> Result<bool, Self::Error> {
//         Ok(self.is_high())
//     }

//     fn is_low(&self) -> Result<bool, Self::Error> {
//         Ok(self.is_low())
//     }
// }

pub struct Output<'d, T: Pin> {
    pin: T,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Pin> Output<'d, T> {
    // TODO opendrain
    pub fn new(pin: impl Unborrow<Target = T> + 'd, initial_output: Level) -> Self {
        unborrow!(pin);

        Self {
            pin,
            phantom: PhantomData,
        }
    }

    /// Set the output as high.
    pub fn set_high(&mut self) {
        let val = 1 << self.pin.pin();
        unsafe { self.pin.sio_out().value_set().write_value(val) };
    }

    /// Set the output as low.
    pub fn set_low(&mut self) {
        let val = 1 << self.pin.pin();
        unsafe { self.pin.sio_out().value_clr().write_value(val) };
    }

    /// Is the output pin set as high?
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    /// Is the output pin set as low?
    pub fn is_set_low(&self) -> bool {
        // todo
        true
    }
}

impl<'d, T: Pin> Drop for Output<'d, T> {
    fn drop(&mut self) {
        // todo
    }
}

impl<'d, T: Pin> digital::OutputPin for Output<'d, T> {
    type Error = Infallible;

    /// Set the output as high.
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    /// Set the output as low.
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<'d, T: Pin> digital::StatefulOutputPin for Output<'d, T> {
    /// Is the output pin set as high?
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_high())
    }

    /// Is the output pin set as low?
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_low())
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Pin: Sized {
        fn pin_bank(&self) -> u8;

        #[inline]
        fn pin(&self) -> u8 {
            self.pin_bank() & 0x1f
        }

        #[inline]
        fn bank(&self) -> Bank {
            if self.pin_bank() & 0x20 == 0 {
                Bank::Bank0
            } else {
                Bank::Qspi
            }
        }

        fn io(&self) -> pac::io::Gpio {
            let block = match self.bank() {
                Bank::Bank0 => crate::pac::IO_BANK0,
                Bank::Qspi => crate::pac::IO_QSPI,
            };
            block.gpio(self.pin() as _)
        }

        fn pad_ctrl(&self) -> Reg<pac::pads::regs::GpioCtrl, RW> {}
        fn sio_out(&self) -> pac::sio::Gpio {}
        fn sio_oe(&self) -> pac::sio::Gpio {}
        fn sio_in(&self) -> Reg<u32, RW> {}
    }

    pub trait OptionalPin {}
}

pub trait Pin: Unborrow<Target = Self> + sealed::Pin {
    /// Degrade to a generic pin struct
    fn degrade(self) -> AnyPin {
        AnyPin {
            pin_bank: self.pin_bank(),
        }
    }
}

pub struct AnyPin {
    pin_bank: u8,
}
unsafe_impl_unborrow!(AnyPin);
impl Pin for AnyPin {}
impl sealed::Pin for AnyPin {
    fn pin_bank(&self) -> u8 {
        self.pin_bank
    }
}

// ==========================

pub trait OptionalPin: Unborrow<Target = Self> + sealed::OptionalPin + Sized {
    type Pin: Pin;
    fn pin(&self) -> Option<&Self::Pin>;
    fn pin_mut(&mut self) -> Option<&mut Self::Pin>;

    /// Convert from concrete pin type PIN_XX to type erased `Option<AnyPin>`.
    #[inline]
    fn degrade_optional(mut self) -> Option<AnyPin> {
        self.pin_mut()
            .map(|pin| unsafe { core::ptr::read(pin) }.degrade())
    }
}

impl<T: Pin> sealed::OptionalPin for T {}
impl<T: Pin> OptionalPin for T {
    type Pin = T;

    #[inline]
    fn pin(&self) -> Option<&T> {
        Some(self)
    }

    #[inline]
    fn pin_mut(&mut self) -> Option<&mut T> {
        Some(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NoPin;
unsafe_impl_unborrow!(NoPin);
impl sealed::OptionalPin for NoPin {}
impl OptionalPin for NoPin {
    type Pin = AnyPin;

    #[inline]
    fn pin(&self) -> Option<&AnyPin> {
        None
    }

    #[inline]
    fn pin_mut(&mut self) -> Option<&mut AnyPin> {
        None
    }
}

// ==========================

macro_rules! impl_pin {
    ($name:ident, $bank:expr, $pin_num:expr) => {
        impl Pin for peripherals::$name {}
        impl sealed::Pin for peripherals::$name {
            fn pin_bank(&self) -> u8 {
                ($bank as u8) * 32 + $pin_num
            }
        }
    };
}
