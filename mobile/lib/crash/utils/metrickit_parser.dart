// Kanso — metrickit_parser.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

import '../models/data/crash_trace.dart';
import 'crash_grammar.dart';
import 'crash_scrubber.dart';

/// A crash from MetricKit's JSON: `MXDiagnosticPayload.jsonRepresentation()`
/// (with `crashDiagnostics`) or `MXCrashDiagnostic.jsonRepresentation()`.
///
/// Read: the crashed thread's frames (binary name, offset into the binary,
/// binary UUID), `exceptionType`, `signal`, and the app version, build, OS
/// version and device type the diagnostic says. Never read: `exceptionReason`,
/// `terminationReason`, or any other text.
abstract final class MetricKitParser {
  /// The most crash diagnostics read from one payload.
  static const int _maxDiagnostics = 8;

  /// The deepest a call stack is followed.
  static const int _maxDepth = 4096;

  /// The largest text read.
  static const int _maxText = 16 * 1024 * 1024;

  static const Map<int, String> _exceptionTypes = <int, String>{
    1: 'EXC_BAD_ACCESS',
    2: 'EXC_BAD_INSTRUCTION',
    3: 'EXC_ARITHMETIC',
    4: 'EXC_EMULATION',
    5: 'EXC_SOFTWARE',
    6: 'EXC_BREAKPOINT',
    7: 'EXC_SYSCALL',
    8: 'EXC_MACH_SYSCALL',
    9: 'EXC_RPC_ALERT',
    10: 'EXC_CRASH',
    11: 'EXC_RESOURCE',
    12: 'EXC_GUARD',
    13: 'EXC_CORPSE_NOTIFY',
  };

  static const Map<int, String> _signals = <int, String>{
    1: 'SIGHUP',
    2: 'SIGINT',
    3: 'SIGQUIT',
    4: 'SIGILL',
    5: 'SIGTRAP',
    6: 'SIGABRT',
    7: 'SIGEMT',
    8: 'SIGFPE',
    9: 'SIGKILL',
    10: 'SIGBUS',
    11: 'SIGSEGV',
    12: 'SIGSYS',
    13: 'SIGPIPE',
    14: 'SIGALRM',
    15: 'SIGTERM',
  };

  /// The crashes in [json], one per crash diagnostic that has frames.
  /// Never throws.
  static List<CrashTrace> parse(String json) {
    try {
      if (json.length > _maxText) {
        return const <CrashTrace>[];
      }
      final root = jsonDecode(json);
      if (root is! Map<String, dynamic>) {
        return const <CrashTrace>[];
      }
      final day = _day(root['timeStampEnd']) ?? _day(root['timeStampBegin']);
      final diagnostics = root['crashDiagnostics'];
      final candidates = diagnostics is List
          ? diagnostics.take(_maxDiagnostics)
          : <Object?>[if (root.containsKey('callStackTree')) root];
      return <CrashTrace>[
        for (final candidate in candidates)
          if (candidate is Map<String, dynamic>) ?_trace(candidate, day),
      ];
    } on Object {
      return const <CrashTrace>[];
    }
  }

  static CrashTrace? _trace(Map<String, dynamic> diagnostic, String? day) {
    final frames = _frames(diagnostic['callStackTree']);
    if (frames.isEmpty) {
      return null;
    }
    final meta = diagnostic['diagnosticMetaData'];
    final data = meta is Map<String, dynamic>
        ? meta
        : const <String, dynamic>{};
    final exception = _integer(data['exceptionType']);
    final signal = _integer(data['signal']);
    // EXC_CRASH only says "the process was killed"; the signal says why.
    final type = exception != null && exception != 10
        ? _exceptionTypes[exception]
        : _signals[signal] ?? _exceptionTypes[exception];
    final code = _terminationCode(data['terminationReason']);
    final name = CrashScrubber.type(type);
    return CrashTrace(
      // `SIGKILL.0xdead10cc`: a kill by the system names its reason.
      type: code != null && name == 'SIGKILL' ? '$name.$code' : name,
      frames: frames,
      day: day,
      appVersion: CrashScrubber.appVersion(_text(data['appVersion'])),
      appBuild: CrashScrubber.appBuild(_text(data['appBuildVersion'])),
      osVersion: CrashScrubber.osVersionIn(_text(data['osVersion'])),
      device: CrashScrubber.device(_text(data['deviceType'])),
    );
  }

  /// The termination code in MetricKit's `terminationReason` (`Namespace
  /// RUNNINGBOARD, Code 0xdead10cc`): the eight hexadecimal digits after `0x`,
  /// in lower case, as `0x...`; null if there is not exactly that. Nothing
  /// else of the text is kept.
  static String? _terminationCode(Object? reason) {
    final text = _text(reason);
    if (text == null) {
      return null;
    }
    final found = RegExp(
      r'\b0x([0-9a-fA-F]{8})\b',
    ).allMatches(text).toList();
    return found.length == 1
        ? '0x${found.single.group(1)!.toLowerCase()}'
        : null;
  }

  /// The crashed thread's frames, innermost first.
  static List<String> _frames(Object? tree) {
    if (tree is! Map<String, dynamic>) {
      return const <String>[];
    }
    final stacks = tree['callStacks'];
    if (stacks is! List) {
      return const <String>[];
    }
    Map<String, dynamic>? crashed;
    for (final stack in stacks) {
      if (stack is Map<String, dynamic> && stack['threadAttributed'] == true) {
        crashed = stack;
        break;
      }
    }
    if (crashed == null) {
      return const <String>[];
    }
    // The tree runs from the outermost frame in; a crash is one path of it.
    final path = <String>[];
    Object? level = crashed['callStackRootFrames'];
    for (var depth = 0; depth < _maxDepth; depth++) {
      final node = _firstMap(level);
      if (node == null) {
        break;
      }
      final frame = _frame(node);
      if (frame != null) {
        path.add(frame);
      }
      level = node['subFrames'];
    }
    return path.reversed.take(CrashGrammar.maxFrames).toList();
  }

  static String? _frame(Map<String, dynamic> node) {
    final offset = _integer(node['offsetIntoBinaryTextSegment']);
    if (offset == null) {
      return null;
    }
    return CrashScrubber.nativeFrame(
      binary: _text(node['binaryName']),
      offset: offset,
      buildId: _text(node['binaryUUID']),
    );
  }

  static Map<String, dynamic>? _firstMap(Object? level) {
    if (level is Map<String, dynamic>) {
      return level;
    }
    if (level is List) {
      for (final item in level) {
        if (item is Map<String, dynamic>) {
          return item;
        }
      }
    }
    return null;
  }

  static int? _integer(Object? value) {
    if (value is int) {
      return value;
    }
    if (value is double && value.isFinite) {
      return value.toInt();
    }
    if (value is String && value.length < 24) {
      return int.tryParse(value);
    }
    return null;
  }

  static String? _text(Object? value) {
    if (value is String && value.length <= 256) {
      return value;
    }
    return null;
  }

  static String? _day(Object? stamp) {
    if (stamp is! String || stamp.length < 10) {
      return null;
    }
    return CrashScrubber.day(stamp.substring(0, 10));
  }
}
