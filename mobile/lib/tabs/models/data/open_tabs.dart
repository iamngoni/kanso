// Kanso — open_tabs.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/models/data/navigation_history.dart';
import 'closed_tab.dart';
import 'open_tab.dart';
import 'tab_meta.dart';

part 'open_tabs.freezed.dart';

/// How many closed tabs can be reopened, the newest first.
const int mostClosed = 25;

/// The notes open in the window as a row of tabs, the one showing, and the
/// ones closed that can be reopened: the desktop's `Tabs`, rule for rule.
/// Empty when no note is open at all; a tab is never blank.
///
/// It is a value: every change returns the tabs as they are afterwards, and
/// the very same object ([identical]) when there was nothing to change, which
/// is how the desktop's `bool` results read here.
@freezed
abstract class OpenTabs with _$OpenTabs {
  const factory OpenTabs({
    /// The tabs, left to right.
    @Default(<OpenTab>[]) List<OpenTab> open,

    /// The index of the tab showing; 0 while none is open.
    @Default(0) int active,

    /// The tabs closed, the newest last; at most [mostClosed].
    @Default(<ClosedTab>[]) List<ClosedTab> closed,
  }) = _OpenTabs;

  const OpenTabs._();

  /// The tabs as they were kept in the preferences: the first of any
  /// repeated note only, with [active] clamped to one that exists.
  factory OpenTabs.restore(Iterable<TabMeta> metas, int active) {
    final tabs = <OpenTab>[];
    for (final meta in metas) {
      if (tabs.any((tab) => tab.meta.id == meta.id)) {
        continue;
      }
      tabs.add(OpenTab.opened(meta));
    }
    return OpenTabs(
      open: tabs,
      active: tabs.isEmpty ? 0 : active.clamp(0, tabs.length - 1),
    );
  }

  int get length => open.length;

  bool get isEmpty => open.isEmpty;

  bool get isNotEmpty => open.isNotEmpty;

  /// The tab showing, when one is open.
  OpenTab? get activeTab => active < open.length ? open[active] : null;

  /// The note of the tab showing.
  String? get activeId => activeTab?.meta.id;

  /// The showing tab's own back and forward.
  NavigationHistory? get history => activeTab?.history;

  /// Whether a closed tab can be brought back (if its note is still there).
  bool get canReopen => closed.isNotEmpty;

  /// The tab at [index], if there is one.
  OpenTab? tabAt(int index) =>
      index >= 0 && index < open.length ? open[index] : null;

  /// Where note [id] is open, if it is.
  int? indexOf(String id) {
    final at = open.indexWhere((tab) => tab.meta.id == id);
    return at < 0 ? null : at;
  }

  List<String> get ids => <String>[for (final tab in open) tab.meta.id];

  List<TabMeta> get metas => <TabMeta>[for (final tab in open) tab.meta];

  /// Opens [meta] in the tab showing, or switches to it when it is open
  /// already.
  OpenTabs openNote(TabMeta meta) {
    final existing = indexOf(meta.id);
    if (existing != null) {
      return copyWith(active: existing);
    }
    if (open.isEmpty) {
      return copyWith(open: <OpenTab>[OpenTab.opened(meta)], active: 0);
    }
    // The tab keeps its back-and-forward trail and the new note is a visit
    // on it, as a browser keeps a tab's history.
    final tab = open[active];
    return _withTab(
      active,
      OpenTab(meta: meta, history: tab.history.visit(meta.id)),
    );
  }

  /// Opens [meta] in a new tab after the one showing, or switches to it when
  /// it is open already.
  OpenTabs openInNew(TabMeta meta) {
    final existing = indexOf(meta.id);
    if (existing != null) {
      return copyWith(active: existing);
    }
    final at = open.isEmpty ? 0 : active + 1;
    return copyWith(
      open: <OpenTab>[...open]..insert(at, OpenTab.opened(meta)),
      active: at,
    );
  }

  /// Makes the tab showing show [meta] without adding a visit: the history
  /// has already moved (going back or forward). The note may be in another
  /// tab too; a browser allows that, and navigation never leaves the tab it
  /// started in.
  OpenTabs setActiveMeta(TabMeta meta) {
    final tab = activeTab;
    if (tab == null) {
      return this;
    }
    return _withTab(active, tab.copyWith(meta: meta));
  }

  /// Steps the showing tab's history back ([back]) or forward, to the
  /// nearest visit whose note [live] says is still there; the note landed on
  /// is null when there is none. Only this tab's history moves: notes open
  /// in other tabs are not switched to. The caller then shows the landed
  /// note with [setActiveMeta].
  ({OpenTabs tabs, String? landed}) stepActive({
    required bool back,
    required bool Function(String id) live,
  }) {
    final tab = activeTab;
    if (tab == null) {
      return (tabs: this, landed: null);
    }
    final stepped = back
        ? tab.history.goBack(live)
        : tab.history.goForward(live);
    return (
      tabs: _withTab(active, tab.copyWith(history: stepped.history)),
      landed: stepped.landed,
    );
  }

  /// Shows the tab at [index], if there is one.
  OpenTabs activate(int index) {
    if (index < 0 || index >= open.length) {
      return this;
    }
    return index == active ? this : copyWith(active: index);
  }

