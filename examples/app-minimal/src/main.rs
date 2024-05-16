#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use hal::{self, clocks::Clocks, pac};

#[rtic::app(device = pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let _dp = ctx.device;

        let clock_cfg = Clocks::default();
        clock_cfg.setup().unwrap();

        defmt::println!("Hello, world!");

        (Shared {}, Local {})
    }
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
