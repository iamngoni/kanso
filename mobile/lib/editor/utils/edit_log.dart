// Kanso — edit_log.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';
import 'shift_runs.dart';

/// The changes made to the editor's text since colours were last asked for,
/// so that colours which arrive for an older text can be moved to fit the
/// newer one instead of being thrown away.
///
/// While a person types, the text changes faster than colours come back (a
/// call into Rust and back takes longer than a key at thirty a second). If
/// only colours of the very latest text were kept, none would be shown until
/// the typing paused. With the log, colours of a text a few keys old are
/// shown moved along ([shiftRuns]), as the colours already on screen are.
///
/// Each entry is a change from one version of the text to the next, in units
/// of the field. A change the log cannot follow (the window moved, a text
/// came from outside) ends the chain with [reset].
class EditLog {
  EditLog({this.limit = 256});

  /// How many changes are kept; the oldest go first, which makes colours of
  /// a text that old unusable.
  final int limit;

  final List<(int, int, int, int, int)> _entries =
      <(int, int, int, int, int)>[];

  /// How many changes are kept.
  int get length => _entries.length;

  /// Notes that the text went from version [from] to [to] by replacing the
  /// units [start] to [end] with [inserted] units.
  void record({
    required int from,
    required int to,
    required int start,
    required int end,
    required int inserted,
  }) {
    _entries.add((from, to, start, end, inserted));
    if (_entries.length > limit) {
      _entries.removeRange(0, _entries.length - limit);
    }
  }

  /// Forgets every change: the text now cannot be followed from any version
  /// before it.
  void reset() => _entries.clear();

  /// Forgets the changes that lead to versions up to [version] (colours for
  /// them have been shown).
  void forget(int version) {
    _entries.removeWhere((entry) => entry.$2 <= version);
  }

  /// [runs], the colours of the text at version [from], moved to fit the text
  /// at version [now]; null when the log does not hold every change between
  /// the two.
  List<EditorRun>? carry(
    List<EditorRun> runs, {
    required int from,
    required int now,
  }) {
    if (from == now) {
      return runs;
    }
    var carried = runs;
    var at = from;
    while (at != now) {
      final next = _entries.where((entry) => entry.$1 == at).firstOrNull;
      if (next == null || next.$2 == at) {
        return null;
      }
      carried = shiftRuns(
        carried,
        start: next.$3,
        end: next.$4,
        inserted: next.$5,
      );
      at = next.$2;
    }
    return carried;
  }
}
