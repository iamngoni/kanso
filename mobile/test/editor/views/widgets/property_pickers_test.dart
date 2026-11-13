// Kanso — property_pickers_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/views/widgets/kanso_check_row.dart';
import 'package:kanso/core/views/widgets/kanso_month_grid.dart';
import 'package:kanso/editor/views/widgets/properties_section.dart';
import 'package:kanso/editor/views/widgets/property_choice_sheet.dart';
import 'package:kanso/editor/views/widgets/property_moment_sheet.dart';
import 'package:kanso/editor/views/widgets/property_pick_cell.dart';

import '../../../helpers/pump.dart';

Property _stage({String text = 'Idea', bool unlisted = false}) => Property(
  key: 'stage',
  kind: PropertyKind.text,
  text: text,
  items: const <String>[],
  pick: PropertyPick(
    kind: PickKind.select,
    rows: <String>[
      if (unlisted) text,
      'Idea',
      'Shooting',
    ],
    unlisted: unlisted,
    zoneSet: false,
  ),
);

const _owner = Property(
  key: 'owner',
  kind: PropertyKind.text,
  text: '',
  items: <String>[],
  pick: PropertyPick(
    kind: PickKind.person,
    rows: <String>['amy', 'bo'],
    unlisted: false,
    zoneSet: false,
  ),
);

const _due = Property(
  key: 'due',
  kind: PropertyKind.date,
  text: '2026-11-20',
  items: <String>[],
  pick: PropertyPick(
    kind: PickKind.date,
    rows: <String>[],
    unlisted: false,
    moment: Moment(year: 2026, month: 11, day: 20, hour: 0, minute: 0),
    zoneSet: false,
  ),
);

const _publish = Property(
  key: 'publish',
  kind: PropertyKind.dateTime,
  text: '',
  items: <String>[],
  pick: PropertyPick(
    kind: PickKind.dateTime,
    rows: <String>[],
    unlisted: false,
    zoneSet: false,
  ),
);

Property _embargo({bool withTime = true}) => Property(
  key: 'embargo',
  kind: withTime ? PropertyKind.dateTimeZone : PropertyKind.text,
  text: withTime ? '2026-11-18T09:00 America/New_York' : '',
  items: const <String>[],
  pick: PropertyPick(
    kind: PickKind.dateTimeZone,
    rows: const <String>[],
    unlisted: false,
    moment: withTime
        ? const Moment(year: 2026, month: 11, day: 18, hour: 9, minute: 0)
        : null,
    zone: withTime ? 'America/New_York' : 'Africa/Harare',
    zoneSet: withTime,
    label: withTime ? 'Wed 18 Nov, 09:00 New York' : null,
  ),
);

const _zones = <ZoneRow>[
  ZoneRow(name: 'Africa/Harare', label: 'Harare'),
  ZoneRow(name: 'America/New_York', label: 'New York'),
  ZoneRow(name: 'Europe/London', label: 'London'),
];

class _Picks {
  final List<String> picks = <String>[];
  PropertyChoice? last;
}

Future<_Picks> _pump(
  WidgetTester tester,
  List<Property> properties, {
  bool dropped = false,
}) async {
  final picks = _Picks();
  await pumpKanso(
    tester,
    SizedBox(
      width: 361,
      child: PropertiesSection(
        properties: properties,
        open: true,
        dropped: dropped,
        onToggle: () {},
        onSet: (key, value, {index}) {},
        onRemove: (_) {},
        onPick: (index, key, original, choice) {
          picks.picks.add('$index $key "$original"');
          picks.last = choice;
        },
        loadZones: () async => _zones,
      ),
    ),
  );
  return picks;
}

