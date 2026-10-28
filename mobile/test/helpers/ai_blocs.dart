// Kanso — ai_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/core/models/data/ai_settings.dart';

import 'close_bloc_later.dart';
import 'fixture_ai_repository.dart';

/// The settings of a phone with AI [enabled], using Anthropic.
AiSettings aiSettings({bool enabled = false, bool hasKey = false}) =>
    AiSettings(
      enabled: enabled,
      provider: 'anthropic',
      hasKey: hasKey,
      host: 'api.anthropic.com',
      isRemote: true,
      privacy: fixturePrivacy,
    );

/// An `AiStatusBloc` that has read the settings: AI off unless [enabled].
Future<AiStatusBloc> startedAiStatus({
  bool enabled = false,
  FixtureAiRepository? repository,
}) async {
  final bloc = AiStatusBloc(
    repository ?? FixtureAiRepository(settings: aiSettings(enabled: enabled)),
  )..add(const AiStatusEvent.started());
  closeBlocLater(bloc);
  await bloc.stream.firstWhere((state) => state.loaded);
  return bloc;
}
