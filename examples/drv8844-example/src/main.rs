#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use hal::{
    self,
    clocks::Clocks,
    dma,
    dma::{ChannelCfg, Circular, Dma, DmaChannel, DmaInput, DmaPeriph, IncrMode, Priority},
    gpio::{Edge, Pin, PinMode, Port, Pull},
    pac,
    pac::{TIM2, TIM3},
    timer::{
        Alignment, CaptureCompareDma, CountDir, OutputCompare, TimChannel, Timer, TimerConfig,
        TimerInterrupt, UpdateReqSrc,
    },
};
pub static BUF: [u16; 4] = [4249, 4249, 4249, 4249];

#[rtic::app(device = pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        timer_pwd: Timer<TIM2>,
        timer: Timer<TIM3>,
    }

    fn init_pins() {
        // setup pins
        let mut sw1_button = Pin::new(Port::A, 10, PinMode::Input);
        sw1_button.pull(Pull::Up);
        sw1_button.enable_interrupt(Edge::Rising); // and enable interrupt

        let mut dr_reset = Pin::new(Port::B, 2, PinMode::Output);
        dr_reset.set_high();

        let mut dr_en = Pin::new(Port::A, 4, PinMode::Output);
        dr_en.set_high();

        // driver step pin for motor pwd control
        Pin::new(Port::A, 1, PinMode::Alt(1)); // ch1 - in1 - a2 -- ch2
        Pin::new(Port::A, 0, PinMode::Alt(1)); // ch2 - in2 - a1 -- ch1
        Pin::new(Port::B, 11, PinMode::Alt(1)); // ch3 - in3 - b1 -- ch4
        Pin::new(Port::B, 10, PinMode::Alt(1)); // ch4 - in4 - b2 -- ch3
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let dp = ctx.device;

        hal::debug_workaround();

        let clock_cfg = Clocks::default();
        clock_cfg.setup().unwrap();

        init_pins();

        let mut timer_pwd = Timer::new_tim2(
            dp.TIM2,
            20_000.,
            TimerConfig {
                one_pulse_mode: false,
                update_request_source: UpdateReqSrc::OverUnderFlow,
                auto_reload_preload: true,
                alignment: Alignment::Edge,
                capture_compare_dma: CaptureCompareDma::Ccx,
                direction: CountDir::Up,
            },
            &clock_cfg,
        );

        timer_pwd.enable_pwm_output(TimChannel::C1, OutputCompare::Pwm1, 0.1);
        timer_pwd.enable_pwm_output(TimChannel::C2, OutputCompare::Pwm1, 0.1);

        timer_pwd.enable_pwm_output(TimChannel::C3, OutputCompare::Pwm1, 0.1);
        timer_pwd.enable_pwm_output(TimChannel::C4, OutputCompare::Pwm1, 0.1);

        let mut dma = Dma::new(dp.DMA1);
        dma::mux(DmaPeriph::Dma1, DmaChannel::C1, DmaInput::Tim2Up);

        timer_pwd.enable_interrupt(TimerInterrupt::UpdateDma);

        unsafe {
            timer_pwd.write_dma_burst(
                &BUF,
                13,
                4,
                DmaChannel::C1,
                ChannelCfg {
                    priority: Priority::Medium,
                    circular: Circular::Enabled,
                    periph_incr: IncrMode::Disabled,
                    mem_incr: IncrMode::Enabled,
                },
                true,
                DmaPeriph::Dma1,
            );
        }

        let mut timer = Timer::new_tim3(dp.TIM3, 0.2, Default::default(), &clock_cfg);
        timer.enable_interrupt(TimerInterrupt::Update);
        timer.enable();

        (Shared {}, Local { timer_pwd, timer })
    }

    #[task(binds = TIM3, local=[timer_pwd, timer, a: i32 = 0], priority = 1)]
    fn on_dma1_complete(cx: on_dma1_complete::Context) {
        cx.local.timer.clear_interrupt(TimerInterrupt::Update);

        defmt::println!("psc: {:?}", cx.local.timer_pwd.regs.psc.read().bits());
        defmt::println!("arr: {:?}", cx.local.timer_pwd.regs.arr.read().bits());
        defmt::println!("rcr: {:?}", cx.local.timer_pwd.regs.rcr.read().bits());

        defmt::println!(
            "cx.local.timer_pwd1: {:?}",
            cx.local.timer_pwd.get_duty(TimChannel::C1)
        );
        defmt::println!(
            "cx.local.timer_pwd2: {:?}",
            cx.local.timer_pwd.get_duty(TimChannel::C2)
        );
        defmt::println!(
            "cx.local.timer_pwd3: {:?}",
            cx.local.timer_pwd.get_duty(TimChannel::C3)
        );
        defmt::println!(
            "cx.local.timer_pwd4: {:?}",
            cx.local.timer_pwd.get_duty(TimChannel::C4)
        );
    }
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
