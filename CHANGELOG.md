# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-11-12

### Added

- Daily store dialog now shows VP and RP wallet balances, skin tier badges and icons, reset countdown, and skeleton loading states.
- Click any skin card in the daily store to search it on YouTube.
- Footer status bar with quick cache cleaner access and a GitHub repository link.

### Changed

- Rewrote the desktop UI from Slint to GPUI (`gpui-kit`) with a Valorant-inspired dark theme.
- Consolidated account switching, daily store viewing, and cache cleaning into a single-window layout with modal dialogs.
- Simplified cache cleaning to clear temporary caches in `%LOCALAPPDATA%\VALORANT` via a confirmation dialog showing total size.

## [0.1.1] - 2026-08-25

### Fixed

- Fixed login issues on newer versions of Riot Client.

## [0.1.0] - 2026-06-12

[Unreleased]: https://github.com/clitic/valtools/compare/0.2.0...HEAD
[0.2.0]: https://github.com/clitic/valtools/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/clitic/valtools/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/clitic/valtools/compare/c22322b...0.1.0
