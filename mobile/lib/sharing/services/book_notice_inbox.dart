// Kanso — book_notice_inbox.dart
// Created 2026-11-14.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:injectable/injectable.dart';

import '../../core/bridge/api/types.dart';
import '../../core/models/repos/abs/shared_notes_repository.dart';

/// Keeps what syncs of shared notebooks said ("You were removed from
/// "Studio"…", ""Brief" was edited after you deleted it, so it's back.")
/// until the person reads it.
///
/// The core says each notice once, to whoever listens at that moment. On a
/// phone the library, and its Shared section, is a drawer that is only built
/// while it is open, so a notice from a sync while it was closed (at launch,
/// or any time the person was in a note) was never shown. This listens for
/// the life of the app, from [start], and the section shows what it holds.
@lazySingleton
class BookNoticeInbox {
  BookNoticeInbox(this._shared);

  final SharedNotesRepository _shared;
  final List<String> _notices = <String>[];
  final StreamController<List<String>> _changes =
      StreamController<List<String>>.broadcast();
  StreamSubscription<SharedEvent>? _listening;

  /// The notices not read yet, oldest first.
  List<String> get notices => List<String>.unmodifiable(_notices);

  /// [notices] each time they change.
  Stream<List<String>> get changes => _changes.stream;

  /// Starts listening to the syncs; once is enough, later calls do nothing.
  void start() {
    _listening ??= _shared.events().listen((news) {
      if (news.bookNotices.isNotEmpty) {
        _notices.addAll(news.bookNotices);
        _tell();
      }
    });
  }

  /// The first notice was read.
  void dismissFirst() {
    if (_notices.isNotEmpty) {
      _notices.removeAt(0);
      _tell();
    }
  }

  void _tell() {
    if (!_changes.isClosed) {
      _changes.add(notices);
    }
  }

  /// Stops listening.
  Future<void> close() async {
    await _listening?.cancel();
    _listening = null;
    await _changes.close();
  }
}
