#![no_std]
#![no_main]

use core::option::Option::Some;
use defmt::{panic, *};
use defmt_rtt as _; // global logger
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, OutputType, Pull, Speed};
use embassy_stm32::i2c;
use embassy_stm32::i2c::I2c;
use embassy_stm32::rcc::*;
use embassy_stm32::time::{khz, Hertz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel as PWMChannel;
use embassy_stm32::usart::BufferedUart;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{bind_interrupts, peripherals, usart, usb};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use embedded_io_async::Read;
use static_cell::StaticCell;

use embassy_stm32::timer::low_level::OutputPolarity;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use embedded_io_async::Write;
use futures::future::join4;
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
    USART1 => usart::BufferedInterruptHandler<peripherals::USART1>;
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;

});
use embassy_stm32::peripherals::*;

#[derive(PartialEq)]
enum ResetManagerCommand {
    Reset,
    Shutdown,
}

static RESET_MANAGER_SIGNAL: Signal<CriticalSectionRawMutex, ResetManagerCommand> = Signal::new();

static PGOOD: Mutex<ThreadModeRawMutex, bool> = Mutex::new(false);

static TEMP1: Mutex<ThreadModeRawMutex, u16> = Mutex::new(0u16);
static TEMP2: Mutex<ThreadModeRawMutex, u16> = Mutex::new(0u16);

struct PWMControl {
    pwm1_value: u16,
    pwm2_value: u16,
}

static PWM_CTRL_CHANNEL: Channel<ThreadModeRawMutex, PWMControl, 1> = Channel::new();

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
    config.rcc.sys = Sysclk::PLL1_R;
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
    config.product = Some("Qaxe");
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
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state_usb_asic = State::new();
    let mut state_usb_ctrl = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    let class_usb_asic = CdcAcmClass::new(&mut builder, &mut state_usb_asic, 64);
    let (mut sender, mut receiver) = class_usb_asic.split();

    let mut class_usb_ctrl = CdcAcmClass::new(&mut builder, &mut state_usb_ctrl, 64);

    // Build the builder.
    let mut usb = builder.build();

    let mut config = usart::Config::default();
    config.baudrate = 115200;

    static TX_BUF: StaticCell<[u8; TX_BUF_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; TX_BUF_SIZE])[..];
    static RX_BUF: StaticCell<[u8; RX_BUF_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; RX_BUF_SIZE])[..];

    let usart = BufferedUart::new(p.USART1, Irqs, p.PA10, p.PA9, tx_buf, rx_buf, config).unwrap();
    let (mut tx_ctrl, mut rx_ctrl) = usart.split();

    // Run the USB device.
    let usb_fut = usb.run();

    let run_1v2 = Output::new(p.PA2, Level::Low, Speed::Low);
    let pgood_1v2 = Input::new(p.PA3, Pull::None);
    let pgood_led = Output::new(p.PA5, Level::High, Speed::Low);
    let mut activity_led = Output::new(p.PA4, Level::High, Speed::Low);

    // init ldo_en with low
    let ldo_en = Output::new(p.PA7, Level::Low, Speed::Low);

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
    pwm1.set_duty(PWMChannel::Ch1, pwm1.get_max_duty());
    pwm1.set_duty(PWMChannel::Ch2, pwm1.get_max_duty());
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
        p.DMA1_CH4,
        p.DMA1_CH5,
        Hertz(100_000),
        i2c_config, /*Default::default()*/
    );

    unwrap!(spawner.spawn(reset_manager(run_1v2, reset, ldo_en)));
    unwrap!(spawner.spawn(power_good_task(pgood_1v2, pgood_led)));
    unwrap!(spawner.spawn(pwm_manager(pwm1)));
    unwrap!(spawner.spawn(temp_manager(i2c)));

    let protobuf_rpc_fut = async {
        loop {
            class_usb_ctrl.wait_connection().await;
            info!("Connected");
            let _ = json_rpc(&mut class_usb_ctrl).await;
            info!("Disconnected");
        }
    };

    let relay_receiver_fut = async {
        loop {
            let mut usb_buf = [0; 64];
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

                if let Err(e) = tx_ctrl.write_all(&usb_buf[..usb_read]).await {
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

            let mut toggle = 0;
            let mut num_bytes = 0;
            let mut received = [0u8;11];
            let preample = [0xaa, 0x55];
            loop {
                let mut byte = [0u8;1];
                match rx_ctrl.read_exact(&mut byte).await {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Error reading from USART: {:?}", e);
                    }
                };

                received[num_bytes] = byte[0];

                // try to sync on serial data
                match num_bytes {
                    0 | 1 => // wait for 0xaa 0x55
                        if received[num_bytes] != preample[num_bytes] {
                            debug!("unexpected start of serial data, trying to resync ...");
                            num_bytes = 0;
                            continue;
                        },
                    _ => {},
                };

                num_bytes += 1;

                if num_bytes != 11 {
                    continue;
                }
                num_bytes = 0;

                // toggle led with each response received
                toggle = 1 - toggle;
                match toggle {
                    0 => activity_led.set_high(),
                    1 => activity_led.set_low(),
                    _ => {}
                };

                debug!("USART -> USB: {:x}", &received[..]);
                if let Err(e) = sender.write_packet(&received[..]).await {
                    error!("Error writing to USB: {:?}", e);
                    break;
                }
            }
        }
    };

    let _ = join4(
        usb_fut,
        protobuf_rpc_fut,
        relay_receiver_fut,
        relay_sender_fut,
    )
    .await;
}

