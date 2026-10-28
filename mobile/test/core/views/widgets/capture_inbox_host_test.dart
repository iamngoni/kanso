// Kanso — capture_inbox_host_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/state/capture_inbox/capture_inbox_bloc.dart';
import 'package:kanso/core/views/widgets/capture_inbox_host.dart';
import 'package:mocktail/mocktail.dart';

class _MockCaptureInboxBloc
    extends MockBloc<CaptureInboxEvent, CaptureInboxState>
    implements CaptureInboxBloc {}

void main() {
  late _MockCaptureInboxBloc bloc;

  setUpAll(
    () => registerFallbackValue(const CaptureInboxEvent.saveRequested()),
  );

  setUp(() {
    bloc = _MockCaptureInboxBloc();
    when(() => bloc.state).thenReturn(const CaptureInboxState());
  });

  Future<void> pump(WidgetTester tester) {
    return tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: BlocProvider<CaptureInboxBloc>.value(
          value: bloc,
          child: const CaptureInboxHost(child: Text('app')),
        ),
      ),
    );
  }

  group('CaptureInboxHost', () {
    testWidgets('shows its child and asks for nothing by itself', (
      tester,
    ) async {
      await pump(tester);
      expect(find.text('app'), findsOneWidget);
      verifyNever(() => bloc.add(any()));
    });

    testWidgets('asks for a save each time the app comes back', (tester) async {
      await pump(tester);
      tester.binding
        ..handleAppLifecycleStateChanged(AppLifecycleState.inactive)
        ..handleAppLifecycleStateChanged(AppLifecycleState.hidden)
        ..handleAppLifecycleStateChanged(AppLifecycleState.paused);
      verifyNever(() => bloc.add(any()));
      tester.binding
        ..handleAppLifecycleStateChanged(AppLifecycleState.hidden)
        ..handleAppLifecycleStateChanged(AppLifecycleState.inactive)
        ..handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      verify(
        () => bloc.add(const CaptureInboxEvent.saveRequested()),
      ).called(1);
      tester.binding
        ..handleAppLifecycleStateChanged(AppLifecycleState.inactive)
        ..handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      verify(
        () => bloc.add(const CaptureInboxEvent.saveRequested()),
      ).called(1);
    });

    testWidgets('stops listening when it is removed', (tester) async {
      await pump(tester);
      await tester.pumpWidget(const SizedBox());
      tester.binding
        ..handleAppLifecycleStateChanged(AppLifecycleState.inactive)
        ..handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      verifyNever(() => bloc.add(any()));
    });
  });
}