void main() {
  group('a select', () {
    testWidgets('shows its value and picks from the options', (tester) async {
      final picks = await _pump(tester, <Property>[_stage()]);
      expect(find.text('Idea'), findsOneWidget);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      expect(find.byType(PropertyChoiceSheet), findsOneWidget);
      await tester.tap(find.text('Shooting'));
      await tester.pumpAndSettle();
      expect(picks.picks, <String>['0 stage "Idea"']);
      expect(picks.last, const PropertyChoice.option(text: 'Shooting'));
      expect(find.byType(PropertyChoiceSheet), findsNothing);
    });

    testWidgets('clears its value', (tester) async {
      final picks = await _pump(tester, <Property>[_stage()]);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Clear'));
      await tester.pumpAndSettle();
      expect(picks.last, const PropertyChoice.clear());
    });

    testWidgets('says Choose while empty and offers no Clear', (tester) async {
      await _pump(tester, <Property>[_stage(text: '')]);
      expect(find.text('Choose'), findsOneWidget);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      expect(find.text('Clear'), findsNothing);
    });

    testWidgets('keeps a value the notebook no longer lists, and says so', (
      tester,
    ) async {
      await _pump(tester, <Property>[_stage(text: 'Archived', unlisted: true)]);
      expect(find.text('Archived'), findsOneWidget);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      expect(find.text("Not in the notebook's list"), findsOneWidget);
      expect(find.text('Archived'), findsWidgets);
    });

    testWidgets('dismissing the list picks nothing', (tester) async {
      final picks = await _pump(tester, <Property>[_stage()]);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      await tester.tapAt(const Offset(10, 10));
      await tester.pumpAndSettle();
      expect(picks.picks, isEmpty);
    });
  });

  group('a person', () {
    testWidgets('picks one of the members', (tester) async {
      final picks = await _pump(tester, <Property>[_owner]);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      expect(find.text('amy'), findsOneWidget);
      await tester.tap(find.text('bo'));
      await tester.pumpAndSettle();
      expect(picks.last, const PropertyChoice.option(text: 'bo'));
    });

    testWidgets('is a text field outside a shared note', (tester) async {
      await _pump(tester, const <Property>[
        Property(
          key: 'owner',
          kind: PropertyKind.text,
          text: 'amy',
          items: <String>[],
        ),
      ]);
      expect(find.byType(PropertyPickCell), findsNothing);
      expect(find.byType(EditableText), findsOneWidget);
    });
  });

  group('a date', () {
    testWidgets('shows the day and picks another', (tester) async {
      final picks = await _pump(tester, <Property>[_due]);
      expect(find.textContaining('Nov 20, 2026'), findsOneWidget);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      expect(find.byType(KansoMonthGrid), findsOneWidget);
      expect(find.text('Time'), findsNothing);
      await tester.tap(find.text('25'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      expect(picks.picks, <String>['0 due "2026-11-20"']);
      expect(
        picks.last,
        const PropertyChoice.day(
          moment: Moment(year: 2026, month: 11, day: 25, hour: 0, minute: 0),
        ),
      );
    });

    testWidgets('clears its value', (tester) async {
      final picks = await _pump(tester, <Property>[_due]);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Clear'));
      await tester.pumpAndSettle();
      expect(picks.last, const PropertyChoice.clear());
    });

    testWidgets('steps the month', (tester) async {
      final picks = await _pump(tester, <Property>[_due]);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      expect(find.text('November 2026'), findsOneWidget);
      await tester.tap(find.bySemanticsLabel('Next month'));
      await tester.pumpAndSettle();
      expect(find.text('December 2026'), findsOneWidget);
      await tester.tap(find.text('3').first);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      expect(
        picks.last,
        const PropertyChoice.day(
          moment: Moment(year: 2026, month: 12, day: 3, hour: 0, minute: 0),
        ),
      );
    });

    testWidgets('cannot be done before a day is picked', (tester) async {
      final picks = await _pump(tester, <Property>[
        const Property(
          key: 'due',
          kind: PropertyKind.date,
          text: '',
          items: <String>[],
          pick: PropertyPick(
            kind: PickKind.date,
            rows: <String>[],
            unlisted: false,
            zoneSet: false,
          ),
        ),
      ]);
      expect(find.text('Pick a date'), findsOneWidget);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      expect(picks.picks, isEmpty);
      expect(find.byType(PropertyMomentSheet), findsOneWidget);
    });
  });

  group('a date and time', () {
    testWidgets('picks a day at the default time', (tester) async {
      final picks = await _pump(tester, <Property>[_publish]);
      expect(find.text('Pick a date and time'), findsOneWidget);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      expect(find.text('Time'), findsOneWidget);
      expect(find.text('09:00'), findsOneWidget);
      // Today's month opens; any day of it that is in it will do.
      await tester.tap(find.text('15').first);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      final choice = picks.last;
      expect(choice, isA<PropertyChoice_Moment>());
      final moment = (choice! as PropertyChoice_Moment).moment;
      expect((moment.day, moment.hour, moment.minute), (15, 9, 0));
    });

    testWidgets('opens the clock from the time row', (tester) async {
      await _pump(tester, <Property>[_publish]);
      await tester.tap(find.byType(PropertyPickCell));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Time'));
      await tester.pumpAndSettle();
      expect(find.byType(ListWheelScrollView), findsWidgets);
    });
  });

  group('a date and time with a zone', () {
    testWidgets('shows the time and the zone, and picks a zone', (
      tester,
    ) async {
      final picks = await _pump(tester, <Property>[_embargo()]);
      expect(find.textContaining('Nov 18, 2026 · 09:00'), findsOneWidget);
      expect(find.text('New York'), findsOneWidget);
      await tester.tap(find.text('New York'));
      await tester.pumpAndSettle();
      // A zone has a search field and a row for each of the zones.
      expect(find.text('London'), findsOneWidget);
      await tester.tap(find.text('London'));
      await tester.pumpAndSettle();
      expect(picks.picks, <String>[
        '0 embargo "2026-11-18T09:00 America/New_York"',
      ]);
      expect(picks.last, const PropertyChoice.zone(zone: 'Europe/London'));
    });

    testWidgets('writes a picked time in the zone it shows', (tester) async {
      final picks = await _pump(tester, <Property>[_embargo()]);
      await tester.tap(find.textContaining('Nov 18, 2026'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('19'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      expect(
        picks.last,
        const PropertyChoice.zoned(
          moment: Moment(year: 2026, month: 11, day: 19, hour: 9, minute: 0),
          zone: 'America/New_York',
        ),
      );
    });

    testWidgets('keeps a zone picked before there is a time', (tester) async {
      final picks = await _pump(tester, <Property>[_embargo(withTime: false)]);
      expect(find.text('Pick a date and time'), findsOneWidget);
      // The device's zone is offered.
      expect(find.text('Harare'), findsOneWidget);
      await tester.tap(find.text('Harare'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('London'));
      await tester.pumpAndSettle();
      // Nothing to attach it to: nothing is written.
      expect(picks.picks, isEmpty);
      expect(find.text('London'), findsOneWidget);
      await tester.tap(find.text('Pick a date and time'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('15').first);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      expect(picks.last, isA<PropertyChoice_Zoned>());
      expect((picks.last! as PropertyChoice_Zoned).zone, 'Europe/London');
    });

    testWidgets('lists every zone', (tester) async {
      await _pump(tester, <Property>[_embargo()]);
      await tester.tap(find.text('New York'));
      await tester.pumpAndSettle();
      expect(find.text('Harare'), findsOneWidget);
    });
  });

  group('the dropped notice', () {
    testWidgets('is shown when a pick was dropped', (tester) async {
      await _pump(tester, <Property>[_stage()], dropped: true);
      expect(
        find.text('That property changed while you were editing it.'),
        findsOneWidget,
      );
    });

    testWidgets('is not shown otherwise', (tester) async {
      await _pump(tester, <Property>[_stage()]);
      expect(
        find.text('That property changed while you were editing it.'),
        findsNothing,
      );
    });
  });

  group('the list sheet', () {
    testWidgets('searches over many rows', (tester) async {
      final rows = <PropertyChoiceRow>[
        for (var i = 0; i < 12; i++) PropertyChoiceRow('v$i', 'Row $i'),
      ];
      await pumpKanso(
        tester,
        Builder(
          builder: (context) => TextButton(
            onPressed: () => PropertyChoiceSheet.show(
              context,
              title: 'T',
              rows: rows,
              selected: null,
              searchHint: 'Search',
              emptyLabel: 'None',
            ),
            child: const Text('open'),
          ),
        ),
      );
      await tester.tap(find.text('open'));
      await tester.pumpAndSettle();
      expect(find.text('Search'), findsOneWidget);
      await tester.enterText(find.byType(EditableText), 'Row 11');
      await tester.pumpAndSettle();
      expect(find.byType(KansoCheckRow), findsOneWidget);
      expect(find.text('Row 2'), findsNothing);
      await tester.enterText(find.byType(EditableText), 'zzz');
      await tester.pumpAndSettle();
      expect(find.text('None'), findsOneWidget);
    });
  });
}
