# Changelog
The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).

Entries should have the imperative form, just like commit messages. Start each entry with words like
add, fix, increase, force etc.. Not added, fixed, increased, forced etc.

### Categories each change fall into

* **Added**: for new features.
* **Changed**: for changes in existing functionality.
* **Deprecated**: for soon-to-be removed features.
* **Removed**: for now removed features.
* **Fixed**: for any bug fixes.
* **Security**: in case of vulnerabilities.


## [Unreleased]


## [0.2.2] - 2025-01-31
### Changed
- Upgrade `esp-idf-sys` to 0.36
- Upgrade `embuild` to 0.33
- Disable the `log` feature by default, making it opt-in. Default features are evil,
  especially the ones pulling in dependencies.


## [0.2.1] - 2025-01-31
### Added
- Implement `Send` for `OtaUpdate`.

### Changed
- Upgrade `esp-idf-sys` to 0.35
- Upgrade `embuild` to 0.32


## [0.2.0] - 2023-07-09
### Changed
- Upgrade `esp-idf-sys` from 0.31 to 0.33.
- Upgrade `embuild` from 0.30 to 0.31


## [0.1.0] - 2022-10-01
Initial release. Can perform OTA updates
