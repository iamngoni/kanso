// Kanso — attachment_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:share_plus/share_plus.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// An attachment tapped in a preview: an image full screen, where it can be
/// pinched and moved; anything else (a PDF, a file) in a sheet with its name
/// and size, to share it with the app that opens it.
Future<void> showAttachment(BuildContext context, AttachmentFile file) {
  if (file.contentType.startsWith('image/')) {
    return Navigator.of(context, rootNavigator: true).push<void>(
      MaterialPageRoute<void>(
        fullscreenDialog: true,
        builder: (_) => AttachmentImagePage(file: file),
      ),
    );
  }
  return showKansoBottomSheet<void>(
    context,
    builder: (_) => AttachmentSheet(file: file),
  );
}

/// [bytes] as a person reads a file's size: 512 B, 3.4 KB, 12 MB.
String byteSize(int bytes) {
  const units = <String>['B', 'KB', 'MB', 'GB'];
  var size = bytes.toDouble();
  var unit = 0;
  while (size >= 1000 && unit < units.length - 1) {
    size /= 1000;
    unit++;
  }
  final digits = unit == 0 || size >= 10 ? 0 : 1;
  return '${size.toStringAsFixed(digits)} ${units[unit]}';
}

/// Hands [file] to the system's share sheet, from which it can be opened,
/// saved or sent.
Future<void> shareAttachment(AttachmentFile file) => SharePlus.instance.share(
  ShareParams(
    files: <XFile>[
      XFile.fromData(file.bytes, name: file.name, mimeType: file.contentType),
    ],
    fileNameOverrides: <String>[file.name],
  ),
);

class AttachmentSheet extends StatelessWidget {
  const AttachmentSheet({required this.file, super.key});

  final AttachmentFile file;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Padding(
      padding: EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.md + MediaQuery.paddingOf(context).bottom,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Row(
            children: <Widget>[
              Expanded(
                child: Text(
                  file.name,
                  style: context.typography.barTitle,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              KansoPressable(
                onTap: () => Navigator.of(context).maybePop(),
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.sm),
                  child: Text(
                    l10n.attachmentDone,
                    style: context.typography.body.copyWith(
                      color: colors.accent,
                    ),
                  ),
                ),
              ),
            ],
          ),
          Text(
            l10n.attachmentDetails(
              file.contentType,
              byteSize(file.bytes.length),
            ),
            style: context.typography.secondary,
          ),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: l10n.attachmentShare,
            onPressed: () => shareAttachment(file),
          ),
        ],
      ),
    );
  }
}

/// An image attachment over everything, on black, to pinch and move; Done
/// closes it and Share hands it on.
class AttachmentImagePage extends StatelessWidget {
  const AttachmentImagePage({required this.file, super.key});

  final AttachmentFile file;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final light = context.typography.body.copyWith(color: Colors.white);
    return Scaffold(
      backgroundColor: Colors.black,
      body: Stack(
        children: <Widget>[
          Positioned.fill(
            child: InteractiveViewer(
              maxScale: 8,
              child: Center(
                child: Image.memory(
                  file.bytes,
                  semanticLabel: file.name,
                  gaplessPlayback: true,
                ),
              ),
            ),
          ),
          SafeArea(
            child: Row(
              children: <Widget>[
                KansoPressable(
                  onTap: () => Navigator.of(context).maybePop(),
                  child: Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: Text(l10n.attachmentDone, style: light),
                  ),
                ),
                const Spacer(),
                KansoPressable(
                  onTap: () => shareAttachment(file),
                  child: Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: Text(l10n.attachmentShare, style: light),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