#[embassy_executor::task]
async fn power_good_task(pgood_1v2: Input<'static>, mut pgood_led: Output<'static>) {
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
async fn reset_manager(mut run_1v2: Output<'static>, mut reset: Output<'static>, mut ldo_en: Output<'static>) {
    loop {
        let signal = RESET_MANAGER_SIGNAL.wait().await;

        match signal {
            ResetManagerCommand::Reset => {
                info!("reset triggered!");
                // switch off all LDOs and assert reset
                run_1v2.set_low();
                ldo_en.set_low();
                reset.set_high();
                Timer::after_millis(250).await;

                // switch on LDOs
                ldo_en.set_high();
                Timer::after_millis(100).await;

                // switch on buck
                run_1v2.set_high();
                Timer::after_millis(250).await;

                // deassert reset
                reset.set_low();
                Timer::after_millis(250).await;
            }
            ResetManagerCommand::Shutdown => {
                info!("shutdown triggered!");
                // switch off all LDOs and assert reset
                run_1v2.set_low();
                ldo_en.set_low();
                reset.set_high();
            }
        }
    }
}

#[embassy_executor::task]
async fn pwm_manager(mut pwm1: SimplePwm<'static, TIM2>) {
    loop {
        let pwm = PWM_CTRL_CHANNEL.receive().await;
        let max_duty = pwm1.get_max_duty() as u32;
        for i in 0..2 {
            let (channel, value) = match i {
                0 => (PWMChannel::Ch1, pwm.pwm1_value),
                1 => (PWMChannel::Ch2, pwm.pwm2_value),
                _ => (PWMChannel::Ch3, 0),
            };
            let duty = max_duty * value as u32 / 100;
            info!("pwm{}: {}, max: {}", i, duty, max_duty);
            pwm1.set_duty(
                channel,
                if duty <= max_duty {
                    duty
                } else {
                    max_duty
                },
            );
        }

        Timer::after_millis(500).await;
    }
}
struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

enum Errors {
    None = 0,
    InvalidCommand = 1,
    ErrorDeserializingRequest = 2,
    ErrorSerializingResponse = 3,
    ErrorDeserializingRequestData = 4,
    ErrorSerializingResponseData = 5,
}

impl Errors {
    fn to_string(error: &Errors) -> &'static str {
        match error {
            Errors::InvalidCommand => "invalid command",
            Errors::ErrorDeserializingRequest => "error deserializing request",
            Errors::ErrorSerializingResponse => "error serializing response",
            Errors::ErrorDeserializingRequestData => "error deserializing request data",
            Errors::ErrorSerializingResponseData => "error serializing response data",
            _ => "unknown error",
        }
    }
}
enum Commands {
    Nop = 0,
    Control = 1,
    Status = 2,
    Reset = 3,
    Shutdown = 4,
}

impl Commands {
    fn from_i32(value: i32) -> Option<Commands> {
        match value {
            0 => Some(Commands::Nop),
            1 => Some(Commands::Control),
            2 => Some(Commands::Status),
            3 => Some(Commands::Reset),
            4 => Some(Commands::Shutdown),
            _ => None,
        }
    }
}

impl QResponse<'_> {
    fn default() -> QResponse<'static> {
        QResponse {
            id: 0,
            error: 0,
            data: Cow::Borrowed(&[0u8]),
        }
    }
}

