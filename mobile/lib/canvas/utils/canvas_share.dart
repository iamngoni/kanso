// Kanso — canvas_share.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';
import 'dart:ui';

import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:share_plus/share_plus.dart';

/// Writes [content] to a file called [name] in the temporary folder and
/// offers it to the system's share sheet. [origin] is where the sheet points
/// on a tablet, which asks for one.
Future<void> shareCanvasFile(
  String content, {
  required String name,
  required String mimeType,
  Rect? origin,
}) async {
  final folder = await getTemporaryDirectory();
  final path = p.join(folder.path, name);
  await File(path).writeAsString(content);
  await SharePlus.instance.share(
    ShareParams(
      files: <XFile>[XFile(path, mimeType: mimeType, name: name)],
      sharePositionOrigin: origin,
    ),
  );
}

/// A file name for a canvas titled [title]: letters, digits and dashes.
String canvasFileName(String title, String extension) {
  final slug = title
      .trim()
      .toLowerCase()
      .replaceAll(RegExp('[^a-z0-9]+'), '-')
      .replaceAll(RegExp(r'^-+|-+$'), '');
  return '${slug.isEmpty ? 'canvas' : slug}.$extension';
}
