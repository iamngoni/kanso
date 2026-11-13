// Kanso — deep_links_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        CombinableAny,
        ExploreConfig,
        Generator,
        GeneratorUtils,
        Glados,
        ListAnys,
        any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/models/data/new_link_kind.dart';
import 'package:kanso/core/router/deep_links.dart';

import '../../helpers/property.dart';

/// The seed of the property tests of the `task=1` link (#390 slice 19).
ExploreConfig _taskLinkConfig() => ExploreConfig(random: Random(20261017));

/// One parameter of a `/new` address: the ones it reads, and others.
typedef _Parameter = ({String key, String value});

extension on Any {
  Generator<_Parameter> get _parameter => combine2(
    choose(<String>['task', 'voice', 'q', 'from', 'Task', 'tasks']),
    choose(<String>['1', '0', '', 'true', 'yes', '11', ' 1']),
    (String key, String value) => (key: key, value: value),
  );

  /// The parameters of a `/new` address, each key once.
  Generator<Map<String, String>> get newQuery =>
      listWithLengthInRange<_Parameter>(
        0,
        6,
        _parameter,
      ).map(
        (List<_Parameter> parameters) => <String, String>{
          for (final parameter in parameters) parameter.key: parameter.value,
        },
      );
}

void main() {
  group('newLinkKind', () {
    test('reads task=1 and voice=1, and is a note otherwise', () {
      expect(newLinkKind(const <String, String>{}), NewLinkKind.note);
      expect(newLinkKind(const {'task': '1'}), NewLinkKind.task);
      expect(newLinkKind(const {'voice': '1'}), NewLinkKind.voice);
      expect(newLinkKind(const {'task': '1', 'voice': '1'}), NewLinkKind.task);
      expect(newLinkKind(const {'task': '0', 'voice': '1'}), NewLinkKind.voice);
      expect(newLinkKind(const {'task': 'true'}), NewLinkKind.note);
    });

    Glados<Map<String, String>>(any.newQuery, _taskLinkConfig()).test(
      'task=1 wins, then voice=1, and anything else is a note',
      (query) {
        final expected = query['task'] == '1'
            ? NewLinkKind.task
            : query['voice'] == '1'
            ? NewLinkKind.voice
            : NewLinkKind.note;
        expect(newLinkKind(query), expected);
      },
    );

    Glados<Map<String, String>>(any.newQuery, _taskLinkConfig()).test(
      'a kanso://open/new link keeps what it asks for on its way in',
      (query) {
        final link = Uri(
          scheme: 'kanso',
          host: 'open',
          path: '/new',
          queryParameters: query.isEmpty ? null : query,
        );
        final landed = Uri.parse(redirectLocation(link)!);
        expect(landed.path, '/new');
        expect(newLinkKind(landed.queryParameters), newLinkKind(query));
        expect(opensTaskCapture(landed), query['task'] == '1');
      },
    );
  });

  group('opensTaskCapture', () {
    test('is /new?task=1 and nothing else', () {
      expect(opensTaskCapture(Uri.parse('/new?task=1')), isTrue);
      expect(opensTaskCapture(Uri.parse('/new/?task=1')), isTrue);
      expect(opensTaskCapture(Uri.parse('/new?voice=1&task=1')), isTrue);
      expect(opensTaskCapture(Uri.parse('/new')), isFalse);
      expect(opensTaskCapture(Uri.parse('/new?voice=1')), isFalse);
      expect(opensTaskCapture(Uri.parse('/new?task=0')), isFalse);
      expect(opensTaskCapture(Uri.parse('/notes?task=1')), isFalse);
      expect(opensTaskCapture(Uri.parse('/new/note?task=1')), isFalse);
    });
  });

  group('locationUnderSheet', () {
    test('stays where the app is, or All Notes', () {
      expect(locationUnderSheet(null), '/notes');
      expect(locationUnderSheet(Uri.parse('')), '/notes');
      expect(locationUnderSheet(Uri.parse('/')), '/notes');
      expect(locationUnderSheet(Uri.parse('/new')), '/notes');
      expect(locationUnderSheet(Uri.parse('/new?voice=1')), '/notes');
      expect(locationUnderSheet(Uri.parse('/daily')), '/notes');
      expect(
        locationUnderSheet(Uri.parse('/notebooks/b1/note/n1')),
        '/notebooks/b1/note/n1',
      );
      expect(locationUnderSheet(Uri.parse('/search?q=a')), '/search?q=a');
      expect(locationUnderSheet(Uri.parse('/settings')), '/settings');
    });

    Glados<BrowseLocation>(any.browseLocation, _taskLinkConfig()).test(
      'a sheet opens over any place or search without moving it',
      (location) {
        final at = Uri.parse(location.location);
        expect(locationUnderSheet(at), location.location);
        expect(
          BrowseLocation.parse(Uri.parse(locationUnderSheet(at))),
          location,
        );
      },
    );
  });

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
