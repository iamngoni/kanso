// Kanso — kanso_icon_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, any;
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/models/data/kanso_icon_set.dart';
import 'package:kanso/core/views/widgets/kanso_icon.dart';
import 'package:kanso/core/views/widgets/kanso_icon_set_scope.dart';
import 'package:kanso/core/views/widgets/kanso_themed_icon.dart';

import '../../../helpers/property.dart';
import '../../../helpers/pump.dart';

const String _svg =
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">\n'
    '<path d="M4 12h16" stroke="black"/>\n'
    '</svg>';

SvgLoader<dynamic> _loader(WidgetTester tester) {
  return tester.widget<SvgPicture>(find.byType(SvgPicture)).bytesLoader
      as SvgLoader<dynamic>;
}

void main() {
  group('KansoIcon', () {
    testWidgets('draws the bundled Lucide icon where no theme is above', (
      tester,
    ) async {
      await pumpKanso(tester, const KansoIcon(KansoIcons.search));
      expect(
        _loader(tester),
        const SvgAssetLoader('assets/icons/lucide/search.svg'),
      );
    });

    testWidgets("draws the chosen theme's SVG for an icon it draws", (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const KansoIconSetScope(
          iconSet: KansoIconSet(
            id: 'x',
            name: 'X',
            svgs: <String, String>{'search': _svg},
          ),
          child: KansoIcon(KansoIcons.search),
        ),
      );
      expect(_loader(tester), const SvgStringLoader(_svg));
    });

    testWidgets('draws Lucide, icon by icon, for one the theme leaves out', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const KansoIconSetScope(
          iconSet: KansoIconSet(
            id: 'x',
            name: 'X',
            svgs: <String, String>{'search': _svg},
          ),
          child: KansoIcon(KansoIcons.plus),
        ),
      );
      expect(
        _loader(tester),
        const SvgAssetLoader('assets/icons/lucide/plus.svg'),
      );
    });

    testWidgets('redraws at once when the theme changes', (tester) async {
      Widget build(KansoIconSet set) => KansoIconSetScope(
        iconSet: set,
        child: const KansoIcon(KansoIcons.search),
      );
      await pumpKanso(tester, build(KansoIconSet.lucide));
      expect(_loader(tester), isA<SvgAssetLoader>());
      await pumpKanso(
        tester,
        build(
          const KansoIconSet(
            id: 'x',
            name: 'X',
            svgs: <String, String>{'search': _svg},
          ),
        ),
      );
      expect(_loader(tester), const SvgStringLoader(_svg));
      await pumpKanso(tester, build(KansoIconSet.lucide));
      expect(_loader(tester), isA<SvgAssetLoader>());
    });

    testWidgets('keeps its size and colour whatever the theme draws', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const KansoIconSetScope(
          iconSet: KansoIconSet(svgs: <String, String>{'search': _svg}),
          child: KansoIcon(
            KansoIcons.search,
            size: 31,
            color: Color(0xFF123456),
          ),
        ),
      );
      final picture = tester.widget<SvgPicture>(find.byType(SvgPicture));
      expect(picture.width, 31);
      expect(picture.height, 31);
      expect(
        picture.colorFilter,
        const ColorFilter.mode(Color(0xFF123456), BlendMode.srcIn),
      );
    });
  });

  group('KansoThemedIcon', () {
    testWidgets('draws Lucide when it has no SVG', (tester) async {
      await pumpKanso(tester, const KansoThemedIcon(lucideName: 'check'));
      expect(
        _loader(tester),
        const SvgAssetLoader('assets/icons/lucide/check.svg'),
      );
    });

    testWidgets('draws the SVG it is given', (tester) async {
      await pumpKanso(
        tester,
        const KansoThemedIcon(lucideName: 'check', svg: _svg),
      );
      expect(_loader(tester), const SvgStringLoader(_svg));
    });

    testWidgets("shows the Lucide icon when the theme's SVG is broken", (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const KansoThemedIcon(lucideName: 'check', svg: '<svg><path d='),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      expect(
        find.byWidgetPredicate(
          (widget) =>
              widget is SvgPicture &&
              widget.bytesLoader ==
                  const SvgAssetLoader('assets/icons/lucide/check.svg'),
        ),
        findsOneWidget,
      );
    });
  });

  group('KansoIconSet', () {
    test("Kanso's own icons draw nothing over Lucide", () {
      expect(KansoIconSet.lucide.id, 'lucide');
      expect(KansoIconSet.lucide.svgs, isEmpty);
      expect(KansoIconSet.lucide.svgFor('search'), isNull);
    });

    Glados<KansoIcons>(
      any.kansoIcon,
      kansoExploreConfig(),
    ).test('answers only for the icons it holds', (icon) {
      final set = KansoIconSet(
        svgs: <String, String>{icon.lucideName: _svg},
      );
      expect(set.svgFor(icon.lucideName), _svg);
      for (final other in KansoIcons.values.where((o) => o != icon)) {
        expect(set.svgFor(other.lucideName), isNull);
      }
    });
  });
}
