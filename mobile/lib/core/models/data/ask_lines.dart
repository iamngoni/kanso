// Kanso — ask_lines.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ask_lines.freezed.dart';

/// Where the lines a receipt pointed at are now: first and last, from 0.
@freezed
abstract class AskLines with _$AskLines {
  const factory AskLines({required int start, required int end}) = _AskLines;
}
