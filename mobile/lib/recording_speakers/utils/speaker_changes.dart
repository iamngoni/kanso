// Kanso — speaker_changes.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/recording_speakers_types.dart';

/// The changes a Rename Speakers form asks for: each name in [names] paired
/// with what was typed for it ([typed], same order), left out when what was
/// typed is empty or is the name it already has. The core cleans the names
/// it writes; this only decides what to ask for.
List<SpeakerChange> speakerChanges(
  List<SpeakerNameView> names,
  List<String> typed,
) {
  final changes = <SpeakerChange>[];
  for (var i = 0; i < names.length && i < typed.length; i++) {
    final to = typed[i].trim();
    if (to.isNotEmpty && to != names[i].name) {
      changes.add(SpeakerChange(from: names[i].name, to: to));
    }
  }
  return changes;
}
