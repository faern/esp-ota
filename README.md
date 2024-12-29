# esp-ota

This crate allows easy [OTA updates] for ESP32 chips using only safe Rust. The crate is completely
transport agnostic, meaning it does not deal with how you transfer the new app image to the
ESP.

[OTA updates]: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/ota.html

## Comparison with `esp-idf-svc`

After writing this library I learned that the `esp-idf-svc` crate [has experimental support for
OTA updates]. It's "hidden" in a module that is not visible in the generated documentation,
due to being behind a feature that is not enabled by default, nor when generating the docs.

[has experimental support for OTA updates]: https://github.com/esp-rs/esp-idf-svc/blob/34dfee2a4f08692f415e965e4cd117034e6be56e/src/ota.rs

## Usage

This section will explain how to use `esp-ota` in an application to write a downloaded app image
to the flash and boot from it.

### Partition table

The chip must have at least two `app, ota_X` partitions, so it can boot from one
and write the OTA update to the other one. And a `data, ota` partition to store
information about which partition the bootloader should boot from.

Create a file called `partitions.csv`. You can read more about [ESP partition tables]
on espressifs website. But here is a fairly default example that works on a 4M flash:
```csv
# Name,   Type, SubType, Offset,  Size, Flags
nvs,      data, nvs,     0x9000,  0x4000,
otadata,  data, ota,     0xd000,  0x2000,
phy_init, data, phy,     0xf000,  0x1000,
ota_0,    app,  ota_0,   0x10000, 0x180000,
ota_1,    app,  ota_1,   0x190000, 0x180000,
```

And tell espflash to use it in `.cargo/config.toml` similar to the following according to your target. Read more in the [cargo espflash documentation](https://github.com/esp-rs/espflash/blob/main/cargo-espflash/README.md#bootloader-and-partition-table).
```toml
[build]
target = "xtensa-esp32s3-espidf"

[target.xtensa-esp32s3-espidf]
linker = "ldproxy"
runner = "espflash flash --monitor --partition-table=partitions.csv"
```



[cargo espflash documentation]: https://github.com/esp-rs/espflash/blob/master/cargo-espflash/README.md#package-metadata
[ESP partition tables]: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html

### App image format

The app that you want to flash must have the correct format. It should not be the ELF executable
produced by a regular `cargo build` in an ESP project, but rather the ESP32 specific
[app image format].

Both [`esptool.py elf2image`] and `espflash save-image` can be used to convert the binary:
```
# With espflash 2.x (recommended way):
$ espflash save-image --chip esp32c3 target/release/my-app my-app.bin

# With older espflash <=1.7:
$ espflash save-image ESP32-C3 target/release/my-app my-app.bin

# With the not-Rust-specific esptool:
$ esptool.py --chip ESP32-C3 elf2image --output my-app.bin target/release/my-app
```

[app image format]: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/app_image_format.html
[`esptool.py elf2image`]: https://docs.espressif.com/projects/esptool/en/latest/esp32/esptool/basic-commands.html#convert-elf-to-binary-elf2image

### Code

To flash a new app to the next partition on the flash, include code similar to this:

```rust
// This is a very unrealistic example. You usually don't store the new app in the
// old app. Instead you obtain it by downloading it from somewhere or similar.
const NEW_APP: &[u8] = include_bytes!("../my-app.bin");

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
```

And if [the rollback feature] is enabled, you need to validate that the new app works as intended,
or perform a rollback. Read more in the espressif documentation on [app rollback].

[the rollback feature]: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/kconfig.html#config-bootloader-app-rollback-enable
[app rollback]: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/ota.html#app-rollback

```rust
fn main() {
    if is_working_as_intended() {
        esp_ota::mark_app_valid();
    } else {
        esp_ota::rollback_and_reboot().expect("Failed to roll back to working app");
    }
}
```
