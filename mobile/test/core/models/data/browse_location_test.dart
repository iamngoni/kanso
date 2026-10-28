// Kanso — browse_location_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/models/data/search_scope_kind.dart';

import '../../../helpers/property.dart';

BrowseLocation _parsed(String address) {
  final location = BrowseLocation.parse(Uri.parse(address));
  expect(location, isNotNull, reason: address);
  return location!;
}

void main() {
  group('BrowseLocation', () {
    test('writes the address of each place', () {
      expect(const BrowseLocation.place(ListPlace.all()).location, '/notes');
      expect(
        const BrowseLocation.place(ListPlace.pinned()).location,
        '/pinned',
      );
      expect(
        const BrowseLocation.place(ListPlace.recent()).location,
        '/recent',
      );
      expect(const BrowseLocation.place(ListPlace.trash()).location, '/trash');
      expect(
        const BrowseLocation.place(ListPlace.notebook(id: 'b1')).location,
        '/notebooks/b1',
      );
      expect(
        const BrowseLocation.place(ListPlace.tag(id: 't1')).location,
        '/tags/t1',
      );
    });

    test('writes a note under its place, with its id encoded', () {
      expect(
        const BrowseLocation.place(
          ListPlace.notebook(id: 'b 1'),
          noteId: 'n:1/x',
        ).location,
        '/notebooks/b%201/note/n%3A1%2Fx',
      );
    });

    test('writes the tasks, and a note open in them', () {
      expect(const BrowseLocation.tasks().location, '/tasks');
      expect(
        const BrowseLocation.tasks(noteId: 'n:1').location,
        '/tasks/note/n%3A1',
      );
      expect(_parsed('/tasks'), const BrowseLocation.tasks());
      expect(
        _parsed('/tasks/note/n%3A1'),
        const BrowseLocation.tasks(noteId: 'n:1'),
      );
      expect(BrowseLocation.parse(Uri.parse('/tasks/other')), isNull);
      expect(
        const BrowseLocation.tasks(noteId: 'n').withoutNote,
        const BrowseLocation.tasks(),
      );
    });

    test('writes a search with only what differs from the default', () {
      expect(const BrowseLocation.search().location, '/search');
      expect(
        const BrowseLocation.search(query: 'tea time').location,
        '/search?q=tea+time',
      );
      expect(
        const BrowseLocation.search(
          query: 'a',
          scope: SearchScopeKind.notebook,
          notebookId: 'b1',
          noteId: 'n1',
        ).location,
        '/search/note/n1?q=a&scope=notebook&in=b1',
      );
    });

    test('adds and removes the open note', () {
      const list = BrowseLocation.place(ListPlace.pinned());
      final open = list.withNote('n1');
      expect(open.location, '/pinned/note/n1');
      expect(open.noteId, 'n1');
      expect(open.withoutNote, list);
      const search = BrowseLocation.search(query: 'x');
      expect(search.withNote('n2').withoutNote, search);
    });

    test('reads an address it does not know as nothing', () {
      for (final address in <String>[
        '/',
        '/settings',
        '/notes/other',
        '/notes/note',
        '/notes/note/a/b',
        '/notebooks',
        '/tags',
        '/search/other',
        '/design',
      ]) {
        expect(
          BrowseLocation.parse(Uri.parse(address)),
          isNull,
          reason: address,
        );
      }
    });

    test('reads a search with a scope it does not know as the default', () {
      expect(
        _parsed('/search?q=a&scope=weird'),
        const BrowseLocation.search(query: 'a'),
      );
      expect(_parsed('/search?in='), const BrowseLocation.search());
    });

    Glados<BrowseLocation>(any.browseLocation, kansoExploreConfig()).test(
      'an address reads back as the location it was written from',
      (location) {
        expect(_parsed(location.location), location);
      },
    );

    Glados<BrowseLocation>(any.browseLocation, kansoExploreConfig()).test(
      'opening a note and closing it again changes nothing',
      (location) {
        expect(location.withNote('n:1').withoutNote, location.withoutNote);
        expect(_parsed(location.withNote('n:1').location).noteId, 'n:1');
      },
    );
  });
}
