// Kanso — rust_when_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/bridge/api/when.dart' as bridge;
import '../../../../core/utils/journal_time.dart';
import '../abs/when_repository.dart';

/// [WhenRepository] over the Rust core, on the phone's clock.
@LazySingleton(as: WhenRepository)
class RustWhenRepository implements WhenRepository {
  RustWhenRepository();

  @override
  WhenHint? hint(String line, {required bool monthFirst}) {
    try {
      return bridge.whenParse(
        line: line,
        now: localTimeOf(DateTime.now()),
        monthFirst: monthFirst,
      );
    } on Object {
      return null;
    }
  }

  @override
  DateChoices? choices(String beforeCaret) {
    try {
      return bridge.dateChoices(
        beforeCaret: beforeCaret,
        now: localTimeOf(DateTime.now()),
      );
    } on Object {
      return null;
    }
  }
}
