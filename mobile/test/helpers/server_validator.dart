// Kanso — server_validator.dart
// Created 2026-11-09.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

/// A port of `validateReport` in cloud/src/crash_reports.ts, written
/// independently of `CrashGrammar`, so a test can ask "would the server take
/// this body?". Returns the failing field's path, or null when it is valid.
/// Keep it in step with the worker.
String? serverRejects(String body, {int maxBytes = 32 * 1024}) {
  final Object? input;
  try {
    input = _decode(body);
  } on FormatException {
    return '(not json)';
  }
  if (_bytes(body) > maxBytes) {
    return '(too large)';
  }
  final top = _exactly(input, '', const <String>[
    'v',
    'kind',
    'type',
    'app',
    'os',
    'device',
    'day',
    'frames',
  ]);
  if (top.$2 != null) {
    return top.$2;
  }
  final r = top.$1!;
  if (r['v'] != 1) {
    return 'v';
  }
  const kinds = <String>[
    'dart',
    'ios-native',
    'android-native',
    'android-anr',
    'android-jvm',
  ];
  if (r['kind'] is! String || !kinds.contains(r['kind'])) {
    return 'kind';
  }
  if (!_text(r['type'], RegExp(r'^[A-Za-z_$][A-Za-z0-9_$.]{0,127}$'))) {
    return 'type';
  }
  final app = _exactly(r['app'], 'app', const <String>[
    'version',
    'build',
    'patch',
  ]);
  if (app.$2 != null) {
    return app.$2;
  }
  final a = app.$1!;
  if (!_text(a['version'], RegExp(r'^[0-9]{1,4}(\.[0-9]{1,4}){2}$'))) {
    return 'app.version';
  }
  if (!_text(a['build'], RegExp(r'^[0-9]{1,10}$'))) {
    return 'app.build';
  }
  final patch = a['patch'];
  if (patch != null && !(patch is int && patch >= 0 && patch <= 100000)) {
    return 'app.patch';
  }
  final os = _exactly(r['os'], 'os', const <String>['name', 'version']);
  if (os.$2 != null) {
    return os.$2;
  }
  if (os.$1!['name'] != 'ios' && os.$1!['name'] != 'android') {
    return 'os.name';
  }
  if (!_text(os.$1!['version'], RegExp(r'^[0-9A-Za-z._-]{1,32}$'))) {
    return 'os.version';
  }
  if (!_text(r['device'], RegExp(r'^[A-Za-z0-9 ,._()-]{1,64}$'))) {
    return 'device';
  }
  if (!_text(r['day'], RegExp(r'^[0-9]{4}-[0-9]{2}-[0-9]{2}$'))) {
    return 'day';
  }
  final frames = r['frames'];
  if (frames is! List || frames.isEmpty || frames.length > 128) {
    return 'frames';
  }
  for (var i = 0; i < frames.length; i++) {
    final frame = frames[i];
    if (frame is! String || !_validFrame(frame)) {
      return 'frames[$i]';
    }
  }
  return null;
}

int _bytes(String s) => utf8.encode(s).length;

Object? _decode(String body) => jsonDecode(body);

(Map<String, dynamic>?, String?) _exactly(
  Object? value,
  String path,
  List<String> keys,
) {
  if (value is! Map<String, dynamic>) {
    return (null, path.isEmpty ? '(body)' : path);
  }
  for (final key in keys) {
    if (!value.containsKey(key)) {
      return (null, path.isEmpty ? key : '$path.$key');
    }
  }
  return value.length == keys.length
      ? (value, null)
      : (null, path.isEmpty ? '(body)' : path);
}

bool _text(Object? v, RegExp p) => v is String && p.hasMatch(v);

final RegExp _dartSymbol = RegExp(r'^[A-Za-z_$<][A-Za-z0-9_$.<>]{0,199}$');
final RegExp _dartUri = RegExp(
  r'^(?:package:[a-z0-9_]{1,64}/[A-Za-z0-9_/.-]{1,200}\.dart|dart:[a-z0-9_/.-]{1,100})$',
);
final RegExp _number = RegExp(r'^[0-9]{1,7}$');
final RegExp _hex = RegExp(r'^[0-9a-f]{1,16}$');
final RegExp _offset = RegExp(r'^\+0x[0-9a-f]{1,16}$');
final RegExp _binary = RegExp(r'^[A-Za-z0-9_.+-]{1,64}$');
final RegExp _nativeSymbol = RegExp(r'^[A-Za-z0-9_$.]{1,256}$');
final RegExp _buildId = RegExp(r'^[0-9a-fA-F-]{8,64}$');
final RegExp _jvmClass = RegExp(r'^[A-Za-z_$][A-Za-z0-9_$.]{0,199}$');
final RegExp _jvmMethod = RegExp(r'^[A-Za-z_$<][A-Za-z0-9_$<>]{0,99}$');
final RegExp _jvmLine = RegExp(r'^-?[0-9]{1,7}$');

bool _dartLocation(String text) {
  if (text == '?') {
    return true;
  }
  final parts = text.split(':');
  if (parts.length < 3 || parts.length > 4) {
    return false;
  }
  return _dartUri.hasMatch('${parts[0]}:${parts[1]}') &&
      parts.skip(2).every(_number.hasMatch);
}

bool _nativeSym(String text) {
  final plus = text.lastIndexOf('+0x');
  return plus > 0 &&
      _nativeSymbol.hasMatch(text.substring(0, plus)) &&
      _hex.hasMatch(text.substring(plus + 3));
}

bool _jvmLocation(String text) {
  final colon = text.lastIndexOf(':');
  if (colon < 0 || !_jvmLine.hasMatch(text.substring(colon + 1))) {
    return false;
  }
  final dot = text.lastIndexOf('.', colon);
  return dot > 0 &&
      _jvmClass.hasMatch(text.substring(0, dot)) &&
      _jvmMethod.hasMatch(text.substring(dot + 1, colon));
}

bool _validFrame(String frame) {
  if (frame.length > 300) {
    return false;
  }
  final parts = frame.split(' ');
  switch (parts[0]) {
    case 'dart':
      return parts.length == 3 &&
          (parts[1] == '?' || _dartSymbol.hasMatch(parts[1])) &&
          _dartLocation(parts[2]);
    case 'dart-pc':
      return parts.length == 2 && _offset.hasMatch(parts[1]);
    case 'native':
      if (parts.length < 3 ||
          parts.length > 5 ||
          !(parts[1] == '?' || _binary.hasMatch(parts[1])) ||
          !_offset.hasMatch(parts[2])) {
        return false;
      }
      final rest = parts.sublist(3);
      if (rest.isNotEmpty && rest[0].contains('+')) {
        return _nativeSym(rest[0]) &&
            (rest.length == 1 ||
                (rest.length == 2 && _buildId.hasMatch(rest[1])));
      }
      return rest.isEmpty || (rest.length == 1 && _buildId.hasMatch(rest[0]));
    case 'jvm':
      return parts.length == 2 && _jvmLocation(parts[1]);
    default:
      return false;
  }
}
