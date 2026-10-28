// Kanso — property.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math';
import 'dart:ui';

import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        BoolAny,
        CombinableAny,
        ExploreConfig,
        Generator,
        GeneratorUtils,
        IntAnys,
        ListAnys,
        StringAnys;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_fonts.dart';
import 'package:kanso/core/models/data/kanso_text_size.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/search_scope_kind.dart';
import 'package:kanso/core/models/data/theme_selection.dart';

/// Property tests use this configuration so a failure in CI reproduces
/// locally: the random source has a fixed seed.
ExploreConfig kansoExploreConfig() {
  return ExploreConfig(random: Random(20261025));
}

extension KansoAny on Any {
  /// Any opaque sRGB colour.
  Generator<Color> get opaqueColor => combine3(
    intInRange(0, 256),
    intInRange(0, 256),
    intInRange(0, 256),
    (int r, int g, int b) => Color.fromARGB(255, r, g, b),
  );
}

extension KansoIconsAny on Any {
  /// Any of the icons Kanso draws.
  Generator<KansoIcons> get kansoIcon => choose(KansoIcons.values);
}

extension KansoFontsAny on Any {
  /// Any font family, the device's own included.
  Generator<KansoFontFamily> get fontFamily => choose(KansoFontFamily.values);

  /// Any text size.
  Generator<KansoTextSize> get textSize => choose(KansoTextSize.values);

  /// Any combination of the three fonts.
  Generator<KansoFonts> get fonts => combine3(
    fontFamily,
    fontFamily,
    fontFamily,
    (KansoFontFamily editor, KansoFontFamily preview, KansoFontFamily code) =>
        KansoFonts(editorFont: editor, previewFont: preview, codeFont: code),
  );

  /// A text scale a device may set, from 0.5 to 3.5.
  Generator<double> get deviceTextScale =>
      intInRange(50, 351).map((percent) => percent / 100);
}

/// The characters Markdown and search care about, plus letters and space.
const List<String> _markdownAlphabet = <String>[
  'a',
  'B',
  'y',
  's',
  'n',
  'c',
  ' ',
  '*',
  '`',
  '[',
  ']',
  '(',
  ')',
];

extension KansoMarkdownAny on Any {
  /// Short strings made of the characters Markdown and search care about, so
  /// marks pair up, break and overlap often.
  Generator<String> get markdownish {
    return listWithLengthInRange<String>(
      0,
      14,
      choose(_markdownAlphabet),
    ).map((chars) => chars.join());
  }
}

extension KansoErrorAny on Any {
  /// Any error the Rust library can fail with, with any message.
  Generator<KansoError> get kansoError => combine2(
    intInRange(0, 9),
    nonEmptyLetterOrDigits,
    (int kind, String text) => switch (kind) {
      0 => const KansoError.notFound(),
      1 => const KansoError.locked(),
      2 => KansoError.conflict(currentRev: text),
      3 => KansoError.invalid(message: text),
      4 => KansoError.storage(message: text),
      5 => KansoError.network(message: text),
      6 => const KansoError.signedOut(),
      7 => KansoError.auth(message: text),
      _ => KansoError.internal(message: text),
    },
  );
}

/// Theme names: a few, so picks collide with the slots now and then.
const List<String> _themeIds = <String>[
  'kanso-light',
  'kanso-dark',
  'nord',
  'github-light',
  'dracula',
];

extension KansoThemeSelectionAny on Any {
  /// Any built-in theme's name, out of five.
  Generator<String> get themeId => choose(_themeIds);

  /// Any selection of themes.
  Generator<ThemeSelection> get themeSelection => combine4(
    this.bool,
    themeId,
    themeId,
    themeId,
    (bool follow, String ui, String light, String dark) => ThemeSelection(
      followSystem: follow,
      ui: ui,
      light: light,
      dark: dark,
    ),
  );

  /// Any preferences.
  Generator<Preferences> get preferences => combine2(
    appearanceAndLists,
    choose(<String>['lucide', 'phosphor', 'material-icon-theme', 'x']),
    (Preferences before, String iconTheme) =>
        before.copyWith(iconTheme: iconTheme),
  );

  /// Preferences with the icon theme left as the default.
  Generator<Preferences> get appearanceAndLists => combine6(
    themeSelection,
    fonts,
    textSize,
    choose(SortKey.values),
    this.bool,
    this.bool,
    (
      ThemeSelection themes,
      KansoFonts fonts,
      KansoTextSize size,
      SortKey sort,
      bool descending,
      bool seen,
    ) => Preferences(
      themeSelection: themes,
      fonts: fonts,
      textSize: size,
      sort: sort,
      descending: descending,
      backupNoticeSeen: seen,
    ),
  );
}

/// Characters an id or a search may hold that an address must escape: a
/// colon, a slash, a space, a percent sign, a plus, an ampersand, an equals
/// sign, a question mark, and non-ASCII letters.
const List<String> _idCharacters = <String>[
  'a',
  'Z',
  '7',
  ':',
  '/',
  ' ',
  '%',
  '+',
  '&',
  '=',
  '?',
  '#',
  '\u00e9',
  '\u6f22',
];