// The response_bytes should be a mutable slice of u8, not a slice of a mutable slice.
async fn process_request<'a>(
    request: &QRequest<'_>,
    response: &mut QResponse<'_>,
) -> Result<usize, Errors> {
    let mut response_data = [0u8; 32];
    let mut response_len = 0;
    let error = Errors::None as i32;

    let op = Commands::from_i32(request.op);
    if op.is_none() {
        return Err(Errors::InvalidCommand);
    }

    match op.unwrap() {
        Commands::Nop => {
            // nop
        }
        Commands::Control => {
            let cmd: QControl = quick_protobuf::deserialize_from_slice(&request.data)
                .map_err(|_| Errors::ErrorDeserializingRequestData)?;

            info!(
                "received ctrl command with parameters state_1v2: {}, pwm1: {}, pwm2: {}",
                cmd.state_1v2, cmd.pwm1, cmd.pwm2
            );

            PWM_CTRL_CHANNEL
                .send(PWMControl {
                    pwm1_value: cmd.pwm1 as u16,
                    pwm2_value: cmd.pwm2 as u16,
                })
                .await;
        }
        Commands::Status => {
            info!("status");
            // get current power state
            let pgood_state = PGOOD.lock().await;

            let temp1 = TEMP1.lock().await;
            let temp1_data = *temp1;

            let temp2 = TEMP2.lock().await;
            let temp2_data = *temp2;

            let state = QState {
                pgood_1v2: *pgood_state as i32,
                temp1: temp1_data as i32,
                temp2: temp2_data as i32,
            };
            drop(pgood_state);
            drop(temp1);
            drop(temp2);

            response_len = state.get_size() + 1 /* varint */;
            debug!("response-len: {}", response_len);
            quick_protobuf::serialize_into_slice(&state, &mut response_data[..])
                .map_err(|_| Errors::ErrorSerializingResponseData)?;
        }
        Commands::Reset => RESET_MANAGER_SIGNAL.signal(ResetManagerCommand::Reset),
        Commands::Shutdown => RESET_MANAGER_SIGNAL.signal(ResetManagerCommand::Shutdown),
    };

    response.id = request.id;
    response.error = error;
    response.data = Cow::Owned(response_data[..response_len].to_vec());
    debug!(
        "response.id: {}, response.error:{}, response.data: {:?}",
        response.id,
        response.error,
        response_data[..response_len]
    );
    Ok(response_len)
}

async fn json_rpc<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut request_bytes = [0u8; 64];
    let mut response_bytes = [0u8; 64];

    loop {
        let n = class.read_packet(&mut request_bytes).await?;

        let mut response = QResponse::default();

        let request = match quick_protobuf::deserialize_from_slice(&request_bytes[..n]) {
            Ok(req) => Some(req),
            Err(_) => {
                error!("{}", Errors::to_string(&Errors::ErrorDeserializingRequest));
                response = QResponse::default();
                response.error = Errors::ErrorDeserializingRequest as i32;
                None
            }
        };

        // if request is some then we can process the request
        if request.is_some() {
            if let Err(e) = process_request(&request.unwrap(), &mut response).await {
                error!("{}", Errors::to_string(&e));
                response = QResponse::default();
                response.error = e as i32;
            }
        }

        let serialized_len = response.get_size() + 1 /* varint */;
        if quick_protobuf::serialize_into_slice(&response, &mut response_bytes).is_err() {
            error!("{}", Errors::to_string(&Errors::ErrorSerializingResponse));
            continue;
        }

        class
            .write_packet(&response_bytes[..serialized_len])
            .await?;
    }
}

#[embassy_executor::task]
async fn temp_manager(mut i2c: I2c<'static, I2C2, DMA1_CH4, DMA1_CH5>) {
    loop {
        Timer::after_millis(5000).await;

        for i in 0..2 {
            let mut data = [0u8; 2];
            if let Err(e) = i2c.read(0x48 + i, &mut data).await {
                error!("i2c error: {:?}", e);
                continue;
            }

            let mut temp_data = ((data[0] as u16) << 4) | ((data[1] as u16) >> 4);

            if temp_data > 2047 {
                temp_data -= 4096
            }

            info!("read temp{}: {}", i + 1, temp_data);

            if i == 0 {
                let mut temp1 = TEMP1.lock().await;
                *temp1 = temp_data;
            } else {
                let mut temp2 = TEMP2.lock().await;
                *temp2 = temp_data;
            }
        }

    }
}
