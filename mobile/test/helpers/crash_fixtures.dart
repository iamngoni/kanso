// Kanso — crash_fixtures.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

/// A frame of a MetricKit call stack, innermost last as the tree nests.
class MetricKitFrame {
  const MetricKitFrame({
    required this.binaryName,
    required this.offset,
    required this.binaryUuid,
  });

  final String binaryName;
  final int offset;
  final String binaryUuid;
}

/// A realistic `MXDiagnosticPayload.jsonRepresentation()` with one crash
/// diagnostic: the crashed thread's frames nested outermost-first, a thread
/// that did not crash, and [noise] in every text field MetricKit has.
String metricKitPayload({
  required List<MetricKitFrame> crashedOutermostFirst,
  String noise = 'secret',
  Object? exceptionType = 1,
  Object? signal = 11,
  String appVersion = '0.9.0',
  String appBuild = '57',
  String osVersion = 'iPhone OS 18.1 (22B83)',
  String deviceType = 'iPhone17,1',
  String timeStampEnd = '2026-10-26 07:00:00',
  String Function(MetricKitFrame frame)? binaryName,
  String? terminationReason,
}) {
  Map<String, dynamic>? nest(List<MetricKitFrame> frames) {
    if (frames.isEmpty) {
      return null;
    }
    final frame = frames.first;
    final inner = nest(frames.sublist(1));
    return <String, dynamic>{
      'binaryUUID': frame.binaryUuid,
      'offsetIntoBinaryTextSegment': frame.offset,
      'sampleCount': 1,
      'binaryName': binaryName?.call(frame) ?? frame.binaryName,
      'address': frame.offset + 0x100000000,
      if (inner != null) 'subFrames': <Map<String, dynamic>>[inner],
    };
  }

  return jsonEncode(<String, dynamic>{
    'timeStampBegin': '2026-10-25 07:00:00',
    'timeStampEnd': timeStampEnd,
    'crashDiagnostics': <Map<String, dynamic>>[
      <String, dynamic>{
        'version': '1.0.0',
        'callStackTree': <String, dynamic>{
          'callStackPerThread': true,
          'callStacks': <Map<String, dynamic>>[
            <String, dynamic>{
              'threadAttributed': false,
              'callStackRootFrames': <Map<String, dynamic>>[
                <String, dynamic>{
                  'binaryUUID': 'DEADBEEF-0000-0000-0000-000000000000',
                  'offsetIntoBinaryTextSegment': 7,
                  'sampleCount': 1,
                  'binaryName': 'other thread $noise',
                  'address': 7,
                },
              ],
            },
            <String, dynamic>{
              'threadAttributed': true,
              'callStackRootFrames': <Map<String, dynamic>>[
                ?nest(crashedOutermostFirst),
              ],
            },
          ],
        },
        'diagnosticMetaData': <String, dynamic>{
          'appBuildVersion': appBuild,
          'appVersion': appVersion,
          'bundleIdentifier': 'app.kanso.Kanso $noise',
          'deviceType': deviceType,
          'exceptionCode': 1,
          'exceptionType': exceptionType,
          'exceptionReason': <String, dynamic>{
            'composedMessage': 'Could not save "$noise"',
            'exceptionName': noise,
            'className': noise,
            'arguments': <String>[noise],
          },
          'osVersion': osVersion,
          'platformArchitecture': 'arm64e',
          'regionFormat': 'ZW',
          'signal': signal,
          'terminationReason':
              terminationReason ?? 'Namespace SIGNAL, Code 11 $noise',
          'virtualMemoryRegionInfo': '0x0 is not in any region. $noise',
          'pid': 4242,
        },
      },
    ],
  });
}

/// The main thread of an ANR trace, with [noise] where a trace holds free
/// text: the header, the thread names, the locks, the state lines.
String anrTrace({
  String noise = 'secret',
  String libPath = '/apex/com.android.runtime/lib64/bionic',
}) {
  return '''
----- pid 12345 at 2026-10-26 10:00:00.123456789+0200 -----
Cmd line: app.kanso.kanso $noise
Build fingerprint: 'google/panther/panther:15/AP3A.241005.015/12366759:user/release-keys'
ABI: 'arm64'
Build type: optimized

DALVIK THREADS (28):
"Signal Catcher" daemon prio=10 tid=5 Runnable
  | group="system" sCount=0
  at com.evil.Other.method(Other.java:1)

"main" prio=5 tid=1 Native
  | group="main" sCount=1 ucsCount=0 flags=1 obj=0x72c1d3a8 self=0xb4000079f5a3a010
  | sysTid=12345 nice=-10 cgrp=top-app sched=0/0 handle=0x7aa6a9a4f8 name=$noise
  | state=S schedstat=( 123 456 789 ) utm=100 stm=20 core=3 HZ=100
  | held mutexes=
  kernel: __switch_to+0x9c
  native: #00 pc 000000000004e7a4  $libPath/libc.so (syscall+28) (BuildId: 0123456789abcdef0123456789abcdef01234567)
  native: #01 pc 0000000000123456  /apex/com.android.art/lib64/libart.so (art::ConditionVariable::WaitHoldingLocks(art::Thread*)+144) (BuildId: abcdef0123456789abcdef01)
  native: #02 pc 0000000000001000  /data/app/~~x==/app.kanso.kanso-y==/base.apk!libflutter.so (offset 0x1000)
  native: #03 pc 0000000000002000  /data/app/~~x==/app.kanso.kanso-y==/lib/arm64/libapp.so
  at java.lang.Object.wait(Native Method)
  - waiting on <0x0a1b2c3d> (a java.lang.Object) held by thread 3 $noise
  at com.foo.Bar.baz(Bar.kt:12)
  at android.os.Looper.loop(Looper.java:288)
  at com.foo.Main.main(Main.java)

"Binder:12345_1" prio=5 tid=2 Native
  at com.evil.Binder.run(Binder.java:9)
''';
}
