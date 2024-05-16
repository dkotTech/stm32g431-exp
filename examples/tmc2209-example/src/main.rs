#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use hal::{
    self,
    clocks::Clocks,
    gpio,
    gpio::{Edge, Pin, PinMode, Port, Pull},
    pac,
    timer::{OutputCompare, TimChannel, Timer, TimerConfig},
    usart::{Usart, UsartConfig},
};

#[rtic::app(device = pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        m_directory: Pin,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let dp = ctx.device;

        let clock_cfg = Clocks::default();
        clock_cfg.setup().unwrap();

        // setup pins
        let mut sw1_button = Pin::new(Port::A, 10, PinMode::Input);
        sw1_button.pull(Pull::Up);
        sw1_button.enable_interrupt(Edge::Rising); // and enable interrupt

        // Configure pins for UART, according to the user manual.
        let _uart_tx = Pin::new(Port::B, 10, PinMode::Alt(7));
        let _uart_rx = Pin::new(Port::B, 11, PinMode::Alt(7));

        // control motor directory pin
        let mut m_directory = Pin::new(Port::B, 0, PinMode::Output);
        m_directory.set_low();

        // driver step pin for motor pwd control
        let _m_step = Pin::new(Port::B, 1, PinMode::Alt(2));

        // set up pwd timer for pin PB1
        // stm32g431rb datasheet - Table 13. Alternate function
        // freq in Hz
        let mut timer_pwd = Timer::new_tim3(dp.TIM3, 1000., TimerConfig::default(), &clock_cfg);
        timer_pwd.enable_pwm_output(TimChannel::C4, OutputCompare::Pwm1, 0.5); // duty 0.5 == 50%
        timer_pwd.enable();

        // set up uart for communicate with tmc2209 driver
        let mut uart = Usart::new(dp.USART3, 9600, UsartConfig::default(), &clock_cfg);

        // set up default gconf and send it to driver
        let mut gconf = tmc2209::reg::GCONF::default();
        gconf.set_pdn_disable(true);

        uart.write(tmc2209::write_request(0, gconf).bytes())
            .unwrap();

        (Shared {}, Local { m_directory })
    }

    // EXTI15_10 - interrupt line for pins with 10 - 15 pin numbers
    #[task(binds = EXTI15_10, local=[m_directory], priority = 1)]
    fn on_sw1_button(cx: on_sw1_button::Context) {
        gpio::clear_exti_interrupt(10);

        // a possible way to get a pin state, I think better to use local or share
        let b = Pin::new(Port::A, 10, PinMode::Input);

        if b.is_low() {
            defmt::println!("m_directory.toggle");
            cx.local.m_directory.toggle();
        }
    }
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