  /// The note's title or lock changed elsewhere.
  OpenTabs updateNote(
    String id, {
    required String title,
    required bool locked,
  }) {
    final at = indexOf(id);
    if (at == null) {
      return this;
    }
    final meta = open[at].meta.copyWith(title: title, locked: locked);
    return _withTab(at, open[at].copyWith(meta: meta));
  }

  /// The note was trashed or deleted elsewhere: its tab goes, and it is not
  /// kept to reopen.
  OpenTabs removeNote(String id) {
    final at = indexOf(id);
    if (at == null) {
      return this;
    }
    final rest = <OpenTab>[...open]..removeAt(at);
    return copyWith(open: rest, active: _activeAfterRemoving(at, rest.length));
  }

  /// Closes the tab at [index], unless it is the only one: someone who wants
  /// no notes open closes them by trashing, and there is no empty tab.
  OpenTabs close(int index) {
    if (index < 0 || index >= open.length || open.length < 2) {
      return this;
    }
    final rest = <OpenTab>[...open];
    final tab = rest.removeAt(index);
    return copyWith(
      open: rest,
      active: _activeAfterRemoving(index, rest.length),
      closed: _remembered(<ClosedTab>[ClosedTab(meta: tab.meta, at: index)]),
    );
  }

  /// Closes every tab but the one at [index].
  OpenTabs closeOthers(int index) {
    if (index < 0 || index >= open.length || open.length < 2) {
      return this;
    }
    return copyWith(
      open: <OpenTab>[open[index]],
      active: 0,
      closed: _remembered(<ClosedTab>[
        for (var at = 0; at < open.length; at++)
          if (at != index) ClosedTab(meta: open[at].meta, at: at),
      ]),
    );
  }

  /// Closes every tab to the right of the one at [index].
  OpenTabs closeRight(int index) {
    if (index < 0 || index + 1 >= open.length) {
      return this;
    }
    return copyWith(
      open: open.sublist(0, index + 1),
      active: active > index ? index : active,
      closed: _remembered(<ClosedTab>[
        for (var at = index + 1; at < open.length; at++)
          ClosedTab(meta: open[at].meta, at: at),
      ]),
    );
  }

  /// Moves the tab at [from] to [to], both indices in the row; the tab
  /// showing stays showing.
  OpenTabs reorder(int from, int to) {
    if (from < 0 ||
        to < 0 ||
        from >= open.length ||
        to >= open.length ||
        from == to) {
      return this;
    }
    final showing = activeId;
    final moved = <OpenTab>[...open];
    moved.insert(to, moved.removeAt(from));
    final at = showing == null
        ? null
        : moved.indexWhere((tab) => tab.meta.id == showing);
    return copyWith(
      open: moved,
      active: at == null || at < 0 ? active.clamp(0, moved.length - 1) : at,
    );
  }

  /// Brings back the newest closed tab whose note [live] says still exists,
  /// at the place it was closed from, and shows it. Closed tabs for notes
  /// gone since are dropped. The `reopened` tab is null when there was none.
  ({OpenTabs tabs, TabMeta? reopened}) reopenClosed(
    bool Function(String id) live,
  ) {
    final rest = <ClosedTab>[...closed];
    while (rest.isNotEmpty) {
      final last = rest.removeLast();
      if (!live(last.meta.id) || indexOf(last.meta.id) != null) {
        continue;
      }
      final at = last.at.clamp(0, open.length);
      return (
        tabs: copyWith(
          open: <OpenTab>[...open]..insert(at, OpenTab.opened(last.meta)),
          active: at,
          closed: rest,
        ),
        reopened: last.meta,
      );
    }
    return (tabs: copyWith(closed: rest), reopened: null);
  }

  /// The next tab, wrapping past the last to the first.
  OpenTabs next() =>
      open.length < 2 ? this : copyWith(active: (active + 1) % open.length);

  /// The tab before, wrapping from the first to the last.
  OpenTabs previous() => open.length < 2
      ? this
      : copyWith(active: (active + open.length - 1) % open.length);

  /// Shows tab [number] counted from 1; 9 is the last, as in a browser.
  OpenTabs goTo(int number) {
    if (open.isEmpty || number < 1) {
      return this;
    }
    final index = number >= 9 ? open.length - 1 : number - 1;
    if (index >= open.length) {
      return this;
    }
    return activate(index);
  }

  OpenTabs _withTab(int at, OpenTab tab) {
    final tabs = <OpenTab>[...open];
    tabs[at] = tab;
    return copyWith(open: tabs);
  }

  /// The index of the tab showing after the one at [removed] went and
  /// [remaining] are left.
  int _activeAfterRemoving(int removed, int remaining) {
    if (remaining == 0) {
      return 0;
    }
    if (removed < active) {
      return active - 1;
    }
    return active.clamp(0, remaining - 1);
  }

  /// The closed tabs with [more] on the end, the oldest forgotten past
  /// [mostClosed].
  List<ClosedTab> _remembered(List<ClosedTab> more) {
    final all = <ClosedTab>[...closed, ...more];
    return all.length > mostClosed ? all.sublist(all.length - mostClosed) : all;
  }
}
