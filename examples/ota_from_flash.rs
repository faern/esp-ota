extern crate esp_ota;
use log::*;

const FIRMWARE_ID: &str = "alpha";

const NEW_APP: &[u8] = include_bytes!("../beta.bin");

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("FIRMWARE ID : {}", FIRMWARE_ID);
    if FIRMWARE_ID == "alpha" {
        let mut ota = esp_ota::OtaUpdate::begin().unwrap();
        // write app to flash

        for app_chunk in NEW_APP.chunks(4096) {
            if let Err(err) = ota.write(app_chunk) {
                error!("Failed to write chunk");
                break;
            }
        }
        //validate the written app
        match ota.finalize() {
            Err(x) => {
                error!("Failed to validate image.");
                ()
            }
            Ok(mut x) => {
                x.set_as_boot_partition().unwrap();
                x.restart();
            }
        };
    }
}