extension KansoLocationAny on Any {
  /// A note, notebook or tag id: never empty.
  Generator<String> get entityId => combine2(
    choose(_idCharacters),
    listWithLengthInRange<String>(0, 6, choose(_idCharacters)),
    (String first, List<String> rest) => first + rest.join(),
  );

  /// Any place of the library.
  Generator<ListPlace> get listPlace => combine3(
    intInRange(0, 6),
    entityId,
    entityId,
    (int kind, String a, String b) => switch (kind) {
      0 => const ListPlace.all(),
      1 => const ListPlace.pinned(),
      2 => const ListPlace.recent(),
      3 => const ListPlace.trash(),
      4 => ListPlace.notebook(id: a),
      _ => ListPlace.tag(id: b),
    },
  );

  /// Any place, search or the tasks, with or without a note open.
  Generator<BrowseLocation> get browseLocation => combine6(
    this.bool,
    listPlace,
    listWithLengthInRange<String>(0, 6, choose(_idCharacters)),
    choose(SearchScopeKind.values),
    entityId,
    this.bool,
    (
      bool search,
      ListPlace place,
      List<String> query,
      SearchScopeKind scope,
      String id,
      bool noteOpen,
    ) {
      final noteId = noteOpen ? id : null;
      return search
          ? BrowseLocation.search(
              query: query.join(),
              scope: scope,
              notebookId: scope == SearchScopeKind.notebook ? id : null,
              noteId: noteId,
            )
          : query.length == 5
          ? BrowseLocation.tasks(noteId: noteId)
          : BrowseLocation.place(place, noteId: noteId);
    },
  );
}

extension KansoHistoryAny on Any {
  /// An operation on a navigation history: a kind from 0 to 6 (0 to 3 open
  /// a note, 4 goes back, 5 forward, 6 trashes or restores one) and a note
  /// from 0 to 5.
  Generator<(int, int)> get historyOp => combine2(
    intInRange(0, 7),
    intInRange(0, 6),
    (int kind, int note) => (kind, note),
  );
}

extension KansoListAny on Any {
  /// The rows of a list in the order the library lists them: pinned first,
  /// then the most recently changed. Their times are a few months of 2026,
  /// so several share a day.
  Generator<List<NoteRow>> get listedRows {
    final row = combine3(
      this.bool,
      intInRange(1770000000000, 1790000000000),
      intInRange(1770000000000, 1790000000000),
      (bool pinned, int created, int updated) => NoteRow(
        id: '',
        rev: '',
        title: '',
        snippet: '',
        notebookId: '',
        status: 'none',
        tags: const <String>[],
        pinned: pinned,
        createdMs: created,
        updatedMs: updated,
        tasksDone: 0,
        tasksTotal: 0,
        locked: false,
      ),
    );
    return listWithLengthInRange<NoteRow>(0, 40, row).map((rows) {
      final numbered =
          <NoteRow>[
            for (var i = 0; i < rows.length; i++)
              NoteRow(
                id: 'n$i',
                rev: '',
                title: 'Note $i',
                snippet: '',
                notebookId: 'b',
                status: 'none',
                tags: const <String>[],
                pinned: rows[i].pinned,
                createdMs: rows[i].createdMs,
                updatedMs: rows[i].updatedMs,
                tasksDone: 0,
                tasksTotal: 0,
                locked: false,
              ),
          ]..sort((a, b) {
            if (a.pinned != b.pinned) {
              return a.pinned ? -1 : 1;
            }
            return b.updatedMs.compareTo(a.updatedMs);
          });
      return numbered;
    });
  }

  /// Any shape a list can have.
  Generator<ListShape> get listShape => combine4(
    this.bool,
    this.bool,
    this.bool,
    choose(SortKey.values),
    (bool byDate, bool created, bool groupPinned, SortKey sort) => ListShape(
      sort: sort,
      byDate: byDate,
      date: created ? DateField.created : DateField.updated,
      groupPinned: groupPinned,
    ),
  );

  /// A time zone's offset from UTC, in seconds: from -12 to +14 hours.
  Generator<int> get utcOffsetSeconds => intInRange(-12 * 3600, 14 * 3600 + 1);
}

/// One release of a swipe: how far the row was moved and how fast the
/// finger was going along that way (both in points, and points a second),
/// the row's width, and which of the sides have actions and commit, as the
/// bits of a number from 0 to 15.
typedef SwipeRelease = (int offset, int velocity, int width, int sides);

extension KansoSwipeAny on Any {
  /// Any release of a swipe, in either direction.
  Generator<SwipeRelease> get swipeRelease => combine4(
    intInRange(-600, 601),
    intInRange(-3000, 3001),
    intInRange(200, 1201),
    intInRange(0, 16),
    (int offset, int velocity, int width, int sides) =>
        (offset, velocity, width, sides),
  );
}
