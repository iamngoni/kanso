// Kanso — kanso_print_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';
import 'dart:math';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso_print/kanso_print.dart';
import 'package:kanso_print/src/messages.g.dart';

const String _channel = 'dev.flutter.pigeon.kanso_print.PrintHostApi.htmlToPdf';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  group('paperFor', () {
    test('Letter for the Letter countries', () {
      for (final code in <String>[
        'US', 'CA', 'PH', 'MX', 'CL', 'CO', 'VE', 'GT', 'PR', //
      ]) {
        expect(paperFor(code), KansoPaper.letter, reason: code);
      }
    });

    test('A4 elsewhere and when unknown', () {
      for (final code in <String?>['GB', 'DE', 'ZW', 'ZA', 'JP', '', null]) {
        expect(paperFor(code), KansoPaper.a4, reason: '$code');
      }
    });

    test('ignores case and padding', () {
      expect(paperFor('us'), KansoPaper.letter);
      expect(paperFor(' ca '), KansoPaper.letter);
    });

    test('over every two-letter code: Letter exactly for the set', () {
      const a = 65;
      for (var i = 0; i < 26; i++) {
        for (var j = 0; j < 26; j++) {
          final code = String.fromCharCodes(<int>[a + i, a + j]);
          expect(
            paperFor(code),
            letterCountries.contains(code) ? KansoPaper.letter : KansoPaper.a4,
            reason: code,
          );
          expect(paperFor(code.toLowerCase()), paperFor(code));
        }
      }
    });

    test('random strings never throw and only yield a known paper', () {
      final random = Random(48);
      for (var n = 0; n < 2000; n++) {
        final code = String.fromCharCodes(
          List<int>.generate(random.nextInt(6), (_) => random.nextInt(0x250)),
        );
        expect(KansoPaper.values, contains(paperFor(code)));
      }
    });

    test('paper sizes are portrait points', () {
      for (final paper in KansoPaper.values) {
        expect(paper.heightPt, greaterThan(paper.widthPt));
      }
    });
  });

  group('withPrintStyles', () {
    test('adds the style inside head, leaving the doctype first', () {
      final html = File('test/fixture.html').readAsStringSync();
      final out = withPrintStyles(html);
      expect(out, startsWith('<!doctype html>'));
      expect(
        out.indexOf('print-color-adjust'),
        lessThan(out.indexOf('</head>')),
      );
      expect(
        out.indexOf('print-color-adjust'),
        greaterThan(out.indexOf('<head>')),
      );
    });

    test('appends when there is no head', () {
      final out = withPrintStyles('<p>hi</p>');
      expect(out, startsWith('<p>hi</p>'));
      expect(out, contains('print-color-adjust:exact'));
    });

    test('is found whatever the case of the tag', () {
      expect(
        withPrintStyles('<HEAD></HEAD><p/>').indexOf('<style>'),
        '<HEAD>'.length,
      );
    });
  });

  group('htmlToPdf', () {
    tearDown(() {
      messenger.setMockDecodedMessageHandler<Object?>(
        const BasicMessageChannel<Object?>(
          _channel,
          PrintHostApi.pigeonChannelCodec,
        ),
        null,
      );
    });

    void answer(Object? Function(List<Object?> args) reply) {
      messenger.setMockDecodedMessageHandler<Object?>(
        const BasicMessageChannel<Object?>(
          _channel,
          PrintHostApi.pigeonChannelCodec,
        ),
        (message) async => reply(message! as List<Object?>),
      );
    }

    test('sends the html, path, paper and margin; returns the pages', () async {
      List<Object?>? seen;
      answer((args) {
        seen = args;
        return <Object?>[3];
      });
      final html = File('test/fixture.html').readAsStringSync();
      final pages = await KansoPrint.htmlToPdf(
        html: html,
        outputPath: '/tmp/out.pdf',
        paper: KansoPaper.letter,
      );
      expect(pages, 3);
      expect(seen![0], withPrintStyles(html));
      expect(seen![1], '/tmp/out.pdf');
      expect(seen![2], PrintPaper.letter);
      expect(seen![3], 36.0);
    });

    test('A4 maps to the A4 paper', () async {
      List<Object?>? seen;
      answer((args) {
        seen = args;
        return <Object?>[1];
      });
      await KansoPrint.htmlToPdf(
        html: '<p/>',
        outputPath: 'x.pdf',
        paper: KansoPaper.a4,
      );
      expect(seen![2], PrintPaper.a4);
    });

    test('a native failure is a PlatformException with its message', () async {
      answer((_) => <Object?>['timeout', 'The page took too long.', null]);
      await expectLater(
        KansoPrint.htmlToPdf(
          html: '<p/>',
          outputPath: 'x.pdf',
          paper: KansoPaper.a4,
        ),
        throwsA(
          isA<PlatformException>()
              .having((e) => e.code, 'code', 'timeout')
              .having((e) => e.message, 'message', contains('too long')),
        ),
      );
    });
  });
}
