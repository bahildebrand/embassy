#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use atsamd21g as pac;

pub mod config {
    pub struct Config {}
}

// pub fn init(config: config::Config) -> Peripherals {
pub fn init(_config: config::Config) {
    // Set flash wait states
    let nvmctrl = unsafe { &*pac::NVMCTRL::ptr() };
    nvmctrl.ctrlb.modify(|_, w| unsafe { w.rws().bits(0b1) });

    // Enable external crystal
    let sysctrl = unsafe { &*pac::SYSCTRL::ptr() };
    sysctrl.xosc32k.modify(|_, w| {
        w.ondemand().clear_bit();
        // Enable 32khz output
        w.en32k().set_bit();
        w.en1k().set_bit();
        // Crystal connected to xin32/xout32
        w.xtalen().set_bit();
        w.enable().set_bit();
        w.runstdby().set_bit()
    });

    // Wait for oscillator to stabilize
    while sysctrl.pclksr.read().xosc32krdy().bit_is_clear() {}

    // Configure GCLK1 divider
    let gclk = unsafe { &*pac::GCLK::ptr() };
    gclk.gendiv.modify(|_, w| unsafe {
        w.id().bits(0b1);
        w.div().bits(0b1)
    });

    // Configure GCLK1 to use external oscillator
    gclk.genctrl.modify(|_, w| unsafe {
        w.id().bits(0b1);
        w.src().xosc32k();
        w.idc().set_bit();
        w.genen().set_bit()
    });

    // Wait for write to finish
    while gclk.status.read().syncbusy().bit_is_set() {}

    // Connect DFLL to GCLK1 output
    gclk.clkctrl.modify(|_, w| {
        w.id().dfll48();
        w.gen().gclk1();
        w.clken().set_bit()
    });

    // Errata 1.2.1
    while sysctrl.pclksr.read().dfllrdy().bit_is_clear() {}
    sysctrl.dfllctrl.modify(|_, w| w.enable().set_bit());
    while sysctrl.pclksr.read().dfllrdy().bit_is_clear() {}

    sysctrl.dfllmul.modify(|_, w| unsafe {
        w.mul().bits(1465);
        w.fstep().bits(511);
        w.cstep().bits(31)
    });
    while sysctrl.pclksr.read().dfllrdy().bit_is_clear() {}

    sysctrl.dfllctrl.modify(|_, w| {
        w.mode().set_bit();
        w.waitlock().set_bit();
        w.enable().set_bit()
    });

    // Wait for DFLL lock
    while sysctrl.pclksr.read().dflllckc().bit_is_clear()
        || sysctrl.pclksr.read().dflllckf().bit_is_clear()
    {}
}
