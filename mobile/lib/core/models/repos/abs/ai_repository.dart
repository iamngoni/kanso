// Kanso — ai_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../data/ai_provider_info.dart';
import '../../data/ai_settings.dart';
import '../../data/ai_settings_change.dart';
import '../../data/ai_test_result.dart';
import '../../data/application_error.dart';
import '../../data/ask_update.dart';
import '../../data/assist_action.dart';
import '../../data/assist_action_info.dart';
import '../../data/assist_update.dart';

/// AI: its settings, the key, questions asked of the notes, and the inline
/// assistant. The key is kept by the phone's secret store and is never read
/// back; no note that is locked, in the trash or tagged `#ai-exclude` is
/// ever sent.
abstract interface class AiRepository {
  /// Where AI stands now.
  Future<Either<ApplicationError, AiSettings>> settings();

  /// The settings now and each time they change, wherever they were changed.
  Stream<AiSettings> settingsChanges();

  /// The places answers can come from, best first.
  Future<Either<ApplicationError, List<AiProviderInfo>>> providers();

  /// Saves [change] and gives the settings as they are now.
  Future<Either<ApplicationError, AiSettings>> saveSettings(
    AiSettingsChange change,
  );

  /// Saves [key] for the provider named [provider]; an empty [key] removes
  /// the one saved. The settings are returned as they are now.
  Future<Either<ApplicationError, AiSettings>> setKey({
    required String provider,
    required String key,
  });

  /// Sends one short question to check the saved address, model and key.
  /// No note is sent.
  Future<Either<ApplicationError, AiTestResult>> test();

  /// Asks [question] of the notes, as a follow-up to the conversation so
  /// far. The stream ends with one of `AskDone`, `AskNothingMatches`,
  /// `AskCutOff` or `AskFailed`; a failure is an update, not a stream error.
  Stream<AskUpdate> ask(String question);

  /// Stops the question being answered. What was written is kept in the
  /// conversation.
  Future<Either<ApplicationError, Unit>> stop();

  /// Forgets the conversation, so the next question starts a new one.
  Future<Either<ApplicationError, Unit>> clear();

  /// Whether the note may be sent to AI at all.
  Future<Either<ApplicationError, bool>> isEligible(String noteId);

  /// The assistant's actions, in the order a menu lists them.
  List<AssistActionInfo> assistActions();

  /// A suggestion for [selection] (the text just before it, [before], is
  /// context only) of the note [noteId]. A stream of `AssistText`, ending in
  /// `AssistDone`, `AssistKeptOut` or `AssistFailed`.
  Stream<AssistUpdate> assist({
    required String noteId,
    required AssistAction action,
    required String before,
    required String selection,
  });

  /// A summary of the note [noteId], streamed as [assist] is.
  Stream<AssistUpdate> summarize(String noteId);
}
