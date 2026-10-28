// Kanso — frame_report.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/data/frame_report.dart';
import '../models/data/frame_sample.dart';

/// The report for [samples] on a display of [refreshRate] hertz.
///
/// A frame is over budget when either thread took longer than one refresh
/// interval. Percentiles are the nearest rank: the p-th percentile of n
/// frames is the `ceil(p / 100 * n)`-th shortest.
FrameReport frameReport(List<FrameSample> samples, double refreshRate) {
  final budget = 1000 / refreshRate;
  final builds = <double>[for (final sample in samples) sample.buildMs]..sort();
  final rasters = <double>[for (final sample in samples) sample.rasterMs]
    ..sort();
  return FrameReport(
    frames: samples.length,
    refreshRate: refreshRate,
    budgetMs: budget,
    buildP50: percentile(builds, 50),
    buildP90: percentile(builds, 90),
    buildP99: percentile(builds, 99),
    rasterP50: percentile(rasters, 50),
    rasterP90: percentile(rasters, 90),
    rasterP99: percentile(rasters, 99),
    framesOverBudget: samples
        .where((sample) => sample.buildMs > budget || sample.rasterMs > budget)
        .length,
  );
}

/// The [p]-th percentile (1 to 100) of [sorted], smallest first, by nearest
/// rank; 0 for none.
double percentile(List<double> sorted, int p) {
  if (sorted.isEmpty) {
    return 0;
  }
  final rank = (p * sorted.length / 100).ceil();
  return sorted[(rank - 1).clamp(0, sorted.length - 1)];
}

/// What is wrong with [report] by the scrolling check's measure, in words;
/// none when it passes.
///
/// It passes when the 99th percentile of both threads is under the budget,
/// at most [maxOverBudgetShare] of the frames were over it, and the display
/// ran at [minRefreshRate] hertz or more (0 asks nothing of the display).
List<String> frameProblems(
  FrameReport report, {
  double minRefreshRate = 0,
  double maxOverBudgetShare = 0.01,
}) {
  final budget = report.budgetMs.toStringAsFixed(2);
  final problems = <String>[];
  if (report.frames == 0) {
    problems.add('no frames were drawn');
  }
  if (report.refreshRate < minRefreshRate) {
    problems.add(
      'the display ran at ${report.refreshRate.toStringAsFixed(1)} Hz, '
      'not $minRefreshRate Hz or more',
    );
  }
  if (report.buildP99 >= report.budgetMs) {
    problems.add(
      'p99 build ${report.buildP99.toStringAsFixed(2)} ms is not under '
      'the $budget ms budget',
    );
  }
  if (report.rasterP99 >= report.budgetMs) {
    problems.add(
      'p99 raster ${report.rasterP99.toStringAsFixed(2)} ms is not under '
      'the $budget ms budget',
    );
  }
  if (report.overBudgetShare > maxOverBudgetShare) {
    problems.add(
      '${report.framesOverBudget} of ${report.frames} frames were over the '
      '$budget ms budget (more than ${maxOverBudgetShare * 100}%)',
    );
  }
  return problems;
}
