// Kanso — discovery_suite.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/discovery.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// Discovery, through the bridge: every call of `KansoDiscovery` and
/// `KansoLibrary.discovery`, and the two plain functions beside them.
///
/// The embedding model is never downloaded here, so the tests never turn
/// related notes on: they cover what the shell sees with them off (which is
/// how a library starts), what the model costs before it is downloaded, the
/// graph, merging and dismissing, and the queue of images to read.
void discoverySuite(OpenLibrary open) {
  group('KansoDiscovery', () {
    test('plain words of three letters are what a search by meaning takes', () {
      expect(meaningMinChars(), 3);
      expect(meaningQuery(query: 'orchard fruit'), 'orchard fruit');
      expect(meaningQuery(query: '  fruit trees  '), 'fruit trees');
      // Two letters in all say nothing; a qualifier makes it a search,
      // not a query.
      expect(meaningQuery(query: 'ab'), isNull);
      expect(meaningQuery(query: 'a b'), isNull);
      expect(meaningQuery(query: 'tag:fruit'), isNull);
      expect(meaningQuery(query: '-fruit'), isNull);
      expect(meaningQuery(query: '"fruit"'), isNull);
      expect(meaningQuery(query: '   '), isNull);
    });

    test('related notes start off, find nothing, and turn off again', () async {
      final library = await open();
      final discovery = library.discovery();
      final orchard = await makeNote(
        library,
        'Orchard',
        'fruit trees in winter',
      );
      await makeNote(library, 'Pruning', 'fruit trees pruned in winter');

      expect(await discovery.relatedEnabled(), isFalse);
      expect((await discovery.relatedState()).status, isA<RelatedStatus_Off>());
      // Nothing is found while it is off.
      expect(await discovery.relatedNotes(id: orchard, limit: 5), isEmpty);
      expect(
        await discovery.searchByMeaning(query: 'orchard fruit', k: 5),
        isA<MeaningResult_Off>(),
      );
      final meaning = await discovery.noteMeaning(id: orchard);
      expect(meaning.tags, isEmpty);
      expect(meaning.twin, isNull);
      // A query with qualifiers isn't searched by meaning either; it finds
      // nothing rather than asking for a model.
      expect(
        await discovery.searchByMeaning(query: 'tag:fruit', k: 5),
        isA<MeaningResult_Off>(),
      );

      // Turning off what was already off keeps it off, and leaves the model
      // where it is.
      await discovery.setRelatedNotes(on_: false);
      expect(await discovery.relatedEnabled(), isFalse);
      expect((await discovery.relatedState()).status, isA<RelatedStatus_Off>());
      final still = await discovery.noteMeaning(id: orchard);
      expect(still.tags, isEmpty);
      expect(still.twin, isNull);
    });

    test(
      'the status stream says where related notes stand, then changes',
      () async {
        final library = await open();
        final discovery = library.discovery();
        final states = <RelatedState>[];
        final subscription = discovery.relatedStream().listen(states.add);
        await Future<void>.delayed(const Duration(milliseconds: 200));
        expect(states, hasLength(1));
        expect(states.single.status, isA<RelatedStatus_Off>());

        // Turning it off is a change, so the stream says so again.
        await discovery.setRelatedNotes(on_: false);
        await Future<void>.delayed(const Duration(milliseconds: 200));
        expect(states, hasLength(2));
        expect(states.last.status, isA<RelatedStatus_Off>());
        // Cancelling waits for the library's next event: don't wait for it.
        addTearDown(() => unawaited(subscription.cancel()));
      },
    );

    test(
      'where the model lives, and that measuring it says it is missing',
      () async {
        final library = await open();
        final discovery = library.discovery();
        final dir = discovery.modelDir();
        expect(dir, isNotEmpty);
        expect(dir, endsWith('models/potion-base-8M'));

        // The model is downloaded by turning related notes on, which needs the
        // network: here it isn't there, and measuring says so rather than
        // pretending to measure a model that isn't on the phone.
        await expectLater(
          discovery.measure(),
          throwsA(
            isA<KansoError_Invalid>().having(
              (error) => error.message,
              'message',
              contains("The model isn't downloaded yet"),
            ),
          ),
        );
      },
    );

    test(
      'an image nothing shows is not waiting, and words for one that is '
      'not there are refused',
      () async {
        final library = await open();
        final discovery = library.discovery();
        expect(await discovery.ocrWaiting(), 0);
        expect(await discovery.ocrNext(), isNull);
        // Nothing is known of a file that isn't in the library.
        expect(
          await discovery.ocrKeep(
            id: 'file:missing',
            md5: 'd41d8cd98f00b204e9800998ecf8427e',
            text: 'words',
            engine: 'vision',
          ),
          isFalse,
        );
      },
    );

    test('an image a note shows waits to be read once', () async {
      final library = await open();
      final discovery = library.discovery();
      // A one-pixel PNG, captured the way a shared image is: the note shows
      // it, so its words are waiting to be read.
      final png = base64Decode(
        'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQ'
        'DwAEhQGAhKmMIQAAAABJRU5ErkJggg==',
      );
      final captured = await library.capture(
        share: Share(
          files: <ShareFile>[
            ShareFile(name: 'scan.png', contentType: 'image/png', bytes: png),
          ],
        ),
      );
      expect(await discovery.ocrWaiting(), 1);
      final next = await discovery.ocrNext();
      expect(next, isNotNull);
      expect(next!.bytes, png);
      expect(next.md5, isNotEmpty);

      // Keeping the words marks it read: keeping them again is refused
      // because the bytes it read are the ones already kept.
      expect(
        await discovery.ocrKeep(
          id: next.id,
          md5: next.md5,
          text: '  HELLO FROM THE IMAGE  ',
          engine: 'vision',
        ),
        isTrue,
      );
      expect(await discovery.ocrWaiting(), 0);
      expect(await discovery.ocrNext(), isNull);
      // The note keeps showing the image it captured.
      expect((await library.note(id: captured.noteId)).id, captured.noteId);
    });

    test(
      'two notes are different after all once their pair is dismissed',
      () async {
        final library = await open();
        final discovery = library.discovery();
        final first = await makeNote(library, 'First', 'orchard fruit trees');
        final second = await makeNote(library, 'Second', 'orchard fruit trees');
        // Nothing was a twin while related notes are off; dismissing the pair
        // anyway is harmless, and neither note gains a twin.
        await discovery.dismissTwin(id: first, otherId: second);
        expect((await discovery.noteMeaning(id: first)).twin, isNull);
        expect((await discovery.noteMeaning(id: second)).twin, isNull);
      },
    );

    test('merging keeps the note and puts the other in the trash', () async {
      final library = await open();
      final discovery = library.discovery();
      final keep = await makeNote(library, 'Keep', 'kept words');
      final other = await makeNote(library, 'Other', 'other words');
      final sealed = await makeNote(library, 'Sealed', 'sealed words');
      await library.locks().lockNote(id: sealed, password: lockPassword);

      // A locked note can't be merged, from either side.
      await expectLater(
        discovery.mergeNotes(keepId: keep, otherId: sealed),
        throwsA(isA<KansoError_Locked>()),
      );
      await expectLater(
        discovery.mergeNotes(keepId: sealed, otherId: keep),
        throwsA(isA<KansoError_Locked>()),
      );
      await expectLater(
        discovery.mergeNotes(keepId: keep, otherId: keep),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        discovery.mergeNotes(keepId: 'note:missing', otherId: keep),
        throwsA(isA<KansoError_NotFound>()),
      );

      final merged = await discovery.mergeNotes(keepId: keep, otherId: other);
      expect(merged.id, keep);
      expect(merged.body, 'kept words\n\n## Other\n\nother words\n');
      // The other is in the trash now, and no longer among the notes.
      expect(
        (await library.list(query: trashedNotes)).rows.map((row) => row.id),
        contains(other),
      );
      expect(
        (await library.list(query: everyNote)).rows.map((row) => row.id),
        isNot(contains(other)),
      );
    });

    test(
      'the graph holds the notes and their links, laid out the same way twice',
      () async {
        final library = await open();
        final discovery = library.discovery();
        final hub = await makeNote(library, 'Hub', '[[Open]] [[Secret]]');
        final openNote = await makeNote(library, 'Open', '[[Hub]]');
        final secret = await makeNote(library, 'Secret', '[[Hub]]');
        const all = GraphRequest.library_(filter: GraphFilter.all());

        final graph = await discovery.graph(request: all);
        expect(
          graph.nodes.map((node) => node.id),
          containsAll(<String>[hub, openNote, secret]),
        );
        expect(graph.nodes, hasLength(3));
        expect(graph.linkCount, greaterThanOrEqualTo(2));
        for (final node in graph.nodes) {
          expect(node.title, isNotEmpty);
          expect(node.notebookId, isNotEmpty);
          expect(node.x.isFinite && node.y.isFinite, isTrue);
          expect(node.radius, greaterThan(0));
          expect(node.isCenter, isFalse);
          // Opaque, in both themes, and the themes differ.
          expect(node.colorLight >> 24, 0xFF);
          expect(node.colorDark >> 24, 0xFF);
          expect(node.colorLight, isNot(node.colorDark));
        }
        for (final edge in graph.edges) {
          expect(edge.from, lessThan(graph.nodes.length));
          expect(edge.to, lessThan(graph.nodes.length));
        }
        // Laid out the same way twice, to the pixel.
        final again = await discovery.graph(request: all);
        expect(again.nodes, graph.nodes);
        expect(again.edges, graph.edges);
        expect(again.linkCount, graph.linkCount);

        // The neighbourhood of one note has exactly one centre.
        final local = await discovery.graph(
          request: GraphRequest.local(center: hub, hops: 9),
        );
        expect(
          local.nodes.where((node) => node.isCenter).map((node) => node.id),
          <String>[hub],
        );

        // A notebook narrows the graph; one without notes has none.
        final book = graph.nodes.first.notebookId;
        final narrowed = await discovery.graph(
          request: GraphRequest.library_(
            filter: GraphFilter.book(id: book),
          ),
        );
        expect(narrowed.nodes, hasLength(3));
        final none = await discovery.graph(
          request: const GraphRequest.library_(
            filter: GraphFilter.tag(id: 'tag:none'),
          ),
        );
        expect(none.nodes, isEmpty);
        expect(none.edges, isEmpty);
        expect(none.linkCount, 0);
      },
    );

    test(
      'the graph leaves out a locked note and its links until the lock '
      'is opened',
      () async {
        final library = await open();
        final discovery = library.discovery();
        final locks = library.locks();
        final hub = await makeNote(library, 'Hub', '[[Open]] [[Secret]]');
        final openNote = await makeNote(library, 'Open', '[[Hub]]');
        final secret = await makeNote(library, 'Secret', '[[Hub]]');
        const all = GraphRequest.library_(filter: GraphFilter.all());

        // Locking sets the password and leaves the lock open: everything shows.
        await locks.lockNote(id: secret, password: lockPassword);
        expect(
          (await discovery.graph(request: all)).nodes.map((node) => node.id),
          containsAll(<String>[hub, openNote, secret]),
        );

        // Closed: the locked note and its links are gone.
        await locks.lock();
        final closed = await discovery.graph(request: all);
        final ids = closed.nodes.map((node) => node.id).toList();
        expect(ids, isNot(contains(secret)));
        expect(ids, containsAll(<String>[hub, openNote]));
        expect(closed.linkCount, 1);
        // A hidden note has no neighbourhood of its own.
        final hidden = await discovery.graph(
          request: GraphRequest.local(center: secret, hops: 1),
        );
        expect(hidden.nodes, isEmpty);

        // Opened again: it is back.
        expect(await locks.unlock(password: lockPassword), isTrue);
        expect(
          (await discovery.graph(request: all)).nodes.map((node) => node.id),
          contains(secret),
        );
      },
    );
  });
}
