// Kanso — push_setting_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the setting that wakes this device is ready to do.
enum PushSettingStatus {
  /// Reading the setting.
  loading,

  /// It can be switched on and off.
  ready,

  /// Push can't work on this device, so there is nothing to switch.
  unavailable,
}
