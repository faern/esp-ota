extern crate esp_ota;
use log::*;

const FIRMWARE_ID: &str = "beta";

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("FIRMWARE ID : {}", FIRMWARE_ID);

    if FIRMWARE_ID == "beta" {
        esp_ota::mark_app_valid();
    }
}
