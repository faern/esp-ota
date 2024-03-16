# Examples

The directory contains simple examples for OTA updates

## beta

This is the example app that will replace the running OTA app. Typically, any new image should contain OTA update logic.
This app should be built before any of the ota apps can use it. 
It is better to build in release mode to minify the application. 

Build: 
```
RUSTFLAGS="-C strip=symbols -C debuginfo=0" cargo build --example beta --release
espflash save-image --chip <mcu_target> ./target/<mcu_target>/release/examples/beta ./beta.bin
```

## ota_from_flash

The app contains the update in it's flash memory at compile time. The `beta.bin` image in the ESP32 app image format must be available at compile time.

Build:
```
cargo build --example ota_from_flash --release
```
Flash and monitor:
```
espflash flash ./target/<mcu_target>/release/examples/ota_from_flash --partition_table ./examples/partitions_4MB.csv --monitor 
```

## partitions_4MB

A partition table that splits the two OTA partitions evenly.

## partitions_factory.csv

The default partition table shipped with the ESP32. _Not suitable for OTA._