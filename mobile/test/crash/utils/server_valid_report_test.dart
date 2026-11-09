// Kanso — server_valid_report_test.dart
// Created 2026-11-09.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/crash/models/data/crash_app.dart';
import 'package:kanso/crash/models/data/crash_context.dart';
import 'package:kanso/crash/models/data/crash_kind.dart';
import 'package:kanso/crash/models/data/crash_os.dart';
import 'package:kanso/crash/models/data/crash_os_name.dart';
import 'package:kanso/crash/utils/crash_report_assembler.dart';
import 'package:kanso/crash/utils/crash_report_text.dart';
import 'package:kanso/crash/utils/metrickit_parser.dart';

import '../../helpers/crash_fixtures.dart';
import '../../helpers/server_validator.dart';

/// A MetricKit SIGKILL of an idle backgrounded app: a deep CoreFoundation /
/// libsystem_kernel run loop stack under Runner and Flutter frames.
List<MetricKitFrame> _idleStack({int depth = 200}) => <MetricKitFrame>[
  const MetricKitFrame(
    binaryName: 'dyld',
    offset: 0x1a2b,
    binaryUuid: 'AAAAAAAA-1111-2222-3333-444444444444',
  ),
  const MetricKitFrame(
    binaryName: 'Runner',
    offset: 0x4a21c,
    binaryUuid: 'BBBBBBBB-1111-2222-3333-444444444444',
  ),
  for (var i = 0; i < depth; i++)
    MetricKitFrame(
      binaryName: i.isEven ? 'Flutter' : 'Kanso Pocket Runner',
      offset: 0x90000 + i * 16,
      binaryUuid: 'CCCCCCCC-1111-2222-3333-444444444444',
    ),
  const MetricKitFrame(
    binaryName: 'CoreFoundation',
    offset: 0x5f3a0,
    binaryUuid: 'DDDDDDDD-1111-2222-3333-444444444444',
  ),
  const MetricKitFrame(
    binaryName: 'libsystem_kernel.dylib',
    offset: 0x1b64,
    binaryUuid: 'EEEEEEEE-1111-2222-3333-444444444444',
  ),
];

CrashContext _context({int? patch}) => CrashContext(
  app: CrashApp(version: '0.9.4', build: '75', patch: patch),
  os: const CrashOs(name: CrashOsName.ios, version: '27.2'),
  device: 'iPhone18,1',
);

String? _text({
  int? patch,
  String appBuild = '75',
  String? termination,
  int depth = 200,
}) {
  final payload = metricKitPayload(
    crashedOutermostFirst: _idleStack(depth: depth),
    exceptionType: 10,
    signal: 9,
    appVersion: '0.9.4',
    appBuild: appBuild,
    osVersion: 'iPhone OS 27.2 (24C101)',
    deviceType: 'iPhone18,1',
    terminationReason: termination,
  );
  final trace = MetricKitParser.parse(payload).single;
  final report = CrashReportAssembler.assemble(
    kind: CrashKind.iosNative,
    trace: trace,
    context: _context(patch: patch),
    fallbackDay: '2026-11-09',
  );
  return report == null ? null : crashReportText(report);
}

void main() {
  group('a MetricKit SIGKILL report against the server rules', () {
    test('the running build with no Shorebird patch is accepted '
        '(patch is sent as null, not left out)', () {
      final text = _text()!;
      expect(text, contains('"patch": null'));
      expect(serverRejects(text), isNull);
    });

    test('an older build (patch unknown) is accepted', () {
      expect(serverRejects(_text(patch: 3, appBuild: '74')!), isNull);
    });

    test('the running build with a patch is accepted', () {
      expect(serverRejects(_text(patch: 3)!), isNull);
    });

    test('a 4000-deep stack is cut to 128 frames, innermost first', () {
      final text = _text(depth: 4000)!;
      expect(serverRejects(text), isNull);
      expect(text, contains('libsystem_kernel.dylib'));
    });

    test('the termination reason goes in type', () {
      final text = _text(
        termination: 'Namespace RUNNINGBOARD, Code 0xDEAD10CC',
      )!;
      expect(text, contains('"type": "SIGKILL.0xdead10cc"'));
      expect(serverRejects(text), isNull);
    });

    test('a termination reason with no code leaves type at SIGKILL', () {
      final text = _text(termination: 'secret words')!;
      expect(text, contains('"type": "SIGKILL"'));
      expect(text, isNot(contains('secret')));
    });
  });

  group('the port of the server rules', () {
    test('rejects a body without patch', () {
      expect(
        serverRejects(
          '{"v":1,"kind":"ios-native","type":"SIGKILL",'
          '"app":{"version":"0.9.4","build":"75"},'
          '"os":{"name":"ios","version":"27.2"},'
          '"device":"iPhone18,1","day":"2026-11-09",'
          '"frames":["native ? +0x1"]}',
        ),
        'app.patch',
      );
    });
  });
}
