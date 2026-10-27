// Kanso — register_editor_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/editor/models/repos/abs/editor_repository.dart';
import 'package:kanso/editor/state/editor/editor_bloc.dart';
import 'package:kanso/injection.dart';

import 'fake_editor_repository.dart';
import 'fake_web_link_repository.dart';

/// Registers, for a test that builds the note page in edit mode, an
/// `EditorBloc` that opens [document]; it is taken out when the test ends.
FakeEditorRepository registerEditorBloc(
  EditorDocument document, {
  FakeWebLinkRepository? links,
}) {
  final repository = FakeEditorRepository(document: document);
  registerEditorBlocOn(repository, links: links);
  return repository;
}

/// Registers an `EditorBloc` over [repository] (the fake one, or the real
/// core's), for a test that builds the note page in edit mode.
void registerEditorBlocOn(
  EditorRepository repository, {
  FakeWebLinkRepository? links,
}) {
  if (getIt.isRegistered<EditorBloc>()) {
    getIt.unregister<EditorBloc>();
  }
  getIt.registerFactory<EditorBloc>(
    () => EditorBloc(repository, links ?? FakeWebLinkRepository()),
  );
  addTearDown(() {
    if (getIt.isRegistered<EditorBloc>()) {
      getIt.unregister<EditorBloc>();
    }
  });
}
