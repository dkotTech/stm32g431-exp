#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use hal::{
    self,
    adc::{Adc, AdcDevice, AdcInterrupt, Align, InputType, SampleTime},
    clocks::Clocks,
    dma,
    dma::{Dma, DmaChannel, DmaInput, DmaInterrupt, DmaPeriph},
    pac,
    pac::TIM3,
    pac::{ADC1, DMA1},
    timer::Timer,
    timer::TimerInterrupt,
};

static mut ADC_READ_BUF: [u16; 1] = [0; 1];

#[rtic::app(device = pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        adc1: Adc<ADC1>,
    }

    #[local]
    struct Local {
        timer: Timer<TIM3>,
        dma1: Dma<DMA1>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let dp = ctx.device;

        let clock_cfg = Clocks::default();
        clock_cfg.setup().unwrap();

        let mut adc = Adc::new_adc1(
            dp.ADC1,
            AdcDevice::One,
            Default::default(),
            clock_cfg.systick(),
        );

        adc.set_sequence(1, 2);
        adc.set_sequence_len(2);
        adc.set_input_type(2, InputType::Differential);
        adc.set_sample_time(2, SampleTime::T2);
        adc.set_align(Align::Left);
        adc.enable_interrupt(AdcInterrupt::EndOfSequence);

        let dma = Dma::new(dp.DMA1);
        dma::enable_mux1();
        dma::mux(DmaPeriph::Dma1, DmaChannel::C1, DmaInput::Adc1);

        let mut timer = Timer::new_tim3(dp.TIM3, 1., Default::default(), &clock_cfg);
        timer.enable_interrupt(TimerInterrupt::Update);
        timer.enable();

        (Shared { adc1: adc }, Local { timer, dma1: dma })
    }

    #[task(binds = DMA1_CH1, local=[dma1], shared=[adc1], priority = 1)]
    fn on_adc_dma_read(mut cx: on_adc_dma_read::Context) {
        dma::clear_interrupt(
            DmaPeriph::Dma1,
            DmaChannel::C1,
            DmaInterrupt::TransferComplete,
        );

        cx.local.dma1.stop(DmaChannel::C1);

        defmt::println!("ADC DMA read complete");

        let buf = unsafe { &mut ADC_READ_BUF };

        let voltage = cx.shared.adc1.lock(|adc| adc.reading_to_voltage(buf[0]));

        defmt::println!("voltage: {:?}", voltage);
    }

    #[task(binds = TIM3, local=[timer], shared=[adc1], priority = 2)]
    fn on_timer(mut cx: on_timer::Context) {
        cx.local.timer.clear_interrupt(TimerInterrupt::Update);

        cx.shared.adc1.lock(|adc| {
            unsafe {
                adc.read_dma(
                    &mut ADC_READ_BUF,
                    &[2],
                    DmaChannel::C1,
                    Default::default(),
                    DmaPeriph::Dma1,
                )
            };
        });
    }
}

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
