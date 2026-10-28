// Kanso — focus_mode_bloc_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/shell/state/focus_mode/focus_mode_bloc.dart';

import '../../../helpers/seeded_appearance_bloc.dart';

void main() {
  group('FocusModeBloc', () {
    test('starts from the saved choice', () async {
      final preferences = SeededPreferencesRepository(
        const Preferences(focusMode: true),
      );
      final bloc = FocusModeBloc(preferences)
        ..add(const FocusModeEvent.started());
      addTearDown(bloc.close);

      final state = await bloc.stream.firstWhere((state) => state.ready);
      expect(state.active, isTrue);
    });

    test('toggling shows the change at once and saves it', () async {
      final preferences = SeededPreferencesRepository(const Preferences());
      final bloc = FocusModeBloc(preferences)
        ..add(const FocusModeEvent.started());
      addTearDown(bloc.close);
      await bloc.stream.firstWhere((state) => state.ready);

      bloc.add(const FocusModeEvent.toggled());
      final on = await bloc.stream.firstWhere((state) => state.active);
      expect(on.active, isTrue);
      expect(preferences.current.focusMode, isTrue);

      bloc.add(const FocusModeEvent.toggled());
      final off = await bloc.stream.firstWhere((state) => !state.active);
      expect(off.active, isFalse);
      expect(preferences.current.focusMode, isFalse);
    });

    test('leaving focus mode saves the off choice', () async {
      final preferences = SeededPreferencesRepository(
        const Preferences(focusMode: true),
      );
      final bloc = FocusModeBloc(preferences)
        ..add(const FocusModeEvent.started());
      addTearDown(bloc.close);
      await bloc.stream.firstWhere((state) => state.ready);

      bloc.add(const FocusModeEvent.exited());
      final state = await bloc.stream.firstWhere((state) => !state.active);
      expect(state.active, isFalse);
      expect(preferences.current.focusMode, isFalse);
    });
  });
}
