//! The root task.

use crate::{consts::*, thr, thr::ThrsInit, Regs};
//use drone_core::log;
use drone_cortexm::{fib, reg::prelude::*, /*swo,*/ thr::prelude::*};
use drone_stm32_map::{
    periph::{
        gpio::{periph_gpio_a8, pin::GpioPinMap, pin::GpioPinPeriph},
        sys_tick::{periph_sys_tick, SysTickPeriph},
    },
    reg,
};
use futures::prelude::*;

/// An error returned when a receiver has missed too many ticks.
#[derive(Debug)]
pub struct TickOverflow;

/// The root task handler.
#[inline(never)]
pub fn handler(reg: Regs, thr_init: ThrsInit) {
    let thr = thr::init(thr_init);
    let sys_tick = periph_sys_tick!(reg);

    thr.hard_fault.add_once(|| panic!("Hard Fault"));

    raise_system_frequency(
        reg.flash_acr,
        // reg.pwr_cr,
        reg.rcc_pllcfgr,
        reg.rcc_cfgr,
        reg.rcc_cir,
        reg.rcc_cr,
        reg.rcc_apb1enr,
        thr.rcc,
    )
    .root_wait();

    // Enable power for GPIOA
    reg.rcc_ahb1enr.gpioaen.set_bit();

    let led = periph_gpio_a8!(reg);

    beacon(led, sys_tick, thr.sys_tick)
        .root_wait()
        .expect("beacon fail");

    // Enter a sleep state on ISR exit.
    reg.scb_scr.sleeponexit.set_bit();
}

async fn raise_system_frequency(
    flash_acr: reg::flash::Acr<Srt>,
    // pwr_cr: reg::pwr::Cr<Srt>,
    rcc_pllcfgr: reg::rcc::Pllcfgr<Srt>,
    rcc_cfgr: reg::rcc::Cfgr<Srt>,
    rcc_cir: reg::rcc::Cir<Srt>,
    rcc_cr: reg::rcc::Cr<Srt>,
    rcc_apb1enr: reg::rcc::Apb1Enr<Srt>,
    thr_rcc: thr::Rcc,
) {
    // Enable Power Control Clock.
    rcc_apb1enr.pwren.set_bit();

    // The '405 has a single bit VOS field, but it doesn't seem to be in the
    // SVD file. The power on default is 1, which is what we want to set it
    // to anyways, so we'll just leave this commented out for now.
    //
    // The original code below was for the F411 which has a 2-bit VOS field.
    //
    // Set VOS field in PWR_CR register to Scale 1 (0b11) (<= 100 MHz)
    // pwr_cr.modify(|r| r.write_vos(0b11));

    thr_rcc.enable_int();
    rcc_cir.modify(|r| r.set_hserdyie().set_pllrdyie());

    // We need to move ownership of `hserdyc` and `hserdyf` into the fiber.
    let reg::rcc::Cir {
        hserdyc, hserdyf, ..
    } = rcc_cir;
    // Attach a listener that will notify us when RCC_CIR_HSERDYF is asserted.
    let hserdy = thr_rcc.add_future(fib::new_fn(move || {
        if hserdyf.read_bit() {
            hserdyc.set_bit();
            fib::Complete(())
        } else {
            fib::Yielded(())
        }
    }));

    // Turn on the HSE and wait for it to become ready
    rcc_cr.hseon.set_bit();
    // Sleep until RCC_CIR_HSERDYF is asserted.
    hserdy.await;

    // We need to move ownership of `pllrdyc` and `pllrdyf` into the fiber.
    let reg::rcc::Cir {
        pllrdyc, pllrdyf, ..
    } = rcc_cir;
    // Attach a listener that will notify us when RCC_CIR_PLLRDYF is asserted.
    let pllrdy = thr_rcc.add_future(fib::new_fn(move || {
        if pllrdyf.read_bit() {
            pllrdyc.set_bit();
            fib::Complete(())
        } else {
            fib::Yielded(())
        }
    }));

    rcc_pllcfgr.modify(|r| {
        r.set_pllsrc() // HSE oscillator clock selected as PLL input clock
            .write_pllm(PLLM)
            .write_plln(PLLN)
            .write_pllp(PLLP)
            .write_pllq(PLLQ)
    });
    // Enable the PLL.
    rcc_cr.modify(|r| r.set_pllon());
    // Sleep until RCC_CIR_PLLRDYF is asserted.
    pllrdy.await;

    // The power-on reset latency is set to zero. Since we're going to be increasing
    // the clock speed, increase the latency before increasing the clock speed.
    flash_acr.modify(|r| r.write_latency(FLASH_LATENCY));
    if flash_acr.load().latency() != FLASH_LATENCY {
        panic!("LATENCY");
    }

    rcc_cfgr.modify(|r| {
        r.write_ppre1(0b101) // 0b101 = Divide by 4 - APB1 - 24 MHz (must be < 45 MHz)
            .write_ppre2(0b100) // 0b100 = Divide by 2 - APB2 - 48 MHz (must be < 90 MHz)
            .write_hpre(0b0000) // 0b0000 = Divide by 1 - AHB - 96 MHz
            .write_sw(0b10) // 0b10 = Use PLL for System Clock - 96 MHz
    });

    if rcc_cfgr.load().sws() != 0b10 {
        panic!("SYSCLK not set to HSE");
    }
}

async fn beacon<T: GpioPinMap>(
    led: GpioPinPeriph<T>,
    sys_tick: SysTickPeriph,
    thr_sys_tick: thr::SysTick,
) -> Result<(), TickOverflow> {
    led.gpio_moder_moder.write_bits(0b01); // 0b01 - General purpose output mode.
    led.gpio_otyper_ot.clear_bit(); // 0 = Push-Pull
    led.gpio_ospeedr_ospeedr.write_bits(0b11); // 0b11 - High Speed

    // Attach a listener that will notify us on each interrupt trigger.
    let mut tick_stream = thr_sys_tick.add_stream_pulse(
        // This closure will be called when a receiver no longer can store the
        // number of ticks since the last stream poll. If this happens, a
        // `TickOverflow` error will be sent over the stream as is final value.
        || Err(TickOverflow),
        // A fiber that will be called on each interrupt trigger. It sends a
        // single tick over the stream.
        fib::new_fn(|| fib::Yielded(Some(1))),
    );
    // Clear the current value of the timer.
    sys_tick.stk_val.store(|r| r.write_current(0));
    // Set the value to load into the `stk_val` register when the counter
    // reaches 0. We set it to the count of SysTick clocks per second divided by
    // 8, so the reload will be triggered each 125 ms.
    sys_tick
        .stk_load
        .store(|r| r.write_reload(SYS_TICK_FREQ / 8));
    sys_tick.stk_ctrl.store(|r| {
        r.set_tickint() // Counting down to 0 triggers the SysTick interrupt
            .set_enable() // Start the counter in a multi-shot way
    });

    // A value cycling from 0 to 7. Full cycle represents a full second.
    let mut counter: u32 = 0;
    while let Some(tick) = tick_stream.next().await {
        for _ in 0..tick?.get() {
            // Each full second print a message.
            if counter == 0 {
                println!("sec");
            }
            match counter {
                // On 0's and 250's millisecond pull the pin low.
                0 | 2 => {
                    led.gpio_bsrr_br.set_bit();
                }
                // On 125's, 375's, 500's, 625's, 750's, and 875's millisecond
                // pull the pin high.
                _ => {
                    led.gpio_bsrr_bs.set_bit();
                }
            }
            counter = (counter + 1) % 8;
        }
    }

    Ok(())
}
