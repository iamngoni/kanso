// Kanso — voice_settings_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/core/models/data/ai_settings.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/views/widgets/kanso_switch.dart';
import 'package:kanso/dictation/models/data/dictation_capability.dart';
import 'package:kanso/dictation/models/data/dictation_permission.dart';
import 'package:kanso/dictation/models/data/dictation_unavailable_reason.dart';
import 'package:kanso/voice_settings/models/data/voice_settings.dart';
import 'package:kanso/voice_settings/models/data/voice_tidy_mode.dart';
import 'package:kanso/voice_settings/state/voice_settings/voice_settings_bloc.dart';
import 'package:kanso/voice_settings/views/widgets/voice_settings_pane.dart';
import 'package:mocktail/mocktail.dart';

import '../../../dictation/helpers/mock_dictation_repository.dart';
import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/pump_routed.dart';
import '../../helpers/mock_voice_settings_repository.dart';

void main() {
  late MockVoiceSettingsRepository settings;
  late SpokenRepository spoken;

  setUpAll(() => registerFallbackValue(const VoiceSettings()));

  setUp(() {
    settings = MockVoiceSettingsRepository();
    when(settings.settings).thenAnswer(
      (_) async => const Right<ApplicationError, VoiceSettings>(
        VoiceSettings(),
      ),
    );
    when(() => settings.save(any())).thenAnswer(
      (invocation) async => Right<ApplicationError, VoiceSettings>(
        invocation.positionalArguments.first as VoiceSettings,
      ),
    );
  });

  tearDown(() => spoken.close());

  Future<void> pump(
    WidgetTester tester,
    DictationCapability capability, {
    bool ai = false,
  }) async {
    spoken = SpokenRepository(capability: capability);
    final bloc = VoiceSettingsBloc(settings, spoken.repository)
      ..add(const VoiceSettingsEvent.started());
    closeBlocLater(bloc);
    final status = AiStatusBloc(
      FixtureAiRepository(settings: AiSettings(enabled: ai)),
    )..add(const AiStatusEvent.started());
    closeBlocLater(status);
    await pumpRouted(
      tester,
      MultiBlocProvider(
        providers: <BlocProvider<dynamic>>[
          BlocProvider<VoiceSettingsBloc>.value(value: bloc),
          BlocProvider<AiStatusBloc>.value(value: status),
        ],
        child: const VoiceSettingsPane(),
      ),
    );
    await tester.pumpAndSettle();
  }

  testWidgets('shows how voice input works and that the phone can listen', (
    tester,
  ) async {
    await pump(
      tester,
      const DictationCapability(
        available: true,
        onDevice: true,
        permission: DictationPermission.granted,
        language: 'English',
      ),
    );
    expect(find.text('Voice'), findsOneWidget);
    expect(find.text('Spoken punctuation and commands'), findsOneWidget);
    expect(find.textContaining('"Delete that" takes out'), findsOneWidget);
    expect(
      find.text('On-device recognition is available for English'),
      findsOneWidget,
    );
    expect(find.text("Kanso can't use the microphone"), findsNothing);
    expect(
      tester.widget<KansoSwitch>(find.byType(KansoSwitch).first).value,
      isTrue,
    );
  });

  testWidgets('the switch keeps the choice', (tester) async {
    await pump(
      tester,
      const DictationCapability(
        available: true,
        onDevice: true,
        permission: DictationPermission.granted,
        language: 'English',
      ),
    );
    await tester.tap(find.byType(KansoSwitch).first);
    await tester.pumpAndSettle();
    verify(
      () => settings.save(const VoiceSettings(spokenCommands: false)),
    ).called(1);
    expect(
      tester.widget<KansoSwitch>(find.byType(KansoSwitch).first).value,
      isFalse,
    );
  });

  testWidgets('says when speech cannot be recognized on the phone', (
    tester,
  ) async {
    await pump(
      tester,
      const DictationCapability(
        available: false,
        onDevice: false,
        permission: DictationPermission.granted,
        language: 'Shona',
        reason: DictationUnavailableReason.notOnDevice,
      ),
    );
    expect(
      find.text("On-device recognition isn't available for Shona"),
      findsOneWidget,
    );
  });

  testWidgets('a refused microphone has a row that opens Settings', (
    tester,
  ) async {
    await pump(
      tester,
      const DictationCapability(
        available: true,
        onDevice: true,
        permission: DictationPermission.denied,
        language: 'English',
      ),
    );
    await tester.ensureVisible(find.text("Kanso can't use the microphone"));
    await tester.tap(find.text("Kanso can't use the microphone"));
    await tester.pump();
    verify(spoken.repository.openSettings).called(1);
  });

  const granted = DictationCapability(
    available: true,
    onDevice: true,
    permission: DictationPermission.granted,
    language: 'English',
  );

  testWidgets('Tidy asks for AI to be set up when it is off', (tester) async {
    await pump(tester, granted);
    expect(find.text('Tidy voice notes'), findsOneWidget);
    expect(find.text('Ask each time'), findsNothing);
    expect(find.textContaining('Set up AI'), findsOneWidget);
  });

  testWidgets('Tidy is chosen from three, and the choice is kept', (
    tester,
  ) async {
    await pump(tester, granted, ai: true);
    expect(find.text('Off'), findsOneWidget);
    expect(find.text('Ask each time'), findsOneWidget);
    await tester.tap(find.text('Always'));
    await tester.pumpAndSettle();
    final kept =
        verify(() => settings.save(captureAny())).captured.last
            as VoiceSettings;
    expect(kept.tidy, VoiceTidyMode.always);
  });

  testWidgets('the Keep audio and Meetings switches keep their choices', (
    tester,
  ) async {
    await pump(tester, granted);
    await tester.tap(find.text('Keep audio'));
    await tester.pumpAndSettle();
    var kept =
        verify(() => settings.save(captureAny())).captured.last
            as VoiceSettings;
    expect(kept.keepAudio, isTrue);
    await tester.tap(find.text('Meetings'));
    await tester.pumpAndSettle();
    kept =
        verify(() => settings.save(captureAny())).captured.last
            as VoiceSettings;
    expect(kept.meetings, isFalse);
  });
}
