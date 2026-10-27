// Kanso — live_rooms.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

/// Who holds each note's live room open, over a room that has no count of
/// its own: the first function given enters a note's room and the second
/// leaves it.
///
/// A room is entered when its first holder takes it and left when its last
/// holder lets go. Every take and every let-go runs after the one before it
/// finished, so a take never lands between a let-go's count and its leave
/// and finds the room closed under it. A take that fails holds nothing.
class LiveRooms {
  LiveRooms(this._open, this._close);

  final Future<void> Function(String noteId) _open;
  final Future<void> Function(String noteId) _close;
  final Map<String, Set<Object>> _holders = <String, Set<Object>>{};
  Future<void> _last = Future<void>.value();

  /// Whether [noteId]'s room is held open by anyone.
  bool isOpen(String noteId) => _holders.containsKey(noteId);

  /// [holder] takes [noteId]'s room, entering it if nobody held it. Taking
  /// it twice holds it once.
  Future<void> take(String noteId, Object holder) => _inTurn(() async {
    final holders = _holders[noteId];
    if (holders != null) {
      holders.add(holder);
      return;
    }
    await _open(noteId);
    _holders[noteId] = Set<Object>.identity()..add(holder);
  });

  /// [holder] lets [noteId]'s room go, leaving it if nobody else holds it.
  /// Letting go of a room it does not hold does nothing.
  Future<void> letGo(String noteId, Object holder) => _inTurn(() async {
    final holders = _holders[noteId];
    if (holders == null || !holders.remove(holder)) {
      return;
    }
    if (holders.isEmpty) {
      _holders.remove(noteId);
      await _close(noteId);
    }
  });

  Future<void> _inTurn(Future<void> Function() step) {
    final done = _last.then((_) => step());
    // The next step waits for this one whether it worked or not.
    _last = done.then((_) {}, onError: (Object _) {});
    return done;
  }
}
