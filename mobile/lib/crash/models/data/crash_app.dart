// Kanso — crash_app.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'crash_app.freezed.dart';
part 'crash_app.g.dart';

/// The `app` object of a report: which build crashed.
@freezed
abstract class CrashApp with _$CrashApp {
  const factory CrashApp({
    /// `0.9.0`.
    required String version,

    /// `57`.
    required String build,

    /// Shorebird's patch number; null without Shorebird or before the first
    /// patch. Always written, `null` included: the server requires the key
    /// (the build.yaml default, `include_if_null: false`, would drop it and
    /// the server would answer 400 for `app.patch`).
    @JsonKey(includeIfNull: true) required int? patch,
  }) = _CrashApp;

  factory CrashApp.fromJson(Map<String, dynamic> json) =>
      _$CrashAppFromJson(json);
}
