// Kanso — register_editor_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/editor/models/repos/abs/editor_repository.dart';
import 'package:kanso/editor/state/editor/editor_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/journal/state/journal/journal_bloc.dart';

import 'fake_editor_repository.dart';
import 'fake_media_picker_repository.dart';
import 'fake_web_link_repository.dart';
import 'fixture_journal_repository.dart';

/// Registers, for a test that builds the note page in edit mode, an
/// `EditorBloc` that opens [document]; it is taken out when the test ends.
FakeEditorRepository registerEditorBloc(
  EditorDocument document, {
  FakeWebLinkRepository? links,
  FakeMediaPickerRepository? picker,
}) {
  final repository = FakeEditorRepository(document: document);
  registerEditorBlocOn(repository, links: links, picker: picker);
  return repository;
}

/// Registers an `EditorBloc` over [repository] (the fake one, or the real
/// core's), for a test that builds the note page in edit mode.
void registerEditorBlocOn(
  EditorRepository repository, {
  FakeWebLinkRepository? links,
  FakeMediaPickerRepository? picker,
}) {
  if (getIt.isRegistered<EditorBloc>()) {
    getIt.unregister<EditorBloc>();
  }
  getIt.registerFactory<EditorBloc>(
    () => EditorBloc(
      repository,
      links ?? FakeWebLinkRepository(),
      picker ?? FakeMediaPickerRepository(),
    ),
  );
  // The note page carries the journal's day stepper and New menu, which the
  // note pane gets from a JournalOpener reading this one.
  final journal = FixtureJournalRepository();
  if (getIt.isRegistered<JournalBloc>()) {
    getIt.unregister<JournalBloc>();
  }
  getIt.registerFactory<JournalBloc>(() => JournalBloc(journal, journal));
  addTearDown(() {
    if (getIt.isRegistered<EditorBloc>()) {
      getIt.unregister<EditorBloc>();
    }
    if (getIt.isRegistered<JournalBloc>()) {
      getIt.unregister<JournalBloc>();
    }
  });
}
