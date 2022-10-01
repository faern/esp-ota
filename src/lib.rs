#![doc = include_str!("../README.md")]

use core::fmt;
use core::mem;
use core::ptr;
use esp_idf_sys::{
    esp_ota_abort, esp_ota_begin, esp_ota_end, esp_ota_get_next_update_partition, esp_ota_handle_t,
    esp_ota_mark_app_invalid_rollback_and_reboot, esp_ota_mark_app_valid_cancel_rollback,
    esp_ota_set_boot_partition, esp_ota_write, esp_partition_t, esp_restart, ESP_ERR_FLASH_OP_FAIL,
    ESP_ERR_FLASH_OP_TIMEOUT, ESP_ERR_INVALID_ARG, ESP_ERR_INVALID_SIZE, ESP_ERR_INVALID_STATE,
    ESP_ERR_NOT_FOUND, ESP_ERR_NO_MEM, ESP_ERR_OTA_PARTITION_CONFLICT, ESP_ERR_OTA_ROLLBACK_FAILED,
    ESP_ERR_OTA_ROLLBACK_INVALID_STATE, ESP_ERR_OTA_SELECT_INFO_INVALID,
    ESP_ERR_OTA_VALIDATE_FAILED, ESP_FAIL, ESP_OK, OTA_SIZE_UNKNOWN,
};

pub type Result<T> = core::result::Result<T, Error>;

/// An error that can happen during ESP OTA operations.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub(crate) fn from_kind(kind: ErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the kind of error as an enum, that can be matched on.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// No suitable partition for writing OTA update to found.
    NoOtaPartition,
    /// Cannot allocate memory for OTA operation.
    AllocFailed,
    /// Rollback enabled, but the currently running application is still pending. The currently
    /// running application must confirm itself before downloading and flashing a new app.
    InvalidRollbackState,
    /// First byte of image contains invalid app image magic byte.
    InvalidMagicByte,
    /// Flash write operation timed out.
    FlashTimeout,
    /// Flash write operation failed.
    FlashFailed,
    /// OTA data partition has invalid contents.
    InvalidOtaPartitionData,
    /// The [`OtaUpdate`] handle was finalized before any app image was written to it.
    NothingWritten,
    /// OTA image is invalid (either not a valid app image, or - if secure boot is enabled - signature failed to verify.)
    InvalidImage,
    /// If flash encryption is enabled, this result indicates an internal error writing the final encrypted bytes to flash.
    WritingEncryptedFailed,
    /// The rollback failed.
    RollbackFailed,
    /// The rollback is not possible due to flash does not have any apps.
    RollbackFailedNoApps,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;
        match self {
            NoOtaPartition => "No suitable partition for writing OTA update to found",
            AllocFailed => "Cannot allocate memory for OTA operation",
            InvalidRollbackState => {
                "Rollback enabled, but the currently running application is still pending"
            }
            InvalidMagicByte => "First byte of image contains invalid app image magic byte",
            FlashTimeout => "Flash write operation timed out",
            FlashFailed => "Flash write operation failed",
            InvalidOtaPartitionData => "OTA data partition has invalid contents",
            NothingWritten => "OtaUpdate was never written to",
            InvalidImage => "OTA image is invalid",
            WritingEncryptedFailed => "Internal error writing the final encrypted bytes to flash",
            RollbackFailed => "The rollback failed",
            RollbackFailedNoApps => {
                "The rollback is not possible due to flash does not have any apps"
            }
        }
        .fmt(f)
    }
}

/// Represents an ongoing OTA update.
///
/// Dropping this object before calling [`finalize`](OtaUpdate::finalize) will abort the update.
#[derive(Debug)]
pub struct OtaUpdate {
    partition: *const esp_partition_t,
    ota_handle: esp_ota_handle_t,
}

impl OtaUpdate {
    /// Starts an OTA update to the next OTA compatible partition.
    ///
    /// Finds next partition round-robin, starting from the current running partition.
    /// The entire partition is erased.
    pub fn begin() -> Result<Self> {
        let partition = unsafe { esp_ota_get_next_update_partition(ptr::null()) };
        if partition.is_null() {
            return Err(Error::from_kind(ErrorKind::NoOtaPartition));
        }

        let mut ota_handle = 0;
        match unsafe { esp_ota_begin(partition, OTA_SIZE_UNKNOWN, &mut ota_handle) } {
            ESP_OK => Ok(()),
            ESP_ERR_INVALID_ARG => panic!("Invalid partition or out_handle"),
            ESP_ERR_NO_MEM => Err(Error::from_kind(ErrorKind::AllocFailed)),
            ESP_ERR_OTA_PARTITION_CONFLICT => Err(Error::from_kind(ErrorKind::NoOtaPartition)),
            ESP_ERR_NOT_FOUND => panic!("Partition argument not found in partition table"),
            ESP_ERR_OTA_SELECT_INFO_INVALID => {
                Err(Error::from_kind(ErrorKind::InvalidOtaPartitionData))
            }
            ESP_ERR_INVALID_SIZE => panic!("Partition doesn’t fit in configured flash size"),
            ESP_ERR_FLASH_OP_TIMEOUT => Err(Error::from_kind(ErrorKind::FlashTimeout)),
            ESP_ERR_FLASH_OP_FAIL => Err(Error::from_kind(ErrorKind::FlashFailed)),
            ESP_ERR_OTA_ROLLBACK_INVALID_STATE => {
                Err(Error::from_kind(ErrorKind::InvalidRollbackState))
            }
            code => panic!("Unexpected esp_ota_begin return code: {}", code),
        }?;

        Ok(Self {
            partition,
            ota_handle,
        })
    }

