#![no_main]
#![no_std]
/// thx antoinevg for this example - https://github.com/antoinevg/stm32f3-rust-examples
use hal::gpio::AF1;
use hal::prelude::*;
use hal::pwr::PwrExt;
use hal::{rcc, stm32};
use stm32g4xx_hal as hal;

use fugit::ExtU32;

use cortex_m;

use defmt_rtt as _;
use panic_probe as _;

pub const DMA_LENGTH: usize = 16;
pub static DUTY_CYCLES: [u16; DMA_LENGTH] = [1000, 1000, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8];

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let cp = cortex_m::Peripherals::take().expect("cannot take core peripherals");

    init_gpios(&dp);

    unsafe {
        init_tim2(&dp);
        init_dma2(&dp);
        // init_dma2_other_way(&dp)
    }

    let rcc = dp.RCC.constrain();
    let mut delay_syst = cp.SYST.delay(&rcc.clocks);

    let pwr = dp.PWR.constrain().freeze();
    let rcc = rcc.freeze(rcc::Config::hsi(), pwr);

    let tim = dp.TIM2;

    loop {
        delay_syst.delay(1000.millis());

        defmt::println!("ccr1: {:?}", tim.ccr1().read().bits());
        defmt::println!("ccr2: {:?}", tim.ccr2().read().bits());
        defmt::println!("ccr3: {:?}", tim.ccr3().read().bits());
        defmt::println!("ccr4: {:?}", tim.ccr4().read().bits());
    }
}

pub fn init_gpios(dp: &stm32::Peripherals) {
    let rcc = &dp.RCC;
    rcc.ahb2enr.modify(|_, w| w.gpioaen().set_bit()); // en gpioA

    let gpioN = 0;

    let mode = AF1 as u32; // select 1 alt function
    let offset = 2 * gpioN;
    let offset2 = 4 * gpioN;

    let gpio = &dp.GPIOA;

    unsafe {
        if offset2 < 32 {
            gpio.afrl
                .modify(|r, w| w.bits((r.bits() & !(0b1111 << offset2)) | (mode << offset2)));
        } else {
            let offset2 = offset2 - 32;
            gpio.afrh
                .modify(|r, w| w.bits((r.bits() & !(0b1111 << offset2)) | (mode << offset2)));
        }
        gpio.moder
            .modify(|r, w| w.bits((r.bits() & !(0b11 << offset)) | (0b10 << offset)));
        gpio.otyper
            .modify(|r, w| w.bits(r.bits() & !(0b1 << gpioN)));
    }
}

pub unsafe fn init_tim2(dp: &stm32::Peripherals) {
    // enable TIM2 clock
    let rcc = &dp.RCC;
    rcc.apb1enr1.modify(|_, w| w.tim2en().set_bit());

    let sysclk = 170_000_000; // 170 Mhz by default
    let fs = 20_000;
    let arr = sysclk / fs - 1;

    // configure TIM2
    let tim2 = &dp.TIM2;
    tim2.psc.write(|w| w.psc().bits(0u16));
    tim2.arr.write(|w| w.arr().bits(arr));
    // tim2.cr1.write(|w| w.cen().set_bit());  // enable TIM2, but we enable it in dma init

    tim2.ccmr1_output().modify(
        |_, w| {
            w.oc1pe()
                .set_bit() // Enable preload
                .oc1m()
                .pwm_mode1()
        }, // PWM Mode
    );

    tim2.ccer.modify(|_, w| w.cc1e().set_bit());
}

