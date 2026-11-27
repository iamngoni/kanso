// Kanso — library_pane_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:kanso/shell/state/library_pane/library_pane_bloc.dart';

void main() {
  blocTest<LibraryPaneBloc, LibraryPaneState>(
    'starts shown, and each toggle shows or hides it',
    build: LibraryPaneBloc.new,
    act: (bloc) => bloc
      ..add(const LibraryPaneEvent.toggled())
      ..add(const LibraryPaneEvent.toggled())
      ..add(const LibraryPaneEvent.toggled()),
    expect: () => <LibraryPaneState>[
      const LibraryPaneState(visible: false),
      const LibraryPaneState(),
      const LibraryPaneState(visible: false),
    ],
  );

  blocTest<LibraryPaneBloc, LibraryPaneState>(
    'puts both panes away and back, apart from the library toggle',
    build: LibraryPaneBloc.new,
    act: (bloc) => bloc
      ..add(const LibraryPaneEvent.panesToggled())
      ..add(const LibraryPaneEvent.toggled())
      ..add(const LibraryPaneEvent.panesToggled()),
    expect: () => <LibraryPaneState>[
      const LibraryPaneState(panesHidden: true),
      const LibraryPaneState(visible: false, panesHidden: true),
      const LibraryPaneState(visible: false),
    ],
  );
}
