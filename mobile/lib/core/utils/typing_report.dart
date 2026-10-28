// Kanso — typing_report.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/data/frame_sample.dart';
import '../models/data/typing_report.dart';
import 'frame_report.dart';

/// How long the first frame to start building after each keystroke took to
/// get that keystroke on the screen, in milliseconds.
///
/// [injectedUs] are the moments (microseconds, in the clock of the frame
/// timestamps) keystrokes were handed to the field; [frames] are the
/// frames' build start and raster finish in the same clock, in the order
/// they were drawn. A keystroke is shown by the first frame that began
/// building at or after it; one with no such frame (typed after the last)
/// is left out.
List<double> keystrokeLatencies(
  List<int> injectedUs,
  List<(int buildStartUs, int rasterFinishUs)> frames,
) {
  final latencies = <double>[];
  var frame = 0;
  final sorted = <int>[...injectedUs]..sort();
  for (final injected in sorted) {
    while (frame < frames.length && frames[frame].$1 < injected) {
      frame++;
    }
    if (frame == frames.length) {
      break;
    }
    latencies.add((frames[frame].$2 - injected) / 1000);
  }
  return latencies;
}

/// The report of a run of typing into a note of [words] words.
TypingReport typingReport({
  required int words,
  required int characters,
  required int keystrokes,
  required List<FrameSample> frames,
  required List<FrameSample> jumpFrames,
  required int windowMoves,
  required double refreshRate,
  required List<double> latenciesMs,
  required List<double> applyMs,
  required List<double> buildSpanMs,
  required List<double> freshRunsMs,
  required int rssBeforeBytes,
  required int rssAfterBytes,
  required double clockSkewMs,
}) {
  double p(List<double> values, int rank) =>
      percentile(<double>[...values]..sort(), rank);
  double sum(List<double> values) =>
      values.fold(0, (total, value) => total + value);
  const megabyte = 1024 * 1024;
  return TypingReport(
    words: words,
    characters: characters,
    keystrokes: keystrokes,
    frames: frameReport(frames, refreshRate),
    jumpFrames: jumpFrames.length,
    jumpBuildMax: jumpFrames.fold<double>(
      0,
      (most, frame) => frame.buildMs > most ? frame.buildMs : most,
    ),
    windowMoves: windowMoves,
    latencyP50: p(latenciesMs, 50),
    latencyP95: p(latenciesMs, 95),
    latencyP99: p(latenciesMs, 99),
    applyP50: p(applyMs, 50),
    applyP99: p(applyMs, 99),
    applyCalls: applyMs.length,
    applyTotalMs: sum(applyMs),
    buildSpanP50: p(buildSpanMs, 50),
    buildSpanP99: p(buildSpanMs, 99),
    buildSpanCalls: buildSpanMs.length,
    buildSpanTotalMs: sum(buildSpanMs),
    frameBuildTotalMs: sum(<double>[for (final f in frames) f.buildMs]),
    freshRunsP50: p(freshRunsMs, 50),
    freshRunsP95: p(freshRunsMs, 95),
    freshRunsTimed: freshRunsMs.length,
    rssBeforeMb: rssBeforeBytes / megabyte,
    rssAfterMb: rssAfterBytes / megabyte,
    clockSkewMs: clockSkewMs,
  );
}

/// The limits of the gate, besides the frames' (see [frameProblems]).
abstract final class TypingLimits {
  /// A keystroke shows within this many refresh intervals at the 95th
  /// percentile, and this many at the 99th.
  static const double latencyP95Intervals = 2;
  static const double latencyP99Intervals = 4;

  /// New colours arrive within this long at the 95th percentile.
  static const double freshRunsP95Ms = 100;

  /// The core's call for a change takes at most this long at the 99th
  /// percentile.
  static const double applyP99Ms = 1;

  /// Resident memory grows by at most this much over the long stretch of
  /// typing.
  static const double rssGrowthMb = 50;
}

/// What is wrong with [report] by the gate's measures, in words; none when it
/// passes. [minRefreshRate] asks something of the display as in
/// [frameProblems].
List<String> typingProblems(TypingReport report, {double minRefreshRate = 0}) {
  final interval = report.frames.budgetMs;
  String over(String what, double value, double limit, String unit) =>
      '$what ${value.toStringAsFixed(2)} $unit is over '
      '${limit.toStringAsFixed(2)} $unit';
  final p95Limit = TypingLimits.latencyP95Intervals * interval;
  final p99Limit = TypingLimits.latencyP99Intervals * interval;
  return <String>[
    ...frameProblems(report.frames, minRefreshRate: minRefreshRate),
    if (report.keystrokes == 0) 'nothing was typed',
    if (report.latencyP95 > p95Limit)
      over('p95 keystroke to frame', report.latencyP95, p95Limit, 'ms'),
    if (report.latencyP99 > p99Limit)
      over('p99 keystroke to frame', report.latencyP99, p99Limit, 'ms'),
    if (report.freshRunsP95 > TypingLimits.freshRunsP95Ms)
      over(
        'p95 fresh highlight',
        report.freshRunsP95,
        TypingLimits.freshRunsP95Ms,
        'ms',
      ),
    if (report.applyP99 > TypingLimits.applyP99Ms)
      over('p99 kanso.apply', report.applyP99, TypingLimits.applyP99Ms, 'ms'),
    if (report.rssGrowthMb > TypingLimits.rssGrowthMb)
      over('memory growth', report.rssGrowthMb, TypingLimits.rssGrowthMb, 'MB'),
  ];
}
