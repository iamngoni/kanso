// Kanso — coverage_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// Keeps "every function of the bridge is tested, on this computer and on a
/// device" true as the bridge grows.
///
/// The functions are read from the generated Dart (`lib/core/bridge/api`).
/// The tests call them through the suites in `test/core/bridge/suites`, which
/// `test/core/bridge/*_test.dart` runs on this computer and
/// `integration_test/facade_test.dart` runs on a simulator or an emulator;
/// this checks that every function is called in the suites (or the helper
/// that opens a library), and every suite is run in both places.
void main() {
  final apiFiles = Directory('lib/core/bridge/api')
      .listSync()
      .whereType<File>()
      .where(
        (file) =>
            file.path.endsWith('.dart') &&
            !file.path.endsWith('types.dart') &&
            !file.path.endsWith('.freezed.dart'),
      )
      .toList();

  final functions = <_BridgeFunction>[];
  final declaration = RegExp(
    r'^(\s*)(static\s+)?[A-Za-z][\w<>?, \[\]]*\s+(\w+)\(',
    multiLine: true,
  );
  for (final file in apiFiles) {
    var currentClass = '';
    for (final line in file.readAsLinesSync()) {
      final klass = RegExp(r'^(?:abstract )?class (\w+)').firstMatch(line);
      if (klass != null) {
        currentClass = klass.group(1)!;
        continue;
      }
      final match = declaration.firstMatch(line);
      if (match == null || line.trimLeft().startsWith('//')) {
        continue;
      }
      final indented = match.group(1)!.isNotEmpty;
      if (indented && currentClass.isEmpty) {
        continue;
      }
      functions.add(
        _BridgeFunction(
          owner: indented ? currentClass : '',
          name: match.group(3)!,
          isStatic: match.group(2) != null,
        ),
      );
    }
  }

  String read(Iterable<String> paths) => paths
      .expand<File>(
        (path) => FileSystemEntity.isDirectorySync(path)
            ? Directory(path)
                  .listSync(recursive: true)
                  .whereType<File>()
                  .where((file) => file.path.endsWith('.dart'))
            : <File>[File(path)],
      )
      .map((file) => file.readAsStringSync())
      .join('\n');

  final suites = read(<String>['test/core/bridge/suites']);
  final opener = read(<String>['test/helpers/core_library.dart']);

  group('the bridge is covered', () {
    test('the functions are found', () {
      expect(functions.length, greaterThan(30));
      expect(
        functions.map((f) => f.name),
        containsAll(<String>['list', 'unlock', 'syncNow', 'version']),
      );
    });

    for (final function in functions) {
      test('${function.label} is called by a suite', () {
        final calls = function.isStatic
            ? RegExp('${function.owner}\\.${function.name}\\(')
            : function.owner.isEmpty
            ? RegExp('(?<![.\\w])${function.name}\\(')
            : RegExp('\\.${function.name}\\(');
        expect(calls.hasMatch('$suites\n$opener'), isTrue);
      });
    }

    final suiteFiles = Directory('test/core/bridge/suites')
        .listSync()
        .whereType<File>()
        .where((file) => file.path.endsWith('_suite.dart'));
    final onComputer = read(<String>['test/core/bridge']);
    final onDevice = File(
      'integration_test/facade_test.dart',
    ).readAsStringSync();
    for (final file in suiteFiles) {
      final name = file.uri.pathSegments.last.replaceAll('_suite.dart', '');
      final call = name.replaceAllMapped(
        RegExp('_([a-z])'),
        (match) => match.group(1)!.toUpperCase(),
      );
      test('the $name suite runs on this computer and on a device', () {
        expect(onComputer, contains("suites/${name}_suite.dart'"));
        expect(onDevice, contains('${name}_suite.dart'));
        expect(onDevice, contains('${call}Suite('));
      });
    }
  });
}

class _BridgeFunction {
  const _BridgeFunction({
    required this.owner,
    required this.name,
    required this.isStatic,
  });

  /// The class it is a member of; empty for a top-level function.
  final String owner;
  final String name;
  final bool isStatic;

  String get label => owner.isEmpty ? name : '$owner.$name';
}
