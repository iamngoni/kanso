// Kanso — fixture_ai_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/ai_provider_info.dart';
import 'package:kanso/core/models/data/ai_settings.dart';
import 'package:kanso/core/models/data/ai_settings_change.dart';
import 'package:kanso/core/models/data/ai_test_result.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/ask_update.dart';
import 'package:kanso/core/models/data/assist_action.dart';
import 'package:kanso/core/models/data/assist_action_info.dart';
import 'package:kanso/core/models/data/assist_update.dart';
import 'package:kanso/core/models/repos/abs/ai_repository.dart';

/// The line the fixture gives for where a question goes.
const String fixturePrivacy =
    'Questions and the note excerpts they use go straight from this phone '
    'to api.anthropic.com. Nothing goes through Kanso’s servers. Locked '
    'notes and notes tagged #ai-exclude are never sent.';

/// The providers the fixture lists.
const List<AiProviderInfo> fixtureProviders = <AiProviderInfo>[
  AiProviderInfo(
    key: 'anthropic',
    label: 'Anthropic',
    description: 'Claude models, with a key from your Anthropic account.',
    defaultBaseUrl: 'https://api.anthropic.com',
    defaultModel: 'claude-sonnet',
    needsKey: true,
    acceptsKey: true,
  ),
  AiProviderInfo(
    key: 'ollama',
    label: 'Local',
    description: 'Ollama or another server on this phone’s network.',
    defaultBaseUrl: 'http://localhost:11434',
    defaultModel: 'llama3',
    needsKey: false,
    acceptsKey: true,
  ),
  AiProviderInfo(
    key: 'bedrock',
    label: 'Amazon Bedrock',
    description: 'Models in your AWS account.',
    defaultBaseUrl: 'https://bedrock-runtime.amazonaws.com',
    defaultModel: 'claude',
    needsKey: true,
    acceptsKey: true,
    usesRegion: true,
    defaultRegion: 'us-east-1',
  ),
];

/// The assistant's actions as the desktop words them.
const List<AssistActionInfo> fixtureAssistActions = <AssistActionInfo>[
  AssistActionInfo(
    action: AssistAction.improve,
    label: 'Improve Writing',
    noun: 'Improved',
    replaces: true,
  ),
  AssistActionInfo(
    action: AssistAction.shorten,
    label: 'Make Shorter',
    noun: 'Shorter',
    replaces: true,
  ),
  AssistActionInfo(
    action: AssistAction.fix,
    label: 'Fix Spelling and Grammar',
    noun: 'Corrected',
    replaces: true,
  ),
  AssistActionInfo(
    action: AssistAction.continueWriting,
    label: 'Continue Writing',
    noun: 'Continued',
    replaces: false,
    needsSelection: false,
  ),
  AssistActionInfo(
    action: AssistAction.summarize,
    label: 'Summarize Selection',
    noun: 'Summary',
    replaces: false,
  ),
];

/// AI for widget and golden tests: the settings and the saved key are kept
/// in memory, and what a question or a suggestion says is what a test pushes
/// into [askUpdates] and [assistUpdates].
class FixtureAiRepository implements AiRepository {
  FixtureAiRepository({AiSettings? settings, this.keptOut = false})
    : current =
          settings ??
          const AiSettings(
            provider: 'anthropic',
            host: 'api.anthropic.com',
            isRemote: true,
            privacy: fixturePrivacy,
          );

  /// The settings now.
  AiSettings current;

  /// Whether every note is kept out of AI.
  bool keptOut;

  /// What Test finds.
  AiTestResult testResult = const AiTestResult.ok(said: 'OK');

  /// The questions asked, and what was set, in order.
  final List<String> asked = <String>[];
  final List<
    ({String noteId, AssistAction action, String before, String selection})
  >
  assistRequests =
      <
        ({String noteId, AssistAction action, String before, String selection})
      >[];
  final List<String> summarized = <String>[];
  final List<AiSettingsChange> saved = <AiSettingsChange>[];
  final List<String> keysSet = <String>[];
  int stops = 0;
  int clears = 0;

  /// The updates of the question or suggestion being answered.
  final StreamController<AskUpdate> askUpdates =
      StreamController<AskUpdate>.broadcast();
  final StreamController<AssistUpdate> assistUpdates =
      StreamController<AssistUpdate>.broadcast();
  final StreamController<AiSettings> changes =
      StreamController<AiSettings>.broadcast();

  Future<void> close() async {
    await askUpdates.close();
    await assistUpdates.close();
    await changes.close();
  }

  @override
  Future<Either<ApplicationError, AiSettings>> settings() async =>
      Right<ApplicationError, AiSettings>(current);

  @override
  Stream<AiSettings> settingsChanges() => changes.stream;

  @override
  Future<Either<ApplicationError, List<AiProviderInfo>>> providers() async =>
      const Right<ApplicationError, List<AiProviderInfo>>(fixtureProviders);

  @override
  Future<Either<ApplicationError, AiSettings>> saveSettings(
    AiSettingsChange change,
  ) async {
    saved.add(change);
    current = current.copyWith(
      enabled: change.enabled,
      provider: change.provider,
      baseUrl: change.baseUrl,
      model: change.model,
      region: change.region,
    );
    changes.add(current);
    return Right<ApplicationError, AiSettings>(current);
  }

  @override
  Future<Either<ApplicationError, AiSettings>> setKey({
    required String provider,
    required String key,
  }) async {
    keysSet.add(key);
    current = current.copyWith(hasKey: key.isNotEmpty);
    changes.add(current);
    return Right<ApplicationError, AiSettings>(current);
  }

  @override
  Future<Either<ApplicationError, AiTestResult>> test() async =>
      Right<ApplicationError, AiTestResult>(testResult);

  @override
  Stream<AskUpdate> ask(String question) {
    asked.add(question);
    return askUpdates.stream;
  }

  @override
  Future<Either<ApplicationError, Unit>> stop() async {
    stops++;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> clear() async {
    clears++;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, bool>> isEligible(String noteId) async =>
      Right<ApplicationError, bool>(!keptOut);

  @override
  List<AssistActionInfo> assistActions() => fixtureAssistActions;

  @override
  Stream<AssistUpdate> assist({
    required String noteId,
    required AssistAction action,
    required String before,
    required String selection,
  }) {
    assistRequests.add((
      noteId: noteId,
      action: action,
      before: before,
      selection: selection,
    ));
    return assistUpdates.stream;
  }

  @override
  Stream<AssistUpdate> summarize(String noteId) {
    summarized.add(noteId);
    return assistUpdates.stream;
  }
}
