// Kanso — when_hint_view.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../../core/bridge/api/types.dart';

/// A typed-date hint and where its line starts in the note, so that it can
/// be written back in units of the note.
typedef WhenHintView = ({WhenHint hint, int lineStart});
