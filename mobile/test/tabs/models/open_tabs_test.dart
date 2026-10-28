// Kanso — open_tabs_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show
        Any,
        CombinableAny,
        Generator,
        Glados,
        Glados2,
        Glados3,
        IntAnys,
        ListAnys,
        any;
import 'package:kanso/tabs/models/data/open_tabs.dart';
import 'package:kanso/tabs/models/data/tab_meta.dart';

import '../../helpers/property.dart';

TabMeta _meta(String id) => TabMeta(id: id, title: 'Note $id');

bool _all(String id) => true;

OpenTabs _row(List<String> ids, {int? active}) {
  var tabs = const OpenTabs();
  for (final id in ids) {
    tabs = tabs.openInNew(_meta(id));
  }
  return active == null ? tabs : tabs.activate(active);
}

List<String> _ids(OpenTabs tabs) => tabs.ids;

/// One step of a run through the tabs, as a kind from 0 to 11 and two
/// numbers from 0 to 7 (a note, a place in the row).
typedef _Op = (int, int, int);

OpenTabs _apply(OpenTabs tabs, _Op op, bool Function(String id) live) {
  final (kind, a, b) = op;
  final id = '$a';
  switch (kind) {
    case 0 || 1:
      return live(id) ? tabs.openNote(_meta(id)) : tabs;
    case 2 || 3:
      return live(id) ? tabs.openInNew(_meta(id)) : tabs;
    case 4:
      return tabs.close(a);
    case 5:
      return tabs.closeOthers(a);
    case 6:
      return tabs.closeRight(a);
    case 7:
      return tabs.removeNote(id);
    case 8:
      return tabs.reorder(a, b);
    case 9:
      return tabs.next();
    case 10:
      return tabs.previous();
    case 11:
      return tabs.goTo(a + b);
    default:
      return tabs.reopenClosed(live).tabs;
  }
}

extension _TabsAny on Any {
  Generator<_Op> get tabOp => combine3(
    intInRange(0, 13),
    intInRange(0, 8),
    intInRange(0, 8),
    (int kind, int a, int b) => (kind, a, b),
  );
}

