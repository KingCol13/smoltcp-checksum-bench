//! # Pico USB Serial Example
//!
//! Creates a USB Serial device on a Pico board, with the USB driver running in
//! the main thread.
//!
//! This will create a USB Serial device echoing anything it receives. Incoming
//! ASCII characters are converted to upercase, so you can tell it is working
//! and not just local-echo!
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Alias for our HAL crate
use rp235x_hal as hal;

// Some things we need
use core::fmt::Write;
use heapless::String;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
/// Adjust if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

fn time_checksum<F, D: hal::timer::TimerDevice>(
    checksum_impl: &F,
    data: &[u8],
    timer: hal::Timer<D>,
) -> u32
where
    F: Fn(&[u8]) -> u16,
{
    let start = timer.get_counter_low();
    let sum = checksum_impl(core::hint::black_box(&data));
    let end = timer.get_counter_low();

    end.wrapping_sub(start)
}

/// Entry point to our bare-metal application.
///
/// The `#[hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
///
/// The function configures the rp235x peripherals, then writes to the UART in
/// an infinite loop.
#[hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USB,
        pac.USB_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")])
        .unwrap()
        .max_packet_size_0(64)
        .unwrap()
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    // time some checksums
    use rand::prelude::*;
    let mut small_rng = rand::rngs::SmallRng::seed_from_u64(123456);
    let mut data = [0u8; 1024];
    small_rng.fill_bytes(&mut data);

    let time_original = time_checksum(&checksum::checksum_original, &data, timer);
    let time_indexed = time_checksum(&checksum::checksum_indexed, &data, timer);
    let time_chunks_exact_no_bigchunk = time_checksum(&checksum::checksum_chunks_exact_no_bigchunk, &data, timer);
    let time_chunks_exact = time_checksum(&checksum::checksum_chunks_exact, &data, timer);
    let time_sliced_ne = time_checksum(&checksum::checksum_sliced_ne, &data, timer);
    let time_sliced_ne_sep = time_checksum(&checksum::checksum_sliced_ne_sep, &data, timer);
    let time_sliced_ne_sep_unroll = time_checksum(&checksum::checksum_sliced_ne_sep_unroll, &data, timer);
    let time_chunks_ne_sep = time_checksum(&checksum::checksum_chunks_ne_sep, &data, timer);
    let time_sliced_ne_u16 = time_checksum(&checksum::checksum_sliced_ne_u16, &data, timer);
    let time_sliced_ne_u16_unroll = time_checksum(&checksum::checksum_sliced_ne_u16_unroll, &data, timer);
    let time_sliced_ne_u16_unroll_same = time_checksum(&checksum::checksum_sliced_ne_u16_unroll_same, &data, timer);
    let time_sliced_ne_u16_double_unroll_same = time_checksum(&checksum::checksum_sliced_ne_u16_double_unroll_same, &data, timer);
    let time_chunks_ne_u16 = time_checksum(&checksum::checksum_chunks_ne_u16, &data, timer);
    let time_chunks_ne_u16_unroll = time_checksum(&checksum::checksum_chunks_ne_u16_unroll, &data, timer);
    let time_full_indexed = time_checksum(&checksum::checksum_full_indexed, &data, timer);
    let time_muck_chunks_unroll = time_checksum(&checksum::checksum_muck_chunks_unroll, &data, timer);
    let time_into_chunks_unroll =
        time_checksum(&checksum::checksum_into_chunks_unroll, &data, timer);
    let time_wide = time_checksum(&checksum::checksum_wide, &data, timer);
    let time_wide_u16 = time_checksum(&checksum::checksum_wide_u16, &data, timer);

    let mut said_hello = false;
    loop {
        // A welcome message at the beginning
        if !said_hello && timer.get_counter().ticks() >= 2_000_000 {
            said_hello = true;
            let _ = serial.write(b"Hello, World!\r\n");

            let time = timer.get_counter().ticks();
            let mut text: String<64> = String::new();
            writeln!(&mut text, "Current timer ticks: {time}").unwrap();

            // This only works reliably because the number of bytes written to
            // the serial port is smaller than the buffers available to the USB
            // peripheral. In general, the return value should be handled, so that
            // bytes not transferred yet don't get lost.
            let _ = serial.write(text.as_bytes());
        }

        // Check for new data
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(count) => {
                    // Convert to upper case
                    let mut string = heapless::String::<1024>::new();
                    write!(
                        &mut string,
                        "time_original: {}\r\n\
                        time_indexed: {}\r\n\
                        time_chunks_exact: {}\r\n\
                        time_chunks_exact_no_bigchunk: {}\r\n\
                        time_sliced_ne: {}\r\n\
                        time_sliced_ne_sep: {}\r\n\
                        time_sliced_ne_sep_unroll: {}\r\n\
                        time_chunks_ne_sep: {}\r\n\
                        time_sliced_ne_u16: {}\r\n\
                        time_sliced_ne_u16_unroll: {}\r\n\
                        time_sliced_ne_u16_unroll_same: {}\r\n\
                        time_sliced_ne_u16_double_unroll_same: {}\r\n\
                        time_chunks_ne_u16: {}\r\n\
                        time_chunks_ne_u16_unroll: {}\r\n\
                        time_full_indexed: {}\r\n\
                        time_muck_chunks_unroll: {}\r\n\
                        time_into_chunks_unroll: {}\r\n\
                        time_wide: {}\r\n\
                        time_wide_u16: {}\r\n",
                        time_original,
                        time_indexed,
                        time_chunks_exact,
                        time_chunks_exact_no_bigchunk,
                        time_sliced_ne,
                        time_sliced_ne_sep,
                        time_sliced_ne_sep_unroll,
                        time_chunks_ne_sep,
                        time_sliced_ne_u16,
                        time_sliced_ne_u16_unroll,
                        time_sliced_ne_u16_unroll_same,
                        time_sliced_ne_u16_double_unroll_same,
                        time_chunks_ne_u16,
                        time_chunks_ne_u16_unroll,
                        time_full_indexed,
                        time_muck_chunks_unroll,
                        time_into_chunks_unroll,
                        time_wide,
                        time_wide_u16,
                    );
                    use usbd_serial::embedded_io::Write;
                    serial.write_all(&string.as_bytes());
                }
            }
        }
    }
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"USB Serial Example"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];

// End of file
