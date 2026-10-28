// Kanso — closed_tab.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'tab_meta.dart';

part 'closed_tab.freezed.dart';

/// A closed tab, and the place in the row it was closed from.
@freezed
abstract class ClosedTab with _$ClosedTab {
  const factory ClosedTab({required TabMeta meta, required int at}) =
      _ClosedTab;
}
