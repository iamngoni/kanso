// Kanso — deep_links_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/router/deep_links.dart';

import '../../helpers/property.dart';

void main() {
  group('redirectLocation', () {
    test('opens a note in All Notes', () {
      expect(
        redirectLocation(Uri.parse('kanso://open/note/n1')),
        '/notes/note/n1',
      );
      expect(redirectLocation(Uri.parse('/note/n1')), '/notes/note/n1');
    });

    test('opens a notebook', () {
      expect(
        redirectLocation(Uri.parse('kanso://open/notebook/b1')),
        '/notebooks/b1',
      );
      expect(redirectLocation(Uri.parse('/notebook/b1')), '/notebooks/b1');
    });

    test('opens a search, keeping what it says', () {
      expect(
        redirectLocation(
          Uri.parse('kanso://open/search?q=tea+time&scope=tags'),
        ),
        '/search?q=tea+time&scope=tags',
      );
    });

    test("opens today's note by the daily address", () {
      expect(redirectLocation(Uri.parse('kanso://open/daily')), '/daily');
      expect(redirectLocation(Uri.parse('/daily')), isNull);
    });

    test('opens any other address of the app', () {
      expect(
        redirectLocation(Uri.parse('kanso://open/settings/theme')),
        '/settings/theme',
      );
      expect(
        redirectLocation(Uri.parse('kanso://open/notebooks/b1/note/n1')),
        '/notebooks/b1/note/n1',
      );
    });

    test('sends the front door to All Notes', () {
      expect(redirectLocation(Uri.parse('/')), '/notes');
      expect(redirectLocation(Uri.parse('kanso://open')), '/notes');
      expect(redirectLocation(Uri.parse('kanso://open/')), '/notes');
    });

    test('sends a link that is not for opening, or not ours, to All Notes', () {
      expect(redirectLocation(Uri.parse('kanso://other/note/n1')), '/notes');
      expect(
        redirectLocation(Uri.parse('https://example.com/note/n1')),
        '/notes',
      );
    });

    test('leaves the addresses of the app as they are', () {
      for (final address in <String>[
        '/notes',
        '/notes/note/n1',
        '/notebooks/b1',
        '/search?q=a',
        '/settings',
        '/design',
      ]) {
        expect(redirectLocation(Uri.parse(address)), isNull, reason: address);
      }
    });

    Glados<BrowseLocation>(any.browseLocation, kansoExploreConfig()).test(
      'a link to a location lands on that location',
      (location) {
        // A place or a search written as a link is the same address.
        final link = Uri.parse('kanso://open${location.location}');
        final landed = redirectLocation(link) ?? link.toString();
        expect(Uri.parse(landed).path, Uri.parse(location.location).path);
        expect(BrowseLocation.parse(Uri.parse(landed)), location);
      },
    );

    Glados<String>(any.entityId, kansoExploreConfig()).test(
      'a link to a note lands on the note in All Notes whatever its id',
      (id) {
        final link = Uri.parse('kanso://open/note/${Uri.encodeComponent(id)}');
        expect(
          BrowseLocation.parse(Uri.parse(redirectLocation(link)!)),
          BrowseLocation.place(const ListPlace.all(), noteId: id),
        );
      },
    );

    Glados<String>(any.entityId, kansoExploreConfig()).test(
      'a link to a notebook lands on the notebook whatever its id',
      (id) {
        final link = Uri.parse(
          'kanso://open/notebook/${Uri.encodeComponent(id)}',
        );
        expect(
          BrowseLocation.parse(Uri.parse(redirectLocation(link)!)),
          BrowseLocation.place(ListPlace.notebook(id: id)),
        );
      },
    );
  });
}
