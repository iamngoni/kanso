// Kanso — reminder_permission.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Whether the system lets Kanso show notifications.
enum ReminderPermission {
  granted,
  denied,

  /// The person has not been asked yet.
  notDetermined;

  /// The permission [wire] names; not determined for anything unknown.
  static ReminderPermission parse(String? wire) => switch (wire) {
    'granted' => granted,
    'denied' => denied,
    _ => notDetermined,
  };
}
