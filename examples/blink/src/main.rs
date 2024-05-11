//! This minimial example causes an LED to blink using a (blocking) systick delay. It's
//! the canonical "Hello world" of embedded programming. It demonstrates project structure,
//! printing text to the console, using systick delays, and setting GPIO state.

#![no_std]
#![no_main]

use cortex_m::delay::Delay;
use cortex_m_rt::entry; // The runtime

use hal::{
    self,
    clocks::Clocks,
    gpio::{Pin, PinMode, Port},
    pac,
};

use defmt_rtt as _;
// global logger
use panic_probe as _;

// This marks the entrypoint of our application.

#[entry]
fn main() -> ! {
    // Set up CPU peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
    // Set up microcontroller peripherals
    let _dp = pac::Peripherals::take().unwrap();

    let clock_cfg = Clocks::default();

    // Write the clock configuration to the MCU. If you wish, you can modify `clock_cfg` above
    // in accordance with [its docs](https://docs.rs/stm32-hal2/latest/stm32_hal2/clocks/index.html),
    // and the `clock_cfg` example.
    clock_cfg.setup().unwrap();

    // Setup a delay, based on the Cortex-m systick.
    let mut delay = Delay::new(cp.SYST, clock_cfg.systick());

    let mut led_green = Pin::new(Port::B, 7, PinMode::Output);
    let mut led_blue = Pin::new(Port::C, 11, PinMode::Output);
    let mut led_red = Pin::new(Port::C, 10, PinMode::Output);

    led_green.set_high();
    led_blue.set_high();
    led_red.set_high();

    loop {
        led_green.set_low();
        delay.delay_ms(1_000);
        led_green.set_high();

        led_blue.set_low();
        delay.delay_ms(1_000);
        led_blue.set_high();

        led_red.set_low();
        delay.delay_ms(1_000);
        led_red.set_high();

        defmt::println!("Hello, world!");
    }
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
