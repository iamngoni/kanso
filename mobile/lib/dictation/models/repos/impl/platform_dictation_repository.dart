// Kanso — platform_dictation_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';
import 'package:kanso_dictation/kanso_dictation.dart' as plugin;

import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../../data/dictation_capability.dart';
import '../../data/dictation_failure.dart';
import '../../data/dictation_permission.dart';
import '../../data/dictation_signal.dart';
import '../../data/dictation_unavailable_reason.dart';
import '../abs/dictation_repository.dart';

/// Dictation through the `kanso_dictation` plugin: `SFSpeechRecognizer` on
/// iOS and `SpeechRecognizer` on Android, both on the device.
@LazySingleton(as: DictationRepository)
class PlatformDictationRepository implements DictationRepository {
  PlatformDictationRepository(this.dictation);

  final plugin.KansoDictation dictation;

  @override
  Future<Either<ApplicationError, DictationCapability>> availability({
    String? localeTag,
  }) => attempt(() async {
    final answer = await dictation.availability(localeTag);
    return DictationCapability(
      available: answer.available,
      onDevice: answer.onDevice,
      permission: permissionOf(answer.permission),
      language: answer.language,
      reason: answer.reason == null ? null : reasonOf(answer.reason!),
    );
  });

  @override
  Future<Either<ApplicationError, DictationPermission>> requestPermission() =>
      attempt(() async => permissionOf(await dictation.requestPermission()));

  @override
  Future<Either<ApplicationError, Unit>> start({String? localeTag}) =>
      attempt(() async {
        await dictation.start(localeTag);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> stop() => attempt(() async {
    await dictation.stop();
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> cancel() => attempt(() async {
    await dictation.cancel();
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> openSettings() => attempt(() async {
    await dictation.openSettings();
    return unit;
  });

  @override
  Stream<DictationSignal> signals() => dictation.events
      .map(signalOf)
      .where((signal) => signal != null)
      .cast<DictationSignal>();

  /// The plugin's permission as the app's.
  static DictationPermission permissionOf(plugin.DictationPermission value) =>
      switch (value) {
        plugin.DictationPermission.notDetermined =>
          DictationPermission.notDetermined,
        plugin.DictationPermission.denied => DictationPermission.denied,
        plugin.DictationPermission.restricted => DictationPermission.restricted,
        plugin.DictationPermission.granted => DictationPermission.granted,
      };

  static DictationUnavailableReason reasonOf(
    plugin.DictationUnavailableReason value,
  ) => switch (value) {
    plugin.DictationUnavailableReason.noRecognizer =>
      DictationUnavailableReason.noRecognizer,
    plugin.DictationUnavailableReason.notOnDevice =>
      DictationUnavailableReason.notOnDevice,
    plugin.DictationUnavailableReason.unsupportedLanguage =>
      DictationUnavailableReason.unsupportedLanguage,
  };

  static DictationFailure failureOf(plugin.DictationErrorCode? value) =>
      switch (value) {
        plugin.DictationErrorCode.permissionDenied =>
          DictationFailure.permissionDenied,
        plugin.DictationErrorCode.notAvailable => DictationFailure.notAvailable,
        plugin.DictationErrorCode.noSpeech => DictationFailure.noSpeech,
        plugin.DictationErrorCode.audio => DictationFailure.audio,
        plugin.DictationErrorCode.busy => DictationFailure.busy,
        plugin.DictationErrorCode.interrupted => DictationFailure.interrupted,
        plugin.DictationErrorCode.unknown || null => DictationFailure.unknown,
      };

  /// What the plugin reported as a signal; null for an event without the
  /// words or the level it should carry.
  static DictationSignal? signalOf(plugin.DictationEvent event) =>
      switch (event.kind) {
        plugin.DictationEventKind.listening =>
          const DictationSignal.listening(),
        plugin.DictationEventKind.partial =>
          event.text == null ? null : DictationSignal.partial(event.text!),
        plugin.DictationEventKind.finalResult =>
          event.text == null ? null : DictationSignal.finalText(event.text!),
        plugin.DictationEventKind.level =>
          event.level == null ? null : DictationSignal.level(event.level!),
        plugin.DictationEventKind.error => DictationSignal.failed(
          failureOf(event.error),
        ),
        plugin.DictationEventKind.ended => const DictationSignal.ended(),
      };
}
