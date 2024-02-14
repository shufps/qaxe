#![no_std]
#![no_main]

use core::option::Option::Some;
use defmt::{panic, *};
use defmt_rtt as _; // global logger
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Input, Level, Output, OutputType, Pull, Speed};
use embassy_stm32::i2c;
use embassy_stm32::i2c::I2c;
use embassy_stm32::rcc::*;
use embassy_stm32::time::{khz, Hertz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel as PWMChannel;
use embassy_stm32::usart::Uart;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{bind_interrupts, peripherals, usart, usb};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use embedded_hal::serial::Read;
use static_cell::StaticCell;

use embassy_stm32::timer::OutputPolarity;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::Builder;
use embedded_io_async::Write;
use futures::future::join3;
use panic_probe as _;

extern crate alloc;
extern crate alloc_cortex_m;

mod protobuf;
use protobuf::coms::{QControl, QRequest, QResponse, QState};
use quick_protobuf::{self, MessageWrite};

use alloc::borrow::Cow;

use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;

});
use embassy_stm32::peripherals::*;

#[derive(PartialEq)]
enum ResetManagerCommand {
    Reset,
}

#[derive(PartialEq)]
enum PowerManagerCommand {
    BuckOn,
    BuckOff,
}

static RESET_MANAGER_SIGNAL: Signal<CriticalSectionRawMutex, ResetManagerCommand> = Signal::new();
static POWER_MANAGER_SIGNAL: Signal<CriticalSectionRawMutex, PowerManagerCommand> = Signal::new();

static PGOOD: Mutex<ThreadModeRawMutex, bool> = Mutex::new(false);

const RX_BUF_SIZE : usize = 64;
const TX_BUF_SIZE : usize = 64;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    // Initialize the allocator before using it
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024;
    unsafe { ALLOCATOR.init(start, size) }

    let mut config = embassy_stm32::Config::default();
    config.rcc.mux = ClockSrc::PLL1_R;
    config.rcc.hse = Some(Hse {
        freq: Hertz::mhz(8),
        mode: HseMode::Bypass,
    });
    config.rcc.pll = Some(Pll {
        source: PllSource::HSE,
        div: PllDiv::DIV3,
        mul: PllMul::MUL12,
    });

    let p = embassy_stm32::init(config);

    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcaff);
    config.max_packet_size_0 = 64;
    config.manufacturer = Some("Microengineer");
    config.product = Some("Qaxe-Testing");
    config.serial_number = Some("rev2");
    config.self_powered = true;

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state_usb_asic = State::new();
    let mut state_usb_ctrl = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    let class_usb_asic = CdcAcmClass::new(&mut builder, &mut state_usb_asic, 64);
    let (mut sender, mut receiver) = class_usb_asic.split();

    // Build the builder.
    let mut usb = builder.build();

    let mut config = usart::Config::default();
    config.baudrate = 115200;

    let usart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH4, p.DMA1_CH5, config).unwrap();
    let (mut tx_asics, mut rx_asics) = usart.split();


    // Run the USB device.
    let usb_fut = usb.run();

    let _run_1v2 = Output::new(p.PA2, Level::High, Speed::Low);
    let pgood_1v2 = Input::new(p.PA3, Pull::None);
    let pgood_led = Output::new(p.PA5, Level::High, Speed::Low);
    let _activity_led = Output::new(p.PA4, Level::High, Speed::Low);

    let reset = Output::new(p.PB13, Level::High, Speed::Low);

    let ch1 = PwmPin::new_ch1(p.PA0, OutputType::PushPull);
    let ch2 = PwmPin::new_ch2(p.PA1, OutputType::PushPull);
    let mut pwm1 = SimplePwm::new(
        p.TIM2,
        Some(ch1),
        Some(ch2),
        None,
        None,
        khz(10),
        Default::default(),
    );
    pwm1.set_polarity(PWMChannel::Ch1, OutputPolarity::ActiveHigh);
    pwm1.set_polarity(PWMChannel::Ch2, OutputPolarity::ActiveHigh);
    pwm1.set_duty(PWMChannel::Ch1, pwm1.get_max_duty() / 2);
    pwm1.set_duty(PWMChannel::Ch2, pwm1.get_max_duty() / 4);
    pwm1.enable(PWMChannel::Ch1);
    pwm1.enable(PWMChannel::Ch2);

    let mut i2c_config = embassy_stm32::i2c::Config::default();
    i2c_config.scl_pullup = true;
    i2c_config.sda_pullup = true;

    let i2c = I2c::new(
        p.I2C2,
        p.PB10,
        p.PB11,
        Irqs,
        NoDma,
        NoDma,
        Hertz(100_000),
        i2c_config, /*Default::default()*/
    );

    unwrap!(spawner.spawn(reset_manager(reset)));
    unwrap!(spawner.spawn(power_good_task(pgood_1v2, pgood_led)));
    unwrap!(spawner.spawn(temp_manager(i2c)));

    let relay_receiver_fut = async {
        loop {
            let mut usb_buf = [0; 1];
            receiver.wait_connection().await;
            info!("Connected relay receiver");

            loop {
                let usb_read = match receiver.read_packet(&mut usb_buf).await {
                    Ok(n) => n,
                    Err(e) => {
                        error!("Error reading from USB: {:?}", e);
                        break;
                    }
                };

                if usb_read == 0 {
                    continue; // No data read, continue the loop
                }

                debug!("USB -> USART: {:x}", &usb_buf[..usb_read]);

                if let Err(e) = tx_asics.write_all(&usb_buf[..usb_read]).await {
                    error!("Error writing to USART: {:?}", e);
                    break;
                }
            }
        }
    };

    let relay_sender_fut = async {
        loop {
            sender.wait_connection().await;
            info!("Connected relay sender");

            'outer: loop {
                let mut buf = [0u8;1];

                match rx_asics.read(&mut buf).await {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Error reading from USART: {:?}", e);
                        break;
                    }
                };

                debug!("USART -> USB: {:x}", &buf);
                if let Err(e) = sender.write_packet(&buf).await {
                    error!("Error writing to USB: {:?}", e);
                    break 'outer;
                }
            }
        }
    };
    let _ = join3(
        usb_fut,
        relay_receiver_fut,
        relay_sender_fut,
    )
    .await;
}

#[embassy_executor::task]
async fn power_good_task(pgood_1v2: Input<'static, PA3>, mut pgood_led: Output<'static, PA5>) {
    loop {
        let mut pgood_state = PGOOD.lock().await;
        if pgood_1v2.is_high() {
            *pgood_state = true;
            pgood_led.set_low();
        } else {
            *pgood_state = false;
            pgood_led.set_high();
        }
        drop(pgood_state);
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn reset_manager(mut reset: Output<'static, PB13>) {
    loop {
        reset.set_high();
        Timer::after_millis(2000).await;
        reset.set_low();
        Timer::after_millis(2000).await;
    }
}


#[embassy_executor::task]
async fn temp_manager(mut i2c: I2c<'static, I2C2>) {
    loop {
        Timer::after_millis(5000).await;
/*

        for i in 0..2 {
            let mut data = [0u8; 2];
            if let Err(e) = i2c.blocking_read(0x48 + i, &mut data) {
                error!("i2c error: {:?}", e);
                continue;
            }

            let mut temp_data = ((data[0] as u16) << 4) | ((data[1] as u16) >> 4);

            if temp_data > 2047 {
                temp_data -= 4096
            }

            info!("read temp{}: {}", i + 1, temp_data);
        }
*/
    }
}
