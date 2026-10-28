// Kanso — rust_ai_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/ai.dart' as bridge;
import '../../../router/layout_class_notifier.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/ai_note_ref.dart';
import '../../data/ai_provider_info.dart';
import '../../data/ai_settings.dart';
import '../../data/ai_settings_change.dart';
import '../../data/ai_test_result.dart';
import '../../data/application_error.dart';
import '../../data/ask_update.dart';
import '../../data/assist_action.dart';
import '../../data/assist_action_info.dart';
import '../../data/assist_update.dart';
import '../abs/ai_repository.dart';

/// What the core says when a note is kept out of AI.
const String keptOutMessage = 'This note is kept out of AI.';

/// The key the core knows each assistant action by.
String assistActionKey(AssistAction action) => switch (action) {
  AssistAction.improve => 'improve',
  AssistAction.shorten => 'shorten',
  AssistAction.fix => 'fix',
  AssistAction.continueWriting => 'continue',
  AssistAction.summarize => 'summarize',
};

/// The assistant action the core calls [key], if it is one the app knows.
AssistAction? assistActionOfKey(String key) {
  for (final action in AssistAction.values) {
    if (assistActionKey(action) == key) {
      return action;
    }
  }
  return null;
}

/// [AiRepository] over the Rust core's AI. The key goes to the phone's
/// secret store through the core and is never kept, returned or logged here.
@LazySingleton(as: AiRepository)
class RustAiRepository implements AiRepository {
  RustAiRepository(this._ai, this._layout);

  final bridge.KansoAi _ai;
  final LayoutClassNotifier _layout;

  late final bridge.KansoAskConversation _conversation = _ai.conversation();
  final StreamController<AiSettings> _changes =
      StreamController<AiSettings>.broadcast();

  Future<AiSettings> _view(bridge.AiSettingsView view) async {
    final device = _layout.tabletDevice ? 'this tablet' : 'this phone';
    final line = await _ai.privacyLine(device: device);
    return AiSettings(
      enabled: view.enabled,
      provider: view.provider,
      baseUrl: view.baseUrl,
      model: view.model,
      region: view.region,
      hasKey: view.hasKey,
      host: view.host,
      isRemote: view.isRemote,
      privacy: line,
    );
  }

  Future<AiSettings> _announced(bridge.AiSettingsView view) async {
    final settings = await _view(view);
    _changes.add(settings);
    return settings;
  }

  @override
  Future<Either<ApplicationError, AiSettings>> settings() =>
      attempt(() async => _view(await _ai.settings()));

  @override
  Stream<AiSettings> settingsChanges() => _changes.stream;

  @override
  Future<Either<ApplicationError, List<AiProviderInfo>>> providers() =>
      attempt(() async {
        return <AiProviderInfo>[
          for (final info in _ai.providers())
            AiProviderInfo(
              key: info.key,
              label: info.label,
              description: info.description,
              defaultBaseUrl: info.defaultBaseUrl,
              defaultModel: info.defaultModel,
              needsKey: info.needsKey,
              acceptsKey: info.acceptsKey,
              usesRegion: info.defaultRegion.isNotEmpty,
              defaultRegion: info.defaultRegion,
            ),
        ];
      });

  @override
  Future<Either<ApplicationError, AiSettings>> saveSettings(
    AiSettingsChange change,
  ) => attempt(
    () async => _announced(
      await _ai.setSettings(
        input: bridge.AiSettingsInput(
          enabled: change.enabled,
          provider: change.provider,
          baseUrl: change.baseUrl,
          model: change.model,
          region: change.region,
        ),
      ),
    ),
  );

  @override
  Future<Either<ApplicationError, AiSettings>> setKey({
    required String provider,
    required String key,
  }) => attempt(() async {
    await _ai.setKey(provider: provider, key: key);
    return _announced(await _ai.settings());
  });

  @override
  Future<Either<ApplicationError, AiTestResult>> test() async {
    final result = await attempt(_ai.test);
    return Right<ApplicationError, AiTestResult>(
      result.fold(
        (error) => AiTestResult.failed(message: error.message),
        (said) => AiTestResult.ok(said: said),
      ),
    );
  }

