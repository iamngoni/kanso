// Kanso — app_feature_fixtures.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:kanso/ai_settings/state/ai_settings/ai_settings_bloc.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/ask/state/ask/ask_bloc.dart';
import 'package:kanso/assist/state/assist/assist_bloc.dart';
import 'package:kanso/core/models/repos/abs/ai_repository.dart';
import 'package:kanso/dictation/models/repos/abs/dictation_text_rules.dart';
import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';
import 'package:kanso/injection.dart';

import '../dictation/helpers/fake_dictation_text_rules.dart';
import '../dictation/helpers/mock_dictation_repository.dart';
import 'fixture_ai_repository.dart';

/// The app-wide AI and dictation dependencies used when a test pumps App.
/// Native speech and remote model calls stay outside widget tests.
class AppFeatureFixtures {
  AppFeatureFixtures({FixtureAiRepository? ai})
    : ai = ai ?? FixtureAiRepository(),
      dictation = SpokenRepository();

  final FixtureAiRepository ai;
  final SpokenRepository dictation;
  final PassThroughRules rules = PassThroughRules();

  late final AiStatusBloc status = AiStatusBloc(ai)
    ..add(const AiStatusEvent.started());
  late final DictationBloc dictationBloc = DictationBloc(
    dictation.repository,
    rules,
  );

  /// Registers the dependencies the app's editor scope resolves at startup.
  void register() {
    _unregister<AiSettingsBloc>();
    _unregister<AskBloc>();
    _unregister<AssistBloc>();
    if (getIt.isRegistered<AiRepository>()) {
      getIt.unregister<AiRepository>();
    }
    getIt.registerLazySingleton<AiRepository>(() => ai);
    if (getIt.isRegistered<AiStatusBloc>()) {
      getIt.unregister<AiStatusBloc>();
    }
    getIt
      ..registerFactory<AiStatusBloc>(() => status)
      ..registerFactory<AiSettingsBloc>(() => AiSettingsBloc(ai))
      ..registerFactory<AskBloc>(() => AskBloc(ai))
      ..registerFactory<AssistBloc>(() => AssistBloc(ai));
    if (getIt.isRegistered<DictationBloc>()) {
      getIt.unregister<DictationBloc>();
    }
    getIt.registerFactory<DictationBloc>(() => dictationBloc);
    if (getIt.isRegistered<DictationTextRules>()) {
      getIt.unregister<DictationTextRules>();
    }
    getIt.registerFactory<DictationTextRules>(() => rules);
  }

  /// Closes the test streams and BLoCs after the pumped app is removed.
  Future<void> dispose() async {
    if (!status.isClosed) {
      await status.close();
    }
    if (!dictationBloc.isClosed) {
      await dictationBloc.close();
    }
    await ai.close();
    dictation.close();
    _unregister<AiSettingsBloc>();
    _unregister<AskBloc>();
    _unregister<AssistBloc>();
    _unregister<AiRepository>();
    _unregister<AiStatusBloc>();
    _unregister<DictationBloc>();
    _unregister<DictationTextRules>();
  }

  void _unregister<T extends Object>() {
    if (getIt.isRegistered<T>()) {
      getIt.unregister<T>();
    }
  }
}
