// Kanso — open_answer_link.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:url_launcher/url_launcher.dart';

/// Opens the web page [address] in the browser. Only `http` and `https`
/// addresses are opened; the answer's parser lets nothing else through, and
/// this checks again.
Future<void> openAnswerLink(String address) async {
  final uri = Uri.tryParse(address);
  if (uri == null || (uri.scheme != 'http' && uri.scheme != 'https')) {
    return;
  }
  await launchUrl(uri, mode: LaunchMode.externalApplication);
}
