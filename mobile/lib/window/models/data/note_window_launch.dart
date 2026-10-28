// Kanso — note_window_launch.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'note_window_launch.freezed.dart';

/// What a note's window is started with: the note, and where the library is
/// (the same folder as the main window's, so that both engines open the one
/// library).
@freezed
abstract class NoteWindowLaunch with _$NoteWindowLaunch {
  const factory NoteWindowLaunch({
    required String noteId,

    /// The name of the `AppEnvironment` the main window runs in.
    required String environment,

    /// The library folder of the main window; empty for the app's own.
    @Default('') String libraryFolder,
  }) = _NoteWindowLaunch;

  const NoteWindowLaunch._();

  /// The entry point's arguments: the note, the environment and the folder.
  List<String> get args => <String>[noteId, environment, libraryFolder];

  /// Reads the entry point's arguments; null when there is no note in them.
  static NoteWindowLaunch? fromArgs(List<String> args) {
    if (args.isEmpty || args.first.isEmpty) {
      return null;
    }
    return NoteWindowLaunch(
      noteId: args.first,
      environment: args.length > 1 && args[1].isNotEmpty
          ? args[1]
          : 'production',
      libraryFolder: args.length > 2 ? args[2] : '',
    );
  }
}
