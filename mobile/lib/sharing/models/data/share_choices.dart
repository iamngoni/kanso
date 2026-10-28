// Kanso — share_choices.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../../core/bridge/api/types.dart';

/// The roles a person can be given in a note, in the order the desktop
/// offers them: read, comment, edit. The owner's role is never given.
const List<ShareRole> shareableRoles = <ShareRole>[
  ShareRole.viewer,
  ShareRole.commenter,
  ShareRole.editor,
];

/// How many days an invite link can last, as the desktop offers them.
const List<int> shareLinkDays = <int>[1, 7, 30];