// init_dma2 set up a dma burst for time2
pub unsafe fn init_dma2(dp: &stm32::Peripherals) {
    // enable DMA1 clock
    let rcc = &dp.RCC;
    rcc.ahb1enr.modify(|_, w| w.dmamuxen().set_bit()); // mux en
    rcc.ahb1enr.modify(|_, w| w.dma1en().set_bit()); // dma1 en

    let mux = &dp.DMAMUX;
    mux.c0cr.modify(|_, w| w.dmareq_id().bits(60)); // 56-TIM2_CH1 60-TIM2_UP

    // dma parameters
    let ma = DUTY_CYCLES.as_ptr() as usize as u32; // mem address
    let pa = dp.TIM2.dmar.as_ptr() as usize as u32; // time2 dmar register address
    let ndt = DMA_LENGTH as u16;

    // configure DMA1 channel 1
    let dma1 = &dp.DMA1;
    dma1.cmar1.write(|w| w.ma().bits(ma)); // source memory address
    dma1.cpar1.write(|w| w.pa().bits(pa)); // destination peripheral address
    dma1.cndtr1.write(|w| w.ndt().bits(ndt)); // number of items to transfer
    dma1.ccr1.write(|w| {
        w.
            mem2mem().clear_bit().   // source is memory, disable memory to memory transfer
            pl().bits(1).      // set dma priority
            msize().bits(1).   // memory word size is 16 bits
            psize().bits(2).   // peripheral word size is 32 bits

            minc().set_bit().        // increment memory address every transfer
            pinc().clear_bit().      // not increment peripheral address every transfer

            circ().set_bit().        // dma mode is circular
            dir().set_bit().         // set to read from memory
            teie().clear_bit().      // trigger an interrupt if an error occurs
            htie().clear_bit().      // trigger an interrupt when half the transfer is complete
            tcie().clear_bit() // trigger an interrupt when transfer is complete
    });

    // enable DMA transfers for TIM2
    let tim = &dp.TIM2;
    tim.dcr.modify(|_, w| {
        // set up registers for burst mode
        w.dbl().bits(3u8).    // size of burst, 0 - first register after offset, 1 - second etc
            dba().bits(0xDu8) // burst offset
    });
    tim.dier.modify(|_, w| w.ude().set_bit()); // Update DMA request enable

    tim.cr1.modify(|_, w| w.cen().set_bit()); // en tim

    dma1.ccr1.modify(|_, w| w.en().set_bit()); // en dma
}

// init_dma2_other_way set up a dma to circular DUTY_CYCLES for ccr1 register
pub unsafe fn init_dma2_other_way(dp: &stm32::Peripherals) {
    // enable DMA1 clock
    let rcc = &dp.RCC;
    rcc.ahb1enr.modify(|_, w| w.dmamuxen().set_bit()); // mux en
    rcc.ahb1enr.modify(|_, w| w.dma1en().set_bit()); // dma1 en

    let mux = &dp.DMAMUX;
    mux.c0cr.modify(|_, w| w.dmareq_id().bits(56)); // 56-TIM2_CH1 60-TIM2_UP

    // dma parameters
    let ma = DUTY_CYCLES.as_ptr() as usize as u32;
    let pa = dp.TIM2.ccr[0].as_ptr() as usize as u32;
    let ndt = DMA_LENGTH as u16;

    // configure DMA1 channel 1
    let dma1 = &dp.DMA1;
    dma1.cmar1.write(|w| w.ma().bits(ma)); // source memory address
    dma1.cpar1.write(|w| w.pa().bits(pa)); // destination peripheral address
    dma1.cndtr1.write(|w| w.ndt().bits(ndt)); // number of items to transfer

    dma1.ccr1.write(|w| {
        w.
            mem2mem().clear_bit().   // source is memory, disable memory to memory transfer
            pl().bits(1).      // set dma priority
            msize().bits(1).   // memory word size is 16 bits
            psize().bits(2).   // peripheral word size is 32 bits

            minc().set_bit().        // increment memory address every transfer
            pinc().clear_bit().      // not increment peripheral address every transfer

            circ().set_bit().        // dma mode is circular
            dir().set_bit().         // set to read from memory
            teie().clear_bit().      // trigger an interrupt if an error occurs
            htie().clear_bit().      // trigger an interrupt when half the transfer is complete
            tcie().clear_bit() // trigger an interrupt when transfer is complete
    });

    // enable DMA transfers for TIM2
    let tim = &dp.TIM2;
    tim.dier.modify(|_, w| w.cc1de().set_bit()); // Capture/Compare 1 DMA request enable

    tim.cr1.modify(|_, w| w.cen().set_bit()); // en tim

    dma1.ccr1.modify(|_, w| w.en().set_bit()); // en dma
}
