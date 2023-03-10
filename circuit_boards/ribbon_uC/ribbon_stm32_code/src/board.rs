use stm32l4xx_hal::{
    adc::{DmaMode, SampleTime, Sequence, ADC},
    delay::Delay,
    device::SPI1,
    dma::{dma1, CircBuffer, Transfer},
    gpio::{Alternate, Analog, Input, Output, Pin, PullUp, PushPull, H8, L8},
    hal::{
        blocking::spi::transfer,
        spi::{Mode, Phase, Polarity},
    },
    pac::{ADC1, DMA1, USART1},
    prelude::*,
    serial,
    spi::Spi,
};

use nb::block;

/// Pins which may be read by the ADC are represented here
#[derive(Clone, Copy)]
pub enum AdcPin {
    PA0 = 0,
    PA1 = 1,
    PA2 = 2,
    PA3 = 3,
    PA4 = 4,
}

/// Channels of the onboard DAC
#[derive(Clone, Copy)]
pub enum Dac8164Channel {
    A = 0b000,
    B = 0b010,
    C = 0b100,
    D = 0b110,
}

/// Valid states of a 3-way switch
#[derive(Clone, Copy)]
pub enum Switch3wayState {
    UP,
    MIDDLE,
    DOWN,
}

/// The physical board structure is represented here
pub struct Board {
    // USART for MIDI
    tx: serial::Tx<USART1>,
    rx: serial::Rx<USART1>,

    // SPI for DAC
    spi: Spi<
        SPI1,
        (
            Pin<Alternate<PushPull, 5>, L8, 'B', 3>, // SCK
            Pin<Alternate<PushPull, 5>, L8, 'B', 4>, // SDI
            Pin<Alternate<PushPull, 5>, L8, 'B', 5>, // SDO
        ),
    >,
    nss: Pin<Output<PushPull>, H8, 'A', 15>,

    // general purpose delay
    delay: Delay,

    // ADC for reading the ribbon and analog controls
    adc: ADC,
    adc_pins: (
        Pin<Analog, L8, 'A', 0>,
        Pin<Analog, L8, 'A', 1>,
        Pin<Analog, L8, 'A', 2>,
        Pin<Analog, L8, 'A', 3>,
        Pin<Analog, L8, 'A', 4>,
    ),

    // 2 pins for the 3-position QUANTIZE MODE switch
    mode_switch: (
        Pin<Input<PullUp>, L8, 'B', 6>,
        Pin<Input<PullUp>, L8, 'B', 7>,
    ),

    // ribbon gate
    gate_pin: Pin<Output<PushPull>, L8, 'A', 5>,
}

impl Board {
    /// `Board::init()` is the board structure with all peripherals initialized.
    pub fn init() -> Self {
        ////////////////////////////////////////////////////////////////////////
        //
        // general peripheral housekeeping, core peripherals and clocks
        //
        ////////////////////////////////////////////////////////////////////////
        let cp = cortex_m::Peripherals::take().unwrap();
        let dp = stm32l4xx_hal::pac::Peripherals::take().unwrap();
        let mut flash = dp.FLASH.constrain();
        let mut rcc = dp.RCC.constrain();
        let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

        let clocks = rcc
            .cfgr
            .sysclk(80.MHz())
            .pclk1(80.MHz())
            .pclk2(80.MHz())
            .freeze(&mut flash.acr, &mut pwr);

        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);

        let mut delay = Delay::new(cp.SYST, clocks);

        ////////////////////////////////////////////////////////////////////////
        //
        // ADC
        //
        ////////////////////////////////////////////////////////////////////////

        let mut adc = ADC::new(
            dp.ADC1,
            dp.ADC_COMMON,
            &mut rcc.ahb2,
            &mut rcc.ccipr,
            &mut delay,
        );

        adc.set_sample_time(SampleTime::Cycles92_5);

        // configure hardware oversampler for 16 bit resolution
        unsafe {
            (*ADC1::ptr()).cfgr2.modify(|_, w| {
                w.ovsr()
                    .bits(0b100) // oversample 32x
                    .ovss()
                    .bits(0b0001) // shift right by 1
                    .rovse()
                    .set_bit()
            });
        }

