// Kanso — seal_size.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// A size in the words the desktop's lock dialogs use ("84 MB", "3.0 MB",
/// "512 KB"), in units of 1024 as it counts them. [underOneKb] is said for
/// less than a kilobyte.
String sealSize(int bytes, {required String underOneKb}) {
  const kb = 1024.0;
  final size = bytes.toDouble();
  if (size >= kb * kb * 10) {
    return '${(size / (kb * kb)).toStringAsFixed(0)} MB';
  }
  if (size >= kb * kb) {
    return '${(size / (kb * kb)).toStringAsFixed(1)} MB';
  }
  if (size >= kb) {
    final kilobytes = size / kb;
    return '${(kilobytes < 1 ? 1 : kilobytes).toStringAsFixed(0)} KB';
  }
  return underOneKb;
}
