#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::{main, Spawner};
use panic_probe as _;

#[main]
async fn main(_s: Spawner) {
    let _p = embassy_rp::init(<_>::default());
    info!("init");
}
