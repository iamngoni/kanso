// Kanso — sharing_property.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        BoolAny,
        CombinableAny,
        Generator,
        GeneratorUtils,
        IntAnys,
        ListAnys;
import 'package:kanso/core/bridge/api/types.dart';

import 'gallery_fixtures.dart';
import 'sharing_fixtures.dart';

const List<String> _names = <String>[
  'Nord',
  'Dusk',
  'Paper',
  'Moss',
  'nordic dawn',
  'Lines',
  'DUSK light',
];

const List<String> _authors = <String>['Ana', 'Bo', 'Kanso', 'cy'];

const List<String> _descriptions = <String>[
  'A calm theme.',
  'High contrast for bright rooms.',
  'Soft colours, cool greys',
  '',
];

/// What differs between the themes of a generated gallery.
typedef GallerySpec = (String, String, String, int, int, bool);

extension GalleryAny on Any {
  /// Up to eleven gallery themes whose ids are their positions, so no two are
  /// alike; the rest of each is drawn from small pools so that searches find
  /// several and ties in the orders are common.
  Generator<List<GalleryItem>> get galleryItems {
    final spec = combine6(
      choose(_names),
      choose(_authors),
      choose(_descriptions),
      intInRange(0, 4),
      intInRange(0, 4),
      this.bool,
      (
        String name,
        String author,
        String description,
        int installs,
        int updated,
        bool official,
      ) => (name, author, description, installs, updated, official),
    );
    return listWithLengthInRange<GallerySpec>(0, 12, spec).map((specs) {
      return <GalleryItem>[
        for (var position = 0; position < specs.length; position++)
          galleryItem(
            'theme-$position',
            name: specs[position].$1,
            author: specs[position].$2,
            description: specs[position].$3,
            installs: specs[position].$4,
            updatedMs: specs[position].$5 * 1000,
            official: specs[position].$6,
            installed: position.isEven,
            kind: position % 5 == 4
                ? GalleryKind.iconTheme
                : GalleryKind.colorTheme,
            disabled: position % 7 == 6,
          ),
      ];
    });
  }

  /// A search: nothing, words that match often, and words that never do.
  Generator<String> get gallerySearch => choose(<String>[
    '',
    ' ',
    'a',
    'NORD',
    'dusk',
    'by ana',
    'calm theme',
    'contrast  bright',
    'zzz',
    'soft cool',
  ]);

  /// Any person in a shared note, with any combination of the flags.
  Generator<MemberRow> get member => combine5(
    this.bool,
    this.bool,
    this.bool,
    this.bool,
    this.bool,
    (bool me, bool keyChanged, bool invited, bool left, bool verified) =>
        memberRow(
          'ana',
          me: me,
          keyChanged: keyChanged,
          invited: invited,
          left: left,
          verified: verified,
        ),
  );

  /// Any share role.
  Generator<ShareRole> get shareRole => choose(ShareRole.values);

  /// Any status of a shared note's page: any role, live or not, ended or
  /// not, near its limit or not, with up to three problems.
  Generator<LiveStatus> get liveStatus => combine5(
    shareRole,
    this.bool,
    this.bool,
    this.bool,
    listWithLengthInRange<String>(
      0,
      4,
      choose(<String>['Couldn’t send.', 'Too big. ', 'Refused']),
    ),
    (
      ShareRole role,
      bool connected,
      bool ended,
      bool nearLimit,
      List<String> problems,
    ) => LiveStatus(
      role: role,
      connected: connected,
      ended: ended,
      nearLimit: nearLimit,
      problems: problems,
    ),
  );
}
