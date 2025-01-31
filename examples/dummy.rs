// This is a very unrealistic example. You usually don't store the new app in the
// old app. Instead you obtain it by downloading it from somewhere.
// This also will not work at all. The flashing will fail saying
// this is not a valid firmware. But it works to see if the library compiles,
// and can be flashed to a chip.
const NEW_APP: &[u8] = &[0, 1, 2];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    // Finds the next suitable OTA partition and erases it
    let mut ota = esp_ota::OtaUpdate::begin()?;

    // Write the app to flash. Normally you would download
    // the app and call `ota.write` every time you have obtained
    // a part of the app image. This example is not realistic,
    // since it has the entire new app bundled.
    for app_chunk in NEW_APP.chunks(4096) {
        ota.write(app_chunk)?;
    }

    // Performs validation of the newly written app image and completes the OTA update.
    let mut completed_ota = ota.finalize()?;

    // Sets the newly written to partition as the next partition to boot from.
    completed_ota.set_as_boot_partition()?;
    // Restarts the CPU, booting into the newly written app.
    completed_ota.restart();
}
