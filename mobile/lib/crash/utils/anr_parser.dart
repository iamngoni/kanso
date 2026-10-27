// Kanso — anr_parser.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/data/crash_trace.dart';
import 'crash_grammar.dart';
import 'crash_scrubber.dart';

/// An application-not-responding trace: the text `ApplicationExitInfo`
/// gives for `REASON_ANR`. Only the main thread's section is read, only its
/// `native:` and `at` lines, and nothing from the rest of the file.
abstract final class AnrParser {
  /// The type of every ANR report.
  static const String type = 'ANR';

  static const int _maxText = 8 * 1024 * 1024;
  static const int _maxLine = 2000;

  static final RegExp _mainThread = RegExp(r'^"main"\s+(?:daemon\s+)?prio=');
  static final RegExp _native = RegExp(
    r'^\s+native:\s+#[0-9]+\s+pc\s+([0-9a-fA-F]{1,16})\s+(.*?)\s*$',
  );
  static final RegExp _buildId = RegExp(r'\s+\(BuildId:\s*([0-9a-fA-F]+)\)$');
  static final RegExp _java = RegExp(r'^\s+at\s+(\S+)\.([^.\s(]+)\((.*)\)\s*$');
  static final RegExp _symbolOffset = RegExp(r'^(.*)\+([0-9]+)$');
  static final RegExp _lineNumber = RegExp(r':([0-9]+)$');

  /// The main thread of [trace], or null if the trace has none or no frames.
  /// Never throws.
  static CrashTrace? parse(String trace) {
    try {
      final text = trace.length > _maxText
          ? trace.substring(0, _maxText)
          : trace;
      final frames = <String>[];
      var inMain = false;
      for (final line in text.split('\n')) {
        if (line.length > _maxLine) {
          continue;
        }
        if (!inMain) {
          inMain = _mainThread.hasMatch(line);
          continue;
        }
        // The next thread's header, or a blank line, ends the section.
        if (line.startsWith('"') || line.trim().isEmpty) {
          break;
        }
        final frame = _frame(line);
        if (frame != null && frames.length < CrashGrammar.maxFrames) {
          frames.add(frame);
        }
      }
      return frames.isEmpty ? null : CrashTrace(type: type, frames: frames);
    } on Object {
      return null;
    }
  }

  static String? _frame(String line) {
    final native = _native.firstMatch(line);
    if (native != null) {
      return _nativeFrame(
        int.parse(native.group(1)!, radix: 16),
        native.group(2)!,
      );
    }
    final java = _java.firstMatch(line);
    if (java != null) {
      final place = java.group(3)!;
      final number = _lineNumber.firstMatch(place);
      // The runtime's own conventions for "no line": -2 for a native method,
      // -1 for none known.
      final value = place == 'Native Method'
          ? -2
          : number == null
          ? -1
          : int.parse(number.group(1)!);
      return CrashScrubber.jvmFrame(
        cls: java.group(1)!,
        method: java.group(2)!,
        line: value,
      );
    }
    return null;
  }

  /// `<path> (<symbol>+<offset>) (BuildId: <id>)`, the symbol and the build
  /// ID optional. The directories of the path may hold anything, a " (" too:
  /// the symbol starts at the first " (" after the path's last `/`.
  static String? _nativeFrame(int offset, String rest) {
    var text = rest;
    String? buildId;
    final id = _buildId.firstMatch(text);
    if (id != null) {
      buildId = id.group(1);
      text = text.substring(0, id.start);
    }
    final from = text.lastIndexOf('/');
    final open = text.indexOf(' (', from < 0 ? 0 : from);
    var path = text;
    String? symbol;
    int? symbolOffset;
    if (open >= 0) {
      path = text.substring(0, open);
      final inner = text.substring(open + 2);
      final split = inner.endsWith(')')
          ? _symbolOffset.firstMatch(inner.substring(0, inner.length - 1))
          : null;
      if (split != null) {
        symbol = split.group(1);
        symbolOffset = int.tryParse(split.group(2)!);
      }
    }
    return CrashScrubber.nativeFrame(
      binary: CrashScrubber.baseName(path),
      offset: offset,
      symbol: symbol,
      symbolOffset: symbolOffset,
      buildId: buildId,
    );
  }
}
