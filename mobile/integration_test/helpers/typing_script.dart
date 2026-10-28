// Kanso — typing_script.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// This file is shared by the typing test and by the bench app, which runs on
// a phone without `flutter_test`: it must not import it.

import 'dart:async';
import 'dart:developer';
import 'dart:io';
import 'dart:ui' show FramePhase, FrameTiming;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:kanso/core/models/data/frame_sample.dart';
import 'package:kanso/core/models/data/typing_report.dart';
import 'package:kanso/core/utils/typing_report.dart';
import 'package:kanso/editor/utils/editor_timing.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';

/// The state of the editor's field on screen, or null when there is none.
KansoEditableTextState? findEditorState() {
  KansoEditableTextState? found;
  void visit(Element element) {
    if (found != null) {
      return;
    }
    if (element is StatefulElement && element.state is KansoEditableTextState) {
      found = element.state as KansoEditableTextState;
      return;
    }
    element.visitChildren(visit);
  }

  WidgetsBinding.instance.rootElement?.visitChildren(visit);
  return found;
}

/// What a person types into a long note, typed into the field of the open
/// editor as the platform's text input would: through
/// `EditableTextState.updateEditingValue`, the method its channel calls.
///
/// Three hundred characters at the end of the note and three hundred in the
/// middle, at a steady pace and in bursts; Return in a list; one paste; some
/// undos; then a long stretch of steady typing for the memory measure.
class TypingScript {
  const TypingScript({
    this.characters = 300,
    this.steadyPerSecond = 10,
    this.burstPerSecond = 30,
    this.returns = 20,
    this.pasteUnits = 2048,
    this.undos = 20,
    this.soak = const Duration(minutes: 2),
  });

  /// The script of the gate.
  static const TypingScript full = TypingScript();

  /// A short script that makes a report and proves nothing about speed: for
  /// debug builds, simulators and emulators.
  static const TypingScript quick = TypingScript(
    characters: 30,
    returns: 5,
    pasteUnits: 256,
    undos: 5,
    soak: Duration(seconds: 3),
  );

  /// How many characters each of the four stretches types.
  final int characters;
  final int steadyPerSecond;
  final int burstPerSecond;

  /// How many times Return is pressed at the end of a list item.
  final int returns;

  /// How long the pasted text is, in UTF-16 units.
  final int pasteUnits;
  final int undos;

  /// How long the final stretch of steady typing lasts.
  final Duration soak;