    /// Write app image data to partition.
    ///
    /// This method can be called multiple times as data is received during the OTA operation.
    /// Data is written sequentially to the partition.
    ///
    /// The format of the app image can be read about in the main README and crate documentation.
    pub fn write(&mut self, app_image_chunk: &[u8]) -> Result<()> {
        let chunk_ptr = app_image_chunk.as_ptr() as *const _;
        let chunk_len = u32::try_from(app_image_chunk.len()).expect("Too large firmware chunk");

        match unsafe { esp_ota_write(self.ota_handle, chunk_ptr, chunk_len) } {
            ESP_OK => Ok(()),
            ESP_ERR_INVALID_ARG => panic!("Invalid OTA handle"),
            ESP_ERR_OTA_VALIDATE_FAILED => Err(Error::from_kind(ErrorKind::InvalidMagicByte)),
            ESP_ERR_FLASH_OP_TIMEOUT => Err(Error::from_kind(ErrorKind::FlashTimeout)),
            ESP_ERR_FLASH_OP_FAIL => Err(Error::from_kind(ErrorKind::FlashFailed)),
            ESP_ERR_OTA_SELECT_INFO_INVALID => {
                Err(Error::from_kind(ErrorKind::InvalidOtaPartitionData))
            }
            code => panic!("Unexpected esp_ota_write return code: {code}"),
        }
    }

    /// Finish OTA update and validate newly written app image.
    ///
    /// Unless you also call [`set_as_boot_partition`](CompletedOtaUpdate::set_as_boot_partition) the new app will not
    /// start.
    pub fn finalize(self) -> Result<CompletedOtaUpdate> {
        match unsafe { esp_ota_end(self.ota_handle) } {
            ESP_OK => Ok(()),
            ESP_ERR_NOT_FOUND => panic!("Invalid OTA handle"),
            ESP_ERR_INVALID_ARG => Err(Error::from_kind(ErrorKind::NothingWritten)),
            ESP_ERR_OTA_VALIDATE_FAILED => Err(Error::from_kind(ErrorKind::InvalidImage)),
            ESP_ERR_INVALID_STATE => Err(Error::from_kind(ErrorKind::WritingEncryptedFailed)),
            code => panic!("Unexpected esp_ota_end return code: {code}"),
        }?;

        let partition = self.partition;
        mem::forget(self);

        Ok(CompletedOtaUpdate { partition })
    }

    /// Returns a raw pointer to the partition that the new app is/will be written to.
    pub fn raw_partition(&self) -> *const esp_partition_t {
        self.partition
    }
}

impl Drop for OtaUpdate {
    fn drop(&mut self) {
        #[cfg(feature = "log")]
        log::debug!("Aborting OTA update");

        let abort_result_code = unsafe { esp_ota_abort(self.ota_handle) };
        if abort_result_code != ESP_OK {
            #[cfg(feature = "log")]
            log::error!(
                "Aborting the OTA update returned an unexpected code: {}",
                abort_result_code
            )
        }
    }
}

pub struct CompletedOtaUpdate {
    partition: *const esp_partition_t,
}

impl CompletedOtaUpdate {
    /// Sets the boot partition to the newly flashed OTA partition.
    pub fn set_as_boot_partition(&mut self) -> Result<()> {
        match unsafe { esp_ota_set_boot_partition(self.partition) } {
            ESP_OK => Ok(()),
            ESP_ERR_INVALID_ARG => panic!("Invalid partition sent to esp_ota_set_boot_partition"),
            ESP_ERR_OTA_VALIDATE_FAILED => Err(Error::from_kind(ErrorKind::InvalidImage)),
            ESP_ERR_NOT_FOUND => panic!("OTA data partition not found"),
            ESP_ERR_FLASH_OP_TIMEOUT => Err(Error::from_kind(ErrorKind::FlashTimeout)),
            ESP_ERR_FLASH_OP_FAIL => Err(Error::from_kind(ErrorKind::FlashFailed)),
            code => panic!("Unexpected esp_ota_set_boot_partition code: {}", code),
        }
    }

    /// Restarts the CPU. If [`set_as_boot_partition`](CompletedOtaUpdate::set_as_boot_partition) was
    /// called and completed successfully, the CPU will boot into the newly written app.
    ///
    /// After successful restart, CPU reset reason will be SW_CPU_RESET. Peripherals
    /// (except for WiFi, BT, UART0, SPI1, and legacy timers) are not reset.
    pub fn restart(self) -> ! {
        unsafe { esp_restart() }
        unreachable!("esp_restart returned");
    }

    /// Returns a raw pointer to the partition that the new app was written to.
    pub fn raw_partition(&self) -> *const esp_partition_t {
        self.partition
    }
}

/// Call this function to indicate that the running app is working well.
///
/// Should be called (at least) the first time a new app starts up after
/// being flashed.
pub fn mark_app_valid() {
    match unsafe { esp_ota_mark_app_valid_cancel_rollback() } {
        ESP_OK => (),
        code => panic!(
            "Unexpected esp_ota_mark_app_valid_cancel_rollback code: {}",
            code
        ),
    }
}

/// Call this function to roll back to the previously workable app with reboot.
///
/// If rolling back failed, it returns an error, otherwise this function never returns,
/// as the CPU is rebooting.
pub fn rollback_and_reboot() -> Result<core::convert::Infallible> {
    match unsafe { esp_ota_mark_app_invalid_rollback_and_reboot() } {
        ESP_FAIL => Err(Error::from_kind(ErrorKind::RollbackFailed)),
        ESP_ERR_OTA_ROLLBACK_FAILED => Err(Error::from_kind(ErrorKind::RollbackFailedNoApps)),
        code => panic!(
            "Unexpected esp_ota_mark_app_invalid_rollback_and_reboot code: {}",
            code
        ),
    }
}
