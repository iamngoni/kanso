// Kanso — typing_report_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Any, Generator, GeneratorUtils, Glados2, IntAnys, ListAnys, any;
import 'package:kanso/core/models/data/frame_sample.dart';
import 'package:kanso/core/models/data/typing_report.dart';
import 'package:kanso/core/utils/typing_report.dart';

import '../../helpers/property.dart';

TypingReport report({
  double latencyP95 = 5,
  double latencyP99 = 8,
  double freshRunsP95 = 20,
  double applyP99 = 0.2,
  int rssBefore = 100,
  int rssAfter = 120,
  int keystrokes = 100,
  double refreshRate = 120,
}) {
  const megabyte = 1024 * 1024;
  return typingReport(
    words: 50000,
    characters: 300000,
    keystrokes: keystrokes,
    frames: <FrameSample>[
      for (var i = 0; i < 200; i++) const FrameSample(buildMs: 3, rasterMs: 4),
    ],
    jumpFrames: const <FrameSample>[
      FrameSample(buildMs: 30, rasterMs: 2),
      FrameSample(buildMs: 12, rasterMs: 2),
    ],
    windowMoves: 5,
    refreshRate: refreshRate,
    latenciesMs: <double>[
      for (var i = 0; i < 94; i++) latencyP95 / 2,
      latencyP95,
      for (var i = 0; i < 5; i++) latencyP99,
    ],
    applyMs: <double>[applyP99],
    buildSpanMs: const <double>[1],
    freshRunsMs: <double>[freshRunsP95],
    rssBeforeBytes: rssBefore * megabyte,
    rssAfterBytes: rssAfter * megabyte,
    clockSkewMs: 1,
  );
}

extension on Any {
  /// A keystroke time and the frames after it: build starts that only go
  /// up, each with a raster finish after it.
  Generator<List<(int, int)>> get orderedFrames =>
      listWithLengthInRange<int>(0, 30, intInRange(1, 20000)).map((steps) {
        final frames = <(int, int)>[];
        var at = 0;
        for (final step in steps) {
          at += step;
          frames.add((at, at + 1000 + step % 7000));
        }
        return frames;
      });
}

void main() {
  group('keystrokeLatencies', () {
    test('a keystroke is shown by the first frame that began after it', () {
      final latencies = keystrokeLatencies(
        <int>[1000, 2000],
        <(int, int)>[(500, 900), (1500, 4000), (2500, 6000)],
      );
      expect(latencies, <double>[3, 4]);
    });

    test('a frame that began at the keystroke shows it', () {
      expect(
        keystrokeLatencies(<int>[1000], <(int, int)>[(1000, 3000)]),
        <double>[2],
      );
    });

    test('a keystroke after the last frame is left out', () {
      expect(
        keystrokeLatencies(<int>[100, 9000], <(int, int)>[(500, 900)]),
        hasLength(1),
      );
      expect(keystrokeLatencies(<int>[100], const <(int, int)>[]), isEmpty);
    });

    Glados2<List<int>, List<(int, int)>>(
      any.list(any.intInRange(0, 400000)),
      any.orderedFrames,
      kansoExploreConfig(),
    ).test('each is the raster finish of the first frame at or after it', (
      injected,
      frames,
    ) {
      final latencies = keystrokeLatencies(injected, frames);
      final sorted = <int>[...injected]..sort();
      final expected = <double>[];
      for (final at in sorted) {
        final shown = frames.where((frame) => frame.$1 >= at);
        if (shown.isEmpty) {
          break;
        }
        expected.add((shown.first.$2 - at) / 1000);
      }
      expect(latencies, expected);
      // Never before the keystroke.
      expect(latencies.every((ms) => ms >= 0), isTrue);
    });
  });

  group('typingReport', () {
    test('takes percentiles and the memory growth', () {
      final made = report(rssBefore: 200, rssAfter: 230);
      expect(made.rssGrowthMb, 30);
      expect(made.latencyP95, 5);
      expect(made.latencyP99, 8);
      expect(made.applyCalls, 1);
      expect(made.frames.frames, 200);
    });

    test('keeps the frames of jumps apart from those of typing', () {
      final made = report();
      expect(made.frames.frames, 200);
      expect(made.jumpFrames, 2);
      expect(made.jumpBuildMax, 30);
      expect(made.windowMoves, 5);
    });

    test('writes JSON for the driver', () {
      final json = report().toJson();
      expect(json['words'], 50000);
      expect(json['rssGrowthMb'], 20);
      expect((json['frames']! as Map)['refreshRate'], 120);
    });
  });

  group('typingProblems', () {
    test('a calm run passes', () {
      expect(typingProblems(report(), minRefreshRate: 119), isEmpty);
    });

    test('says what is wrong', () {
      // 120 Hz: an interval is 8.33 ms, so 2 are 16.67 ms and 4 are 33.3.
      final slow = report(
        latencyP95: 20,
        latencyP99: 40,
        freshRunsP95: 150,
        applyP99: 2,
        rssAfter: 200,
      );
      final problems = typingProblems(slow);
      expect(problems, hasLength(5));
      expect(problems[0], contains('p95 keystroke to frame'));
      expect(problems[1], contains('p99 keystroke to frame'));
      expect(problems[2], contains('p95 fresh highlight'));
      expect(problems[3], contains('p99 kanso.apply'));
      expect(problems[4], contains('memory growth'));
    });

    test('the limits are inclusive', () {
      expect(
        typingProblems(
          report(
            latencyP95: 1000 / 120 * 2,
            latencyP99: 1000 / 120 * 4,
            freshRunsP95: 100,
            applyP99: 1,
            rssAfter: 150,
          ),
        ),
        isEmpty,
      );
    });

    test('nothing typed is a problem', () {
      expect(
        typingProblems(report(keystrokes: 0)),
        contains('nothing was typed'),
      );
    });

    test('asks the display to keep its rate', () {
      expect(
        typingProblems(report(refreshRate: 60), minRefreshRate: 119),
        <Matcher>[contains('60.0 Hz')],
      );
    });
  });
}