  /// Types the script into the open editor, whose note has [words] words,
  /// and reports it.
  Future<TypingReport> run({required int words}) async {
    final state = findEditorState();
    if (state == null) {
      throw StateError('There is no editor on screen to type into.');
    }
    final controller = state.widget.controller as KansoMarkdownController;
    final noteLength = controller.noteLength;

    // The colours of the whole note are asked for first, so that typing
    // starts from a steady state.
    final started = Stopwatch()..start();
    while (controller.runs.isEmpty &&
        started.elapsed < const Duration(minutes: 2)) {
      await Future<void>.delayed(const Duration(milliseconds: 100));
    }
    await _settle();

    final frames = <FrameSample>[];
    final jumpFrames = <FrameSample>[];
    // When the caret was being put somewhere far away, in the clock of the
    // frames' timestamps: what is drawn then is not typing.
    final jumps = <(int, int)>[];
    final shown = <(int, int)>[];
    final injected = <int>[];
    final applyMs = <double>[];
    final spanMs = <double>[];
    final freshMs = <double>[];
    var keystrokes = 0;
    var lag = double.infinity;
    final firstWindow = controller.windowRevision;

    void onFrames(List<FrameTiming> timings) {
      final now = Timeline.now;
      for (final timing in timings) {
        final sample = FrameSample(
          buildMs: timing.buildDuration.inMicroseconds / 1000,
          rasterMs: timing.rasterDuration.inMicroseconds / 1000,
        );
        final started = timing.timestampInMicroseconds(FramePhase.buildStart);
        final finish = timing.timestampInMicroseconds(FramePhase.rasterFinish);
        if (jumps.any((jump) => started >= jump.$1 && started <= jump.$2)) {
          jumpFrames.add(sample);
          continue;
        }
        frames.add(sample);
        shown.add((started, finish));
        final behind = (now - finish) / 1000;
        if (behind < lag) {
          lag = behind;
        }
      }
    }

    EditorTiming.observer = (name, micros) {
      final ms = micros / 1000;
      switch (name) {
        case 'kanso.apply':
          applyMs.add(ms);
        case 'kanso.buildTextSpan':
          spanMs.add(ms);
        case 'kanso.freshRuns':
          freshMs.add(ms);
      }
    };
    SchedulerBinding.instance.addTimingsCallback(onFrames);

    void keystroke() {
      keystrokes++;
      injected.add(Timeline.now);
    }

    void insert(String text) {
      final value = state.currentTextEditingValue;
      final selection = value.selection;
      final at = selection.isValid ? selection.start : value.text.length;
      final end = selection.isValid ? selection.end : at;
      keystroke();
      state.updateEditingValue(
        TextEditingValue(
          text: value.text.replaceRange(at, end, text),
          selection: TextSelection.collapsed(offset: at + text.length),
        ),
      );
    }

    Future<void> paced(
      int count,
      int perSecond,
      void Function(int) step,
    ) async {
      final period = Duration(microseconds: 1000000 ~/ perSecond);
      final clock = Stopwatch()..start();
      for (var i = 0; i < count; i++) {
        final wait = period * i - clock.elapsed;
        if (wait > Duration.zero) {
          await Future<void>.delayed(wait);
        }
        step(i);
      }
    }

    // Offsets are the note's: the field may hold only a window of it.
    Future<void> moveTo(int offset) async {
      final from = Timeline.now;
      controller.placeCaret(offset);
      await _settle();
      jumps.add((from, Timeline.now));
    }

    int middle() {
      final text = controller.noteText;
      final half = text.length ~/ 2;
      final lineStart = text.lastIndexOf('\n', half);
      return lineStart < 0 ? half : lineStart;
    }

    const alphabet = 'the quick brown fox jumps over a lazy dog ';
    String letters(int i) => alphabet[i % alphabet.length];

    try {
      // Steady typing at the end, then in the middle; then the same in
      // bursts.
      for (final perSecond in <int>[steadyPerSecond, burstPerSecond]) {
        await moveTo(controller.noteLength);
        await paced(
          characters,
          perSecond,
          (i) => insert(letters(i)),
        );
        await moveTo(middle());
        await paced(
          characters,
          perSecond,
          (i) => insert(letters(i)),
        );
      }

      // Return in a list: each Return starts the next item.
      await moveTo(controller.noteLength);
      insert('\n\n- first');
      await _settle();
      await paced(returns, steadyPerSecond, (i) {
        insert('\n');
        insert(letters(i));
      });
      await _settle();

      // One paste.
      await moveTo(middle());
      final pasted = StringBuffer();
      for (var i = 0; pasted.length < pasteUnits; i++) {
        pasted.write('pasted$i ');
      }
      insert(pasted.toString().substring(0, pasteUnits));
      await _settle();

      // Undos, as the accessory bar's key or the system's gesture does.
      await paced(undos, steadyPerSecond, (_) {
        keystroke();
        controller.undo();
      });
      await _settle();

      // A long stretch of steady typing: how much memory it costs.
      await moveTo(controller.noteLength);
      final rssBefore = ProcessInfo.currentRss;
      final soakChars = soak.inMilliseconds * steadyPerSecond ~/ 1000;
      await paced(soakChars, steadyPerSecond, (i) => insert(letters(i)));
      await _settle();
      final rssAfter = ProcessInfo.currentRss;

      // The engine hands over frame timings in batches, a moment after the
      // frames.
      await Future<void>.delayed(const Duration(seconds: 2));
      return typingReport(
        words: words,
        characters: noteLength,
        keystrokes: keystrokes,
        frames: frames,
        jumpFrames: jumpFrames,
        windowMoves: controller.windowRevision - firstWindow,
        refreshRate: WidgetsBinding
            .instance
            .platformDispatcher
            .views
            .first
            .display
            .refreshRate,
        latenciesMs: keystrokeLatencies(injected, shown),
        applyMs: applyMs,
        buildSpanMs: spanMs,
        freshRunsMs: freshMs,
        rssBeforeBytes: rssBefore,
        rssAfterBytes: rssAfter,
        clockSkewMs: lag.isFinite ? lag : 0,
      );
    } finally {
      SchedulerBinding.instance.removeTimingsCallback(onFrames);
      EditorTiming.observer = null;
    }
  }

  /// Lets the frames and the core's answers that follow what was just done
  /// arrive.
  static Future<void> _settle() async {
    await Future<void>.delayed(const Duration(milliseconds: 300));
    await WidgetsBinding.instance.endOfFrame;
  }
}
