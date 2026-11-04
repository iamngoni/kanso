// Kanso — feedback_kind.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the person says a report is. Only a hint: triage may change it.
enum FeedbackKind {
  bug('bug'),
  idea('idea'),
  question('question');

  const FeedbackKind(this.wire);

  /// The word the bridge and the server use.
  final String wire;

  /// The kind a bridge word names; a bug if it is unknown.
  static FeedbackKind fromWire(String wire) => FeedbackKind.values.firstWhere(
    (kind) => kind.wire == wire,
    orElse: () => FeedbackKind.bug,
  );
}
