// Kanso — seeded_dictation_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';

import 'fake_dictation_text_rules.dart';
import 'mock_dictation_repository.dart';

/// A `DictationBloc` that starts in `seed`, for pictures of the states that
/// need a phone to be in them. It listens to nothing.
class SeededDictationBloc extends DictationBloc {
  SeededDictationBloc(DictationState seed, SpokenRepository spoken)
    : super(spoken.repository, PassThroughRules()) {
    emit(seed);
  }
}
