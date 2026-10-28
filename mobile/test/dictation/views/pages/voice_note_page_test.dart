// Kanso — voice_note_page_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/dictation/models/data/dictation_capability.dart';
import 'package:kanso/dictation/models/data/dictation_permission.dart';
import 'package:kanso/dictation/models/data/dictation_status.dart';
import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';
import 'package:kanso/dictation/state/voice_note/voice_note_bloc.dart';
import 'package:kanso/dictation/views/pages/voice_note_page.dart';
import 'package:kanso/injection.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/pump_routed.dart';
import '../../helpers/fake_dictation_text_rules.dart';
import '../../helpers/mock_dictation_repository.dart';

void main() {
  testWidgets('checks permission again when the app resumes', (tester) async {
    final spoken = SpokenRepository(
      capability: const DictationCapability(
        available: true,
        onDevice: true,
        permission: DictationPermission.denied,
      ),
    );
    var permission = DictationPermission.denied;
    when(
      () => spoken.repository.availability(localeTag: any(named: 'localeTag')),
    ).thenAnswer(
      (_) async => Right<ApplicationError, DictationCapability>(
        DictationCapability(
          available: true,
          onDevice: true,
          permission: permission,
        ),
      ),
    );
    final dictation = DictationBloc(spoken.repository, PassThroughRules());
    final library = MockLibraryRepository();
    getIt.registerFactory<VoiceNoteBloc>(() => VoiceNoteBloc(library));
    addTearDown(() async {
      await getIt.reset();
      spoken.close();
    });
    closeBlocLater(dictation);

    await pumpRouted(
      tester,
      BlocProvider<DictationBloc>.value(
        value: dictation,
        child: const VoiceNotePage(),
      ),
    );
    await tester.pumpAndSettle();
    expect(dictation.state.status, DictationStatus.denied);

    permission = DictationPermission.granted;
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    await tester.pump();

    expect(dictation.state.status, DictationStatus.idle);
    verify(
      () => spoken.repository.availability(localeTag: any(named: 'localeTag')),
    ).called(2);
  });
}
