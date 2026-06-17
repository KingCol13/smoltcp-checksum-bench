#![no_std]
#![no_main]

use defmt_rtt as _;
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;
// Alias for HAL crate
use rp235x_hal as hal;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// Entry point to our bare-metal application.
///
/// The `#[hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
///
/// The function configures the rp235x peripherals, then toggles a GPIO pin in
/// an infinite loop. If there is an LED connected to that pin, it will blink.
#[hal::entry]
fn main() -> ! {
    use cortex_m::peripheral::{DWT, Peripherals};
    use rand::prelude::*;

    let mut peripherals = Peripherals::take().unwrap();
    peripherals.DCB.enable_trace();
    peripherals.DWT.enable_cycle_counter();

    let mut small_rng = rand::rngs::SmallRng::seed_from_u64(123456);
    let mut rand = [0u8; 65536];
    small_rng.fill_bytes(&mut rand);

    defmt::info!("Starting test loop!");

    for i in 0..=16 {
        let len = 1 << i;
        let data = &rand[..len];

        let start = DWT::cycle_count();
        let sum_original = checksum::checksum_original(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_original = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_indexed = checksum::checksum_indexed(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_indexed = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_chunks_exact = checksum::checksum_chunks_exact(data);
        let end = DWT::cycle_count();
        let cycles_chunks_exact = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_chunks_exact_no_bigchunk =
            checksum::checksum_chunks_exact_no_bigchunk(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_chunks_exact_no_bigchunk = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_sliced_ne = checksum::checksum_sliced_ne(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_sliced_ne = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_sliced_ne_sep = checksum::checksum_sliced_ne_sep(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_sliced_ne_sep = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_sliced_ne_sep_unroll = checksum::checksum_sliced_ne_sep_unroll(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_sliced_ne_sep_unroll = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_chunks_ne_sep = checksum::checksum_chunks_ne_sep(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_chunks_ne_sep = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_sliced_ne_u16 = checksum::checksum_sliced_ne_u16(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_sliced_ne_u16 = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_chunks_ne_u16 = checksum::checksum_chunks_ne_u16(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_chunks_ne_u16 = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_chunks_ne_u16_unroll = checksum::checksum_chunks_ne_u16_unroll(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_chunks_ne_u16_unroll = end.wrapping_sub(start);

        let start = DWT::cycle_count();
        let sum_wide = checksum::checksum_wide(core::hint::black_box(data));
        let end = DWT::cycle_count();
        let cycles_wide = end.wrapping_sub(start);

        assert_eq!(sum_original, sum_indexed);
        assert_eq!(sum_original, sum_chunks_exact);
        assert_eq!(sum_original, sum_chunks_exact_no_bigchunk);
        assert_eq!(sum_original, sum_sliced_ne);
        assert_eq!(sum_original, sum_sliced_ne_sep);
        assert_eq!(sum_original, sum_sliced_ne_sep_unroll);
        assert_eq!(sum_original, sum_chunks_ne_sep);
        assert_eq!(sum_original, sum_sliced_ne_u16);
        assert_eq!(sum_original, sum_chunks_ne_u16);
        assert_eq!(sum_original, sum_chunks_ne_u16_unroll);
        assert_eq!(sum_original, sum_wide);

        defmt::info!(
            "len={} original={=u32} indexed={=u32} chunks_exact={=u32} chunks_exact_no_bigchunk={=u32} sliced_ne={=u32} sliced_ne_sep={=u32} sliced_ne_sep_unroll={=u32} chunks_ne_sep={=u32} sliced_ne_u16={=u32} chunks_ne_u16={=u32} chunks_ne_u16_unroll={=u32} wide={=u32}",
            len,
            cycles_original,
            cycles_indexed,
            cycles_chunks_exact,
            cycles_chunks_exact_no_bigchunk,
            cycles_sliced_ne,
            cycles_sliced_ne_sep,
            cycles_sliced_ne_sep_unroll,
            cycles_chunks_ne_sep,
            cycles_sliced_ne_u16,
            cycles_chunks_ne_u16,
            cycles_chunks_ne_u16_unroll,
            cycles_wide,
        );
    }

    loop {
        cortex_m::asm::wfi();
    }
}