        let adc_pins = (
            gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa.pa1.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa.pa2.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa.pa3.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
            gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),
        );

        ////////////////////////////////////////////////////////////////////////
        //
        // USART
        //
        ////////////////////////////////////////////////////////////////////////
        let tx_pin = gpioa
            .pa9
            .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);
        let rx_pin =
            gpioa
                .pa10
                .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);

        let usart = serial::Serial::usart1(
            dp.USART1,
            (tx_pin, rx_pin),
            serial::Config::default().baudrate(MIDI_BAUD_RATE.bps()),
            clocks,
            &mut rcc.apb2,
        );
        let (tx, rx) = usart.split();

        ////////////////////////////////////////////////////////////////////////
        //
        // SPI
        //
        ////////////////////////////////////////////////////////////////////////
        let sck = gpiob
            .pb3
            .into_alternate(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let sdi = gpiob
            .pb4
            .into_alternate(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let sdo = gpiob
            .pb5
            .into_alternate(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

        let nss = gpioa.pa15.into_push_pull_output_in_state(
            &mut gpioa.moder,
            &mut gpioa.otyper,
            PinState::High,
        );

        let spi = Spi::spi1(
            dp.SPI1,
            (sck, sdi, sdo),
            Mode {
                phase: Phase::CaptureOnFirstTransition,
                polarity: Polarity::IdleHigh,
            },
            20_u32.MHz(),
            clocks,
            &mut rcc.apb2,
        );

        ////////////////////////////////////////////////////////////////////////
        //
        // 3-way Mode switch
        //
        ////////////////////////////////////////////////////////////////////////
        let mode_switch = (
            gpiob
                .pb6
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            gpiob
                .pb7
                .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
        );

        ////////////////////////////////////////////////////////////////////////
        //
        // Gate pin
        //
        ////////////////////////////////////////////////////////////////////////
        let gate_pin = gpioa
            .pa5
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

        Self {
            tx,
            rx,
            spi,
            nss,
            delay,
            adc,
            adc_pins,
            mode_switch,
            gate_pin,
        }
    }

    /// `board.read_adc(p)` is the digitized analog value on pin `p` in the range `[0.0, +1.0]`
    pub fn read_adc(&mut self, pin: AdcPin) -> f32 {
        match pin {
            AdcPin::PA0 => adc_fs_to_normalized_fl(self.adc.read(&mut self.adc_pins.0).unwrap()),
            AdcPin::PA1 => adc_fs_to_normalized_fl(self.adc.read(&mut self.adc_pins.1).unwrap()),
            AdcPin::PA2 => adc_fs_to_normalized_fl(self.adc.read(&mut self.adc_pins.2).unwrap()),
            AdcPin::PA3 => adc_fs_to_normalized_fl(self.adc.read(&mut self.adc_pins.3).unwrap()),
            AdcPin::PA4 => adc_fs_to_normalized_fl(self.adc.read(&mut self.adc_pins.4).unwrap()),
        }
    }

    /// `board.dac8164_write(v, c)` writes the normalized value `v` in the range `[0.0, +1.0]` to channel `c` of the onboard DAC.
    pub fn dac8164_write(&mut self, val: f32, channel: Dac8164Channel) {
        let val_u14 = normalized_fl_to_dac_fs(val);

        // move the value out of DB0 and DB1
        let val_u14 = val_u14 << 2;
        let low_byte = (val_u14 & 0xFF) as u8;
        let mid_byte = (val_u14 >> 8) as u8;
        let high_byte = channel as u8 | (1 << 4); // set LDO for immediate update

        self.spi_write(&[high_byte, mid_byte, low_byte]);
    }

    /// `board.read_mode_switch()` is the enumerated state of the 3-way mode switch.
    pub fn read_mode_switch(&self) -> Switch3wayState {
        // The physical switch on the PCB is a SPDT on-off-on switch which grounds
        // either PB6, PB7, or neither pins depending on the position.
        match (self.mode_switch.0.is_low(), self.mode_switch.1.is_low()) {
            (false, true) => Switch3wayState::UP,
            (false, false) => Switch3wayState::MIDDLE,
            _ => Switch3wayState::DOWN, // should only happen with (true, false) but catch unlikely (true, true) as well
        }
    }

    /// `board.serial_write(b)` writes the byte `b` via the USART in blocking fashion.
    pub fn serial_write(&mut self, byte: u8) {
        block!(self.tx.write(byte)).ok();
    }

    /// `board.serial_read()` is the optional byte read from the USART.
    pub fn serial_read(&mut self) -> Option<u8> {
        self.rx.is_idle(false); // what is happening here?

        let res = self.rx.read();

        match res {
            Ok(byte) => Some(byte),
            _ => None,
        }
    }

    /// `board.spi_write(words)` writes the words via SPI.
    fn spi_write(&mut self, words: &[u8]) {
        self.nss.set_low();
        self.spi.write(words).unwrap();
        self.nss.set_high();
    }

    /// `board.sleep_ms(ms)` causes the board to busy-wait for the `ms` milliseconds
    pub fn delay_ms(&mut self, ms: u32) {
        self.delay.delay_ms(ms);
    }

    /// `board.set_gate(val)` sets the state of the gate pin to `val`.
    pub fn set_gate(&mut self, val: bool) {
        self.gate_pin.set_state(PinState::from(val));
    }
}

/// The maximum value that can be produced by the Analog to Digital Converters.
pub const ADC_MAX: u16 = 0xFFF0;

/// The maximum value that can be written to the onboard Digital to Analog Converter.
pub const DAC_MAX: u16 = (1 << 14) - 1;

/// The baud rate required for MIDI communication
pub const MIDI_BAUD_RATE: u32 = 31_250_u32;

/// `adc_fs_to_normalized_fl(v)` is the integer adc value with the full scale normalized to [0.0, +1.0]
///
/// If the input value would overflow the output range it is clamped.
fn adc_fs_to_normalized_fl(val: u16) -> f32 {
    let val = val.min(ADC_MAX); // don't need to clamp negative values, it's already unsigned

    (val as f32) / (ADC_MAX as f32)
}

/// `normalized_fl_to_dac_fs(v)` is the normalized [0.0, +1.0] value expanded to DAC full scale range.
///
/// If the input value would overflow the output range it is clamped.
fn normalized_fl_to_dac_fs(val: f32) -> u16 {
    let val = val.min(1.0_f32).max(0.0_f32);

    (val * DAC_MAX as f32) as u16
}
