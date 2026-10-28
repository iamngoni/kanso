// Kanso — bench_results.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:kanso/core/models/data/typing_report.dart';

/// The numbers of a finished typing run, as text on a screen.
class BenchResults extends StatelessWidget {
  const BenchResults({required this.report, required this.problems, super.key});

  final TypingReport report;
  final List<String> problems;

  @override
  Widget build(BuildContext context) {
    String ms(double value) => value.toStringAsFixed(2);
    final frames = report.frames;
    final text = StringBuffer()
      ..writeln('${report.words} words, ${report.characters} units')
      ..writeln(
        '${report.keystrokes} keystrokes, ${frames.frames} frames at '
        '${frames.refreshRate.round()} Hz (${ms(frames.budgetMs)} ms)',
      )
      ..writeln(
        'build p99 ${ms(frames.buildP99)}, raster p99 '
        '${ms(frames.rasterP99)}, ${frames.framesOverBudget} over',
      )
      ..writeln(
        '${report.windowMoves} window moves, ${report.jumpFrames} frames of '
        'jumps (slowest ${ms(report.jumpBuildMax)} ms)',
      )
      ..writeln(
        'keystroke to frame p50/p95/p99 ${ms(report.latencyP50)} / '
        '${ms(report.latencyP95)} / ${ms(report.latencyP99)} ms',
      )
      ..writeln(
        'apply p99 ${ms(report.applyP99)} ms, span p99 '
        '${ms(report.buildSpanP99)} ms',
      )
      ..writeln(
        'fresh highlight p50/p95 ${ms(report.freshRunsP50)} / '
        '${ms(report.freshRunsP95)} ms',
      )
      ..writeln(
        'memory ${ms(report.rssBeforeMb)} to ${ms(report.rssAfterMb)} MB '
        '(+${ms(report.rssGrowthMb)})',
      )
      ..writeln(
        problems.isEmpty ? 'PASS' : problems.map((p) => 'FAIL: $p').join('\n'),
      );
    return MaterialApp(
      home: Scaffold(
        body: SafeArea(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Text(text.toString()),
          ),
        ),
      ),
    );
  }
}