  @override
  Stream<AskUpdate> ask(String question) => _mapped<bridge.AskEvent, AskUpdate>(
    _conversation.ask(question: question),
    (event, sink) {
      switch (event) {
        case bridge.AskEvent_Searching():
          sink.add(const AskUpdate.searching());
        case bridge.AskEvent_Status():
          sink.add(const AskUpdate.thinking());
        case bridge.AskEvent_Used(:final notes):
          sink.add(AskUpdate.usedNotes(notes: _refs(notes)));
        case bridge.AskEvent_Answer(:final markdown):
          sink.add(AskUpdate.answer(text: markdown));
        case bridge.AskEvent_Done(:final markdown, :final notes):
          sink.add(AskUpdate.done(answer: markdown, used: _refs(notes)));
        case bridge.AskEvent_NothingMatches():
          sink.add(const AskUpdate.nothingMatches());
        case bridge.AskEvent_CutOff():
          sink.add(const AskUpdate.cutOff());
        case bridge.AskEvent_Failed(:final message):
          sink.add(AskUpdate.error(message: message));
        case bridge.AskEvent_Stopped():
          sink.close();
      }
    },
  );

  /// [source] with each event handled by [handle], which adds what the app
  /// hears (or closes the sink). The bridge’s streams are generators waiting
  /// on the core: cancelling one does not finish until its next event, so
  /// the stream returned lets go of [source] at once, and tells [onCancel].
  Stream<R> _mapped<E, R>(
    Stream<E> source,
    void Function(E event, EventSink<R> sink) handle, {
    void Function()? onCancel,
  }) {
    late StreamController<R> controller;
    // StreamController's cancellation callback owns and awaits this source.
    // ignore: cancel_subscriptions
    StreamSubscription<E>? subscription;
    controller = StreamController<R>(
      onListen: () {
        subscription = source.listen(
          (event) => handle(event, controller),
          onError: controller.addError,
          onDone: controller.close,
        );
      },
      onCancel: () {
        onCancel?.call();
        final upstream = subscription;
        subscription = null;
        if (upstream != null) {
          unawaited(upstream.cancel());
        }
      },
    );
    return controller.stream;
  }

  List<AiNoteRef> _refs(List<bridge.AskNote> notes) => <AiNoteRef>[
    for (final note in notes) AiNoteRef(id: note.id, title: note.title),
  ];

  @override
  Future<Either<ApplicationError, Unit>> stop() => attempt(() async {
    _conversation.stop();
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> clear() => attempt(() async {
    _conversation.clear();
    return unit;
  });

  @override
  Future<Either<ApplicationError, bool>> isEligible(String noteId) =>
      attempt(() => _ai.noteAllowed(id: noteId));

  @override
  List<AssistActionInfo> assistActions() => <AssistActionInfo>[
    for (final action in _ai.actions())
      if (assistActionOfKey(action.key) case final known?)
        AssistActionInfo(
          action: known,
          label: action.label,
          noun: action.noun,
          replaces: action.replaces,
          needsSelection: known != AssistAction.continueWriting,
        ),
  ];

  @override
  Stream<AssistUpdate> assist({
    required String noteId,
    required AssistAction action,
    required String before,
    required String selection,
  }) {
    final run = _ai.newRun();
    return _assisted(
      _ai.assist(
        id: noteId,
        action: assistActionKey(action),
        before: before,
        selection: selection,
        run: run,
      ),
      run,
    );
  }

  @override
  Stream<AssistUpdate> summarize(String noteId) {
    final run = _ai.newRun();
    return _assisted(_ai.summarize(id: noteId, run: run), run);
  }

  /// The core’s events as updates; a stream nobody listens to any more stops
  /// the run.
  Stream<AssistUpdate> _assisted(
    Stream<bridge.AssistEvent> events,
    bridge.KansoAiRun run,
  ) => _mapped<bridge.AssistEvent, AssistUpdate>(
    events,
    (event, sink) {
      switch (event) {
        case bridge.AssistEvent_Status():
          break;
        case bridge.AssistEvent_Text(:final text):
          sink.add(AssistUpdate.text(text: text));
        case bridge.AssistEvent_Done(:final text):
          sink.add(AssistUpdate.done(suggestion: text));
        case bridge.AssistEvent_CutOff(:final message):
          sink.add(AssistUpdate.error(message: message));
        case bridge.AssistEvent_Failed(:final message):
          sink.add(
            message == keptOutMessage
                ? const AssistUpdate.keptOut()
                : AssistUpdate.error(message: message),
          );
        case bridge.AssistEvent_Stopped():
          sink.close();
      }
    },
    onCancel: run.stop,
  );
}
