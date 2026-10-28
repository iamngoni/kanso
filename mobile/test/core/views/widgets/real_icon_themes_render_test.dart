// Kanso — real_icon_themes_render_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/views/widgets/kanso_icon.dart';
import 'package:kanso/core/views/widgets/kanso_icon_set_scope.dart';

import '../../../helpers/pump.dart';
import '../../../helpers/real_icon_themes.dart';

/// SVGs of the themes that the SVG painter cannot read (`font-size: none`
/// in the notebook of Monospace Icons); `KansoThemedIcon` draws Lucide's icon
/// in their place, which `kanso_icon_test.dart` holds to.
const Set<String> _unreadable = <String>{'monospace-icons/notebook'};

void main() {
  loadRealIconThemes();

  group('the built-in icon themes, as the core gives them', () {
    testWidgets('every SVG of every theme can be drawn, but the few known', (
      tester,
    ) async {
      final failures = <String>[];
      await tester.runAsync(() async {
        for (final MapEntry(key: id, value: icons)
            in realIconThemeIcons.entries) {
          for (final icon in icons.where((icon) => icon.svg != null)) {
            try {
              final info = await vg.loadPicture(
                SvgStringLoader(icon.svg!),
                null,
              );
              info.picture.dispose();
            } on Object {
              failures.add('$id/${icon.name}');
            }
          }
        }
      });
      expect(_unreadable.containsAll(failures), isTrue, reason: '$failures');
    });

    testWidgets('every theme draws every icon of the app, with no error', (
      tester,
    ) async {
      for (final theme in realIconThemes) {
        await pumpKanso(
          tester,
          KansoIconSetScope(
            iconSet: realIconSet(theme.id),
            child: Wrap(
              children: <Widget>[
                for (final icon in KansoIcons.values) KansoIcon(icon),
              ],
            ),
          ),
        );
        expect(tester.takeException(), isNull, reason: theme.id);
        expect(
          find.byType(SvgPicture),
          findsNWidgets(KansoIcons.values.length),
          reason: theme.id,
        );
      }
    });
  });
}
