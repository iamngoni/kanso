// Kanso — speaker_changes_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' as g;
import 'package:kanso/core/bridge/api/recording_speakers_types.dart';
import 'package:kanso/recording_speakers/utils/speaker_changes.dart';

import '../../helpers/property.dart';

List<SpeakerNameView> _names(int count) => <SpeakerNameView>[
  for (var i = 0; i < count; i++)
    SpeakerNameView(name: 'Speaker ${i + 1}', paragraphs: i + 1),
];

void main() {
  // no-prop-test: three fixed forms.
  test('only names that were changed and are not empty are asked for', () {
    final names = _names(3);
    expect(
      speakerChanges(names, <String>['Speaker 1', '  Priya ', '']),
      <SpeakerChange>[const SpeakerChange(from: 'Speaker 2', to: 'Priya')],
    );
    expect(
      speakerChanges(names, <String>['Speaker 1', 'Speaker 2', 'Speaker 3']),
      isEmpty,
    );
    expect(speakerChanges(const <SpeakerNameView>[], <String>[]), isEmpty);
  });

  g.Glados2<int, List<String>>(
    g.any.intInRange(0, 6),
    g.any.list(g.any.letterOrDigits),
    kansoExploreConfig(),
  ).test('every change names a speaker that exists and something new', (
    count,
    typed,
  ) {
    final names = _names(count);
    final changes = speakerChanges(names, typed);
    final known = names.map((n) => n.name).toSet();
    expect(changes.length, lessThanOrEqualTo(count));
    for (final change in changes) {
      expect(known, contains(change.from));
      expect(change.to, isNotEmpty);
      expect(change.to, isNot(change.from));
      expect(change.to, change.to.trim());
    }
    // In the order the speakers appear.
    final order = changes.map((c) => c.from).toList();
    final listed = names.map((n) => n.name).where(order.contains).toList();
    expect(order, listed);
  });
}