void main() {
  group('OpenTabs', () {
    test('a note opened in a new tab goes after the showing one', () {
      var tabs = const OpenTabs().openNote(_meta('a'));
      tabs = tabs.openInNew(_meta('b')).openInNew(_meta('c'));
      expect(_ids(tabs), <String>['a', 'b', 'c']);
      expect(tabs.activeId, 'c');
      tabs = tabs.activate(0).openInNew(_meta('d'));
      expect(_ids(tabs), <String>['a', 'd', 'b', 'c']);
      expect(tabs.activeId, 'd');
    });

    test('a note already open is switched to', () {
      var tabs = const OpenTabs().openNote(_meta('a')).openInNew(_meta('b'));
      tabs = tabs.openNote(_meta('a'));
      expect(_ids(tabs), <String>['a', 'b'], reason: 'no note in two tabs');
      expect(tabs.activeId, 'a');
      tabs = tabs.openNote(_meta('c'));
      expect(
        _ids(tabs),
        <String>['c', 'b'],
        reason: 'the showing tab takes the note',
      );
      expect(tabs.activeId, 'c');
    });

    test('the only tab cannot be closed', () {
      final tabs = const OpenTabs().openNote(_meta('a'));
      expect(identical(tabs.close(0), tabs), isTrue);
      expect(_ids(tabs.close(0)), <String>['a']);
    });

    test('closing shows the neighbour', () {
      var tabs = _row(<String>['a', 'b', 'c']);
      expect(tabs.activeId, 'c');
      tabs = tabs.close(2);
      expect(_ids(tabs), <String>['a', 'b']);
      expect(tabs.activeId, 'b');
      tabs = tabs.close(0);
      expect(_ids(tabs), <String>['b']);
      expect(tabs.activeId, 'b');
    });

    test('closing right and others keep their tab showing', () {
      var tabs = _row(<String>['a', 'b', 'c', 'd'], active: 1);
      tabs = tabs.closeRight(1);
      expect(_ids(tabs), <String>['a', 'b']);
      expect(tabs.activeId, 'b');
      tabs = tabs.closeOthers(1);
      expect(_ids(tabs), <String>['b']);
      expect(tabs.activeId, 'b');
      expect(
        identical(tabs.closeOthers(0), tabs),
        isTrue,
        reason: 'nothing left to close',
      );
    });

    test('a closed tab reopens where it was', () {
      var tabs = _row(<String>['a', 'b', 'c'], active: 1).close(1);
      expect(_ids(tabs), <String>['a', 'c']);
      expect(tabs.activeId, 'c');
      final reopened = tabs.reopenClosed(_all);
      tabs = reopened.tabs;
      expect(reopened.reopened?.id, 'b');
      expect(_ids(tabs), <String>['a', 'b', 'c']);
      expect(tabs.activeId, 'b');
    });

    test('a closed tab of a gone note does not reopen', () {
      final tabs = _row(<String>['a', 'b']).close(1);
      final reopened = tabs.reopenClosed((id) => id == 'a');
      expect(reopened.reopened, isNull);
      expect(_ids(reopened.tabs), <String>['a']);
      expect(reopened.tabs.closed, isEmpty);
    });

    test('a closed tab whose note is open again does not reopen twice', () {
      var tabs = _row(<String>['a', 'b']).close(1);
      tabs = tabs.openInNew(_meta('b'));
      final reopened = tabs.reopenClosed(_all);
      expect(reopened.reopened, isNull);
      expect(_ids(reopened.tabs), <String>['a', 'b']);
    });

    test('next and previous wrap, and 9 is the last', () {
      final tabs = _row(<String>['a', 'b', 'c']);
      expect(tabs.next().active, 0);
      expect(tabs.previous().active, 1);
      expect(tabs.next().previous().active, 2);
      expect(tabs.goTo(2).active, 1);
      expect(tabs.goTo(9).active, 2);
      expect(identical(tabs.goTo(4), tabs), isTrue);
      expect(identical(tabs.goTo(0), tabs), isTrue);
      final one = _row(<String>['a']);
      expect(identical(one.next(), one), isTrue);
      expect(identical(one.previous(), one), isTrue);
    });

    test('a note removed from the library closes its tab', () {
      var tabs = _row(<String>['a', 'b', 'c'], active: 1);
      tabs = tabs.removeNote('b');
      expect(_ids(tabs), <String>['a', 'c']);
      expect(tabs.activeId, 'c');
      tabs = tabs.removeNote('a').removeNote('c');
      expect(tabs.isEmpty, isTrue, reason: 'no blank tab is left behind');
      expect(tabs.active, 0);
      expect(identical(tabs.removeNote('a'), tabs), isTrue);
      expect(tabs.closed, isEmpty, reason: 'not kept to reopen');
    });

    test('a rename updates the title, showing or not', () {
      var tabs = const OpenTabs().openNote(_meta('a'));
      tabs = tabs.updateNote('a', title: 'New title', locked: false);
      expect(tabs.open[0].meta.title, 'New title');
      tabs = tabs.openInNew(_meta('b'));
      tabs = tabs.updateNote('a', title: 'Elsewhere', locked: true);
      expect(tabs.open[0].meta.title, 'Elsewhere');
      expect(tabs.open[0].meta.locked, isTrue);
      expect(tabs.open[1].meta.locked, isFalse);
    });

    test('an empty title shows as Untitled', () {
      const blank = TabMeta(id: 'note:a', title: '  ');
      expect(blank.label('Untitled'), 'Untitled');
      expect(blank.hasTitle, isFalse);
      const named = TabMeta(id: 'note:a', title: 'Plans', locked: true);
      expect(named.label('Untitled'), 'Plans');
    });

    test('restored tabs drop repeats and clamp the showing one', () {
      final tabs = OpenTabs.restore(
        <TabMeta>[_meta('a'), _meta('b'), _meta('a'), _meta('c')],
        7,
      );
      expect(_ids(tabs), <String>['a', 'b', 'c']);
      expect(tabs.active, 2);
      expect(OpenTabs.restore(const <TabMeta>[], 3).active, 0);
      expect(OpenTabs.restore(<TabMeta>[_meta('a')], -2).active, 0);
    });

    test('every tab has its own history', () {
      var tabs = const OpenTabs().openNote(_meta('a')).openNote(_meta('b'));
      tabs = tabs.openInNew(_meta('x')).openNote(_meta('y'));
      expect(tabs.history?.visits.$1, <String>['x', 'y']);
      tabs = tabs.activate(0);
      expect(tabs.history?.visits.$1, <String>['a', 'b']);
      final step = tabs.stepActive(back: true, live: _all);
      expect(step.landed, 'a');
      tabs = step.tabs.setActiveMeta(_meta('a'));
      expect(tabs.activeId, 'a');
      expect(tabs.open[1].history.visits.$1, <String>['x', 'y']);
      expect(tabs.history?.canGoForward(_all), isTrue);
    });

    test('going back stays in its own tab when the note is open elsewhere', () {
      var tabs = const OpenTabs().openNote(_meta('a')).openNote(_meta('b'));
      tabs = tabs.openInNew(_meta('a'));
      expect(_ids(tabs), <String>['b', 'a'], reason: 'a was only a visit');
      tabs = tabs.activate(0);
      final step = tabs.stepActive(back: true, live: _all);
      expect(step.landed, 'a');
      tabs = step.tabs.setActiveMeta(_meta('a'));
      expect(tabs.active, 0, reason: 'the same tab');
      expect(tabs.open[1].meta.id, 'a', reason: 'the other tab stays');
      expect(tabs.history?.current, 'a');
    });

    test('a step with nowhere to land drops the passed-over visits', () {
      var tabs = const OpenTabs().openNote(_meta('a')).openNote(_meta('b'));
      final step = tabs.stepActive(back: true, live: (id) => id != 'a');
      tabs = step.tabs;
      expect(step.landed, isNull);
      expect(tabs.history?.current, 'b');
      expect(tabs.history?.back, isEmpty);
    });

    test('at most 25 closed tabs are kept, the oldest forgotten', () {
      var tabs = _row(<String>[for (var n = 0; n < 40; n++) 'n$n']);
      for (var n = 0; n < 30; n++) {
        tabs = tabs.close(0);
      }
      expect(tabs.closed.length, mostClosed);
      expect(tabs.closed.first.meta.id, 'n5');
      expect(tabs.closed.last.meta.id, 'n29');
    });

    test('closing others remembers each closed tab where it stood', () {
      final tabs = _row(<String>['a', 'b', 'c'], active: 1).closeOthers(1);
      expect(
        <(String, int)>[
          for (final tab in tabs.closed) (tab.meta.id, tab.at),
        ],
        <(String, int)>[('a', 0), ('c', 2)],
      );
    });

    test('a drag keeps the showing note showing', () {
      final tabs = _row(<String>['a', 'b', 'c', 'd'], active: 1).reorder(1, 3);
      expect(_ids(tabs), <String>['a', 'c', 'd', 'b']);
      expect(tabs.activeId, 'b');
      expect(tabs.active, 3);
      final other = _row(<String>['a', 'b', 'c'], active: 0).reorder(2, 0);
      expect(_ids(other), <String>['c', 'a', 'b']);
      expect(other.activeId, 'a');
      expect(identical(other.reorder(1, 1), other), isTrue);
      expect(identical(other.reorder(1, 9), other), isTrue);
    });

    Glados<List<_Op>>(
      any.listWithLengthInRange(0, 120, any.tabOp),
      kansoExploreConfig(),
    ).test('the tabs stay consistent through any run of changes', (ops) {
      var tabs = const OpenTabs();
      for (final op in ops) {
        final before = tabs;
        tabs = _apply(tabs, op, _all);
        if (tabs.isNotEmpty) {
          expect(tabs.active, inInclusiveRange(0, tabs.length - 1));
          expect(tabs.activeId, isNotNull);
          expect(
            tabs.activeTab?.history.current,
            tabs.activeId,
            reason: 'a tab shows the note its history is at',
          );
        } else {
          expect(tabs.active, 0);
        }
        expect(_ids(tabs).toSet().length, tabs.length, reason: 'a note twice');
        expect(tabs.closed.length, lessThanOrEqualTo(mostClosed));
        if (before.isNotEmpty) {
          expect(
            tabs.isNotEmpty || op.$1 == 7,
            isTrue,
            reason: 'only a note gone from the library empties the row',
          );
        }
      }
    });

    Glados<List<_Op>>(
      any.listWithLengthInRange(0, 80, any.tabOp),
      kansoExploreConfig(),
    ).test('close never empties the row', (ops) {
      var tabs = const OpenTabs();
      for (final op in ops) {
        tabs = _apply(tabs, op, _all);
      }
      for (var index = 0; index < 9; index++) {
        final had = tabs.isNotEmpty;
        tabs = tabs.close(index).closeOthers(index).closeRight(index);
        expect(tabs.isNotEmpty, had);
      }
    });

    Glados3<int, int, int>(
      any.intInRange(2, 8),
      any.intInRange(0, 8),
      any.intInRange(0, 8),
      kansoExploreConfig(),
    ).test('a closed tab comes back to the clamped place it left', (
      count,
      at,
      active,
    ) {
      final row = _row(<String>[
        for (var n = 0; n < count; n++) '$n',
      ], active: active % count);
      final index = at % count;
      final closed = row.close(index);
      expect(identical(closed, row), isFalse);
      final back = closed.reopenClosed(_all).tabs;
      expect(_ids(back), _ids(row));
      expect(back.active, index);
      // Closing more first leaves the place to be clamped to the row.
      final shorter = closed.closeRight(0);
      final clamped = shorter.reopenClosed(_all);
      expect(clamped.reopened?.id, shorter.closed.last.meta.id);
      expect(
        clamped.tabs.active,
        shorter.closed.last.at.clamp(0, shorter.length),
      );
      expect(clamped.tabs.active, lessThan(clamped.tabs.length));
    });

    Glados2<int, int>(
      any.intInRange(1, 10),
      any.intInRange(0, 40),
      kansoExploreConfig(),
    ).test('next and previous wrap round the row', (count, steps) {
      final row = _row(<String>[for (var n = 0; n < count; n++) '$n']);
      var forward = row;
      var backward = row;
      for (var step = 0; step < steps; step++) {
        forward = forward.next();
        backward = backward.previous();
        expect(forward.active, inInclusiveRange(0, count - 1));
        expect(backward.active, inInclusiveRange(0, count - 1));
      }
      if (count > 1) {
        expect(forward.active, (row.active + steps) % count);
        expect(backward.active, ((row.active - steps) % count + count) % count);
      }
      expect(forward.next().previous().active, forward.active);
    });

    Glados3<int, int, int>(
      any.intInRange(2, 8),
      any.intInRange(0, 8),
      any.intInRange(0, 8),
      kansoExploreConfig(),
    ).test('a reorder keeps the showing note showing', (count, from, to) {
      final row = _row(<String>[
        for (var n = 0; n < count; n++) '$n',
      ], active: (from + to) % count);
      final showing = row.activeId;
      final expected = _ids(row);
      final source = from % count;
      final target = to % count;
      expected.insert(target, expected.removeAt(source));
      final moved = row.reorder(source, target);
      expect(_ids(moved), expected);
      expect(moved.activeId, showing);
    });
  });
}
