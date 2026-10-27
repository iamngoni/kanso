// Kanso — typing_report.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'frame_report.dart';

part 'typing_report.freezed.dart';

/// What typing into a long note cost: the frames drawn while it was typed,
/// how long a keystroke took to show, what the core's calls took, how soon
/// fresh colours arrived, and how much memory grew. The numbers of the
/// editor's performance gate.
@freezed
abstract class TypingReport with _$TypingReport {
  const factory TypingReport({
    /// How many words and UTF-16 units the note had when typing began.
    required int words,
    required int characters,

    /// How many keystrokes (insertions, Returns, undos) were made.
    required int keystrokes,

    /// The frames drawn while typing, against the display's refresh interval.
    required FrameReport frames,

    /// The frames drawn while the caret was put somewhere far away (the end
    /// of the note, its middle): which of them took longest, in
    /// milliseconds, and how many there were. They are not in [frames]: a
    /// person scrolls to such a place before tapping it.
    required int jumpFrames,
    required double jumpBuildMax,

    /// How many times the window moved or was replaced in all.
    required int windowMoves,

    /// From a keystroke being handed to the field to the end of the raster
    /// of the frame that shows it: 50th, 95th and 99th percentile, in
    /// milliseconds.
    required double latencyP50,
    required double latencyP95,
    required double latencyP99,

    /// The median and 99th percentile of the field's call to the core for a
    /// change (`kanso.apply`), how many calls were timed, and all their time,
    /// in milliseconds.
    required double applyP50,
    required double applyP99,
    required int applyCalls,
    required double applyTotalMs,

    /// The same for building the text's spans (`kanso.buildTextSpan`) when
    /// they were not already built.
    required double buildSpanP50,
    required double buildSpanP99,
    required int buildSpanCalls,
    required double buildSpanTotalMs,

    /// All the time the frames took to build, on the UI thread, in
    /// milliseconds: what [applyTotalMs] and [buildSpanTotalMs] are part of
    /// (or, for the calls outside a frame, are not).
    required double frameBuildTotalMs,

    /// From a change to the fresh colours of its text being shown: 50th and
    /// 95th percentile, in milliseconds, and how many were timed.
    required double freshRunsP50,
    required double freshRunsP95,
    required int freshRunsTimed,

    /// The process's resident memory before and after the long stretch of
    /// typing, in megabytes.
    required double rssBeforeMb,
    required double rssAfterMb,

    /// How far the engine's clock was ahead of the end of the last frame
    /// when it was reported, in milliseconds: small and positive when the
    /// timestamps of frames and the clock keystrokes are stamped with are
    /// the same.
    required double clockSkewMs,
  }) = _TypingReport;

  const TypingReport._();

  /// How much resident memory grew over the long stretch of typing.
  double get rssGrowthMb => rssAfterMb - rssBeforeMb;

  /// The report as JSON, for the test driver and the bench app to write down.
  Map<String, Object> toJson() => <String, Object>{
    'words': words,
    'characters': characters,
    'keystrokes': keystrokes,
    'frames': frames.toJson(),
    'jumpFrames': jumpFrames,
    'jumpBuildMax': jumpBuildMax,
    'windowMoves': windowMoves,
    'latencyP50': latencyP50,
    'latencyP95': latencyP95,
    'latencyP99': latencyP99,
    'applyP50': applyP50,
    'applyP99': applyP99,
    'applyCalls': applyCalls,
    'applyTotalMs': applyTotalMs,
    'buildSpanP50': buildSpanP50,
    'buildSpanP99': buildSpanP99,
    'buildSpanCalls': buildSpanCalls,
    'buildSpanTotalMs': buildSpanTotalMs,
    'frameBuildTotalMs': frameBuildTotalMs,
    'freshRunsP50': freshRunsP50,
    'freshRunsP95': freshRunsP95,
    'freshRunsTimed': freshRunsTimed,
    'rssBeforeMb': rssBeforeMb,
    'rssAfterMb': rssAfterMb,
    'rssGrowthMb': rssGrowthMb,
    'clockSkewMs': clockSkewMs,
  };
}
