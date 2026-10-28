// Kanso — widget_colors_host_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/appearance_status.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/state/widget_colors/widget_colors_bloc.dart';
import 'package:kanso/core/views/widgets/widget_colors_host.dart';
import 'package:mocktail/mocktail.dart';

class _MockAppearanceBloc extends MockBloc<AppearanceEvent, AppearanceState>
    implements AppearanceBloc {}

class _MockWidgetColorsBloc
    extends MockBloc<WidgetColorsEvent, WidgetColorsState>
    implements WidgetColorsBloc {}

void main() {
  late _MockWidgetColorsBloc colors;
  late _MockAppearanceBloc appearance;
  late StreamController<AppearanceState> states;

  setUpAll(
    () => registerFallbackValue(
      const WidgetColorsEvent.changed(
        light: KansoColors.light,
        dark: KansoColors.dark,
      ),
    ),
  );

  setUp(() {
    colors = _MockWidgetColorsBloc();
    when(() => colors.state).thenReturn(const WidgetColorsState());
    appearance = _MockAppearanceBloc();
    states = StreamController<AppearanceState>.broadcast();
    addTearDown(states.close);
    whenListen(
      appearance,
      states.stream,
      initialState: const AppearanceState(),
    );
  });

  Future<void> pump(WidgetTester tester) {
    return tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: MultiBlocProvider(
          providers: <BlocProvider<dynamic>>[
            BlocProvider<AppearanceBloc>.value(value: appearance),
            BlocProvider<WidgetColorsBloc>.value(value: colors),
          ],
          child: const WidgetColorsHost(child: Text('app')),
        ),
      ),
    );
  }

  group('WidgetColorsHost', () {
    testWidgets('tells the bloc when the light or dark theme changes', (
      tester,
    ) async {
      await pump(tester);
      verifyNever(() => colors.add(any()));
      final nord = KansoPalette.kansoDark.copyWith(
        id: 'nord',
        name: 'Nord',
        colors: KansoColors.dark.copyWith(accent: const Color(0xFF88C0D0)),
      );
      states.add(AppearanceState(dark: nord));
      await tester.pump();
      final captured = verify(() => colors.add(captureAny())).captured;
      expect(captured, hasLength(1));
      final event = captured.single as WidgetColorsChangedEvent;
      expect(event.dark, nord.colors);
      expect(event.light, KansoColors.light);
    });

    testWidgets('a change that is not of the themes tells it nothing', (
      tester,
    ) async {
      await pump(tester);
      states.add(const AppearanceState(status: AppearanceStatus.ready));
      await tester.pump();
      verifyNever(() => colors.add(any()));
    });
  });
}
