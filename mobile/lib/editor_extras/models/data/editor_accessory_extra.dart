// Kanso — editor_accessory_extra.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../../../core/configs/kanso_icons.dart';
import 'editor_text_target.dart';

/// A key another feature adds to the editor's accessory bar, after the
/// Markdown keys: dictation, and the inline assistant.
@immutable
class EditorAccessoryExtra {
  const EditorAccessoryExtra({
    required this.id,
    required this.icon,
    required this.semanticLabel,
    required this.onTap,
    this.active,
  });

  /// Stable, for tests and keys: `dictation`, `assist`.
  final String id;

  /// One of `KansoIcons`' Lucide paths.
  final KansoIcons icon;
  final String semanticLabel;

  /// Runs the key on the open note. Null dims the key.
  final void Function(BuildContext context, EditorTextTarget target)? onTap;

  /// Whether the key is on now (dictation listening), drawn highlighted.
  final ValueListenable<bool>? active;
}
