// Kanso — dictation_key_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/dictation/models/repos/impl/plain_dictation_text_rules.dart';
import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';
import 'package:kanso/dictation/utils/dictation_editor_extras.dart';
import 'package:kanso/dictation/views/widgets/dictation_pill.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/golden.dart';
import '../../helpers/accessory_strip.dart';
import '../../helpers/fake_text_target.dart';
import '../../helpers/mock_dictation_repository.dart';
import '../../helpers/seeded_dictation_bloc.dart';

void main() {
  // Widths of the phone, an iPad in both ways round and an Android tablet.
  final widths = <String, double>{
    'phone': phoneSize.width,
    'ipad_portrait': tabletPortraitSize.width,
    'ipad_landscape': tabletLandscapeSize.width,
    'android_tablet': 1280,
  };
  const quiet = 'Listening';
  const spoken = 'Call the plumber about the kitchen tap';
  final states = <String, (bool, String?)>{
    'off': (false, null),
    'listening': (true, quiet),
    'words': (true, spoken),
  };

  for (final brightness in Brightness.values) {
    group('the dictation key and pill, ${brightness.name}', () {
      for (final MapEntry(key: slug, value: width) in widths.entries) {
        for (final MapEntry(key: label, value: state) in states.entries) {
          testWidgets('$label at $slug', (tester) async {
            final bloc = SeededDictationBloc(
              const DictationState(),
              SpokenRepository(),
            );
            closeBlocLater(bloc);
            final extras = DictationEditorExtras(
              bloc: bloc,
              rules: PlainDictationTextRules(),
            );
            addTearDown(extras.dispose);
            extras.listening.value = state.$1;
            final text = state.$2;
            await expectGoldenOnBothPlatforms(
              tester,
              AccessoryStrip(
                extra: extras.accessoryKeys(FakeTextTarget('x')).single,
                pill: text == null
                    ? null
                    : DictationPill(
                        text: text,
                        quiet: text == quiet,
                        actionLabel: 'Stop',
                        onAction: () {},
                      ),
              ),
              name: 'dictation_key_${label}_${slug}_${brightness.name}',
              size: Size(width, 200),
              safeArea: EdgeInsets.zero,
              brightness: brightness,
            );
          });
        }
      }
    });
  }
}
