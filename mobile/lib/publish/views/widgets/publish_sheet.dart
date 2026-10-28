// Kanso — publish_sheet.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../injection.dart';
import '../../state/publish/publish_bloc.dart';

/// The mobile note or notebook site publish sheet.
class PublishSheet extends StatelessWidget {
  const PublishSheet({super.key});

  static Future<void> showNote(BuildContext context, String id) =>
      _show(context, id, notebook: false);

  static Future<void> showNotebook(BuildContext context, String id) =>
      _show(context, id, notebook: true);

  static Future<void> _show(
    BuildContext context,
    String id, {
    required bool notebook,
  }) {
    final appearance = context.read<AppearanceBloc>().state;
    final dark = switch (appearance.themeMode) {
      ThemeMode.dark => true,
      ThemeMode.light => false,
      ThemeMode.system =>
        MediaQuery.platformBrightnessOf(context) == Brightness.dark,
    };
    final palette = (dark ? appearance.dark : appearance.light).preview;
    final options = PreviewOptions(
      font: appearance.fonts.previewFont.family,
      mono: appearance.fonts.codeFont.family,
      size: 16 * appearance.textSize.scale,
      utcOffsetSecs: DateTime.now().timeZoneOffset.inSeconds,
      code: const <SyntaxStyleRow>[],
    );
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => BlocProvider<PublishBloc>(
        create: (_) => getIt<PublishBloc>()
          ..add(
            notebook
                ? PublishEvent.openNotebook(
                    id: id,
                    palette: palette,
                    options: options,
                  )
                : PublishEvent.openNote(
                    id: id,
                    palette: palette,
                    options: options,
                  ),
          ),
        child: const PublishSheet(),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<PublishBloc, PublishState>(
      builder: (context, state) {
        final view = switch (state) {
          PublishIdleState(:final view) => view,
          PublishPublishingState(:final view) => view,
          PublishPublishedState(:final view) => view,
          PublishFailedState(:final view) => view,
        };
        final busy = state is PublishPublishingState;
        final published = state is PublishPublishedState;
        final link = switch (state) {
          PublishPublishedState(:final link) => link,
          _ => view?.link,
        };
        final failure = switch (state) {
          PublishFailedState(:final text) => text,
          _ => null,
        };
        if (view == null) {
          return const Padding(
            padding: EdgeInsets.all(KansoSpacing.lg),
            child: Center(child: CircularProgressIndicator()),
          );
        }
        final title = view.notebook
            ? l10n.publishNotebookTitle
            : l10n.publishNoteTitle;
        final summary = view.notebook
            ? view.summary
            : l10n.publishNoteContentSummary(
                view.contentPreview.trim().isEmpty
                    ? 0
                    : view.contentPreview.trim().split(RegExp(r'\s+')).length,
              );
        return Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.lg,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              Text(title, style: context.typography.barTitle),
              const SizedBox(height: KansoSpacing.sm),
              Text(view.title, style: context.typography.listTitle),
              const SizedBox(height: KansoSpacing.xs),
              Text(summary, style: context.typography.body),
              if (!view.notebook &&
                  view.contentPreview.trim().isNotEmpty) ...<Widget>[
                const SizedBox(height: KansoSpacing.xs),
                Text(
                  view.contentPreview.trim(),
                  maxLines: 5,
                  overflow: TextOverflow.ellipsis,
                  style: context.typography.secondary,
                ),
              ],
              if (view.locked.isNotEmpty) ...<Widget>[
                const SizedBox(height: KansoSpacing.xs),
                Text(view.locked, style: context.typography.secondary),
              ],
              if (view.refusal != null) ...<Widget>[
                const SizedBox(height: KansoSpacing.sm),
                Text(
                  view.refusal!,
                  style: TextStyle(color: context.colors.danger),
                ),
              ],
              if (failure != null) ...<Widget>[
                const SizedBox(height: KansoSpacing.sm),
                Text(failure, style: TextStyle(color: context.colors.danger)),
              ],
              if (view.refusal == null) ...<Widget>[
                const SizedBox(height: KansoSpacing.md),
                Text(
                  l10n.publishExpiryHeading,
                  style: context.typography.sectionLabel,
                ),
                const SizedBox(height: KansoSpacing.xs),
                Wrap(
                  spacing: KansoSpacing.xs,
                  children: <Widget>[
                    for (final option in view.expiryOptions)
                      ChoiceChip(
                        label: Text(option.label),
                        selected: view.expiry == option.expiry,
                        onSelected: busy
                            ? null
                            : (_) => context.read<PublishBloc>().add(
                                PublishEvent.expiryPicked(option.expiry),
                              ),
                      ),
                  ],
                ),
              ],
              if (link != null) ...<Widget>[
                const SizedBox(height: KansoSpacing.md),
                SelectableText(link.url, style: context.typography.body),
                const SizedBox(height: KansoSpacing.xs),
                Text(
                  link.expiresAtMs != null &&
                          link.expiresAtMs! <=
                              DateTime.now().millisecondsSinceEpoch
                      ? l10n.publishExpired
                      : link.expiresAtMs == null
                      ? l10n.publishNeverExpires
                      : l10n.publishExpiresOn(
                          MaterialLocalizations.of(context).formatMediumDate(
                            DateTime.fromMillisecondsSinceEpoch(
                              link.expiresAtMs!,
                            ),
                          ),
                        ),
                  style: context.typography.secondary,
                ),
              ],
              const SizedBox(height: KansoSpacing.md),
              if (busy)
                Center(
                  child: Text(
                    l10n.publishWorking,
                    style: context.typography.secondary,
                  ),
                )
              else if (published)
                Wrap(
                  spacing: KansoSpacing.sm,
                  runSpacing: KansoSpacing.sm,
                  children: <Widget>[
                    if (view.needsUpdate || view.expiry != link!.expiry)
                      FilledButton(
                        onPressed: () => context.read<PublishBloc>().add(
                          PublishEvent.publish(view.expiry),
                        ),
                        child: Text(
                          view.notebook
                              ? l10n.publishUpdateSite
                              : l10n.publishUpdate,
                        ),
                      ),
                    FilledButton.icon(
                      onPressed: () async {
                        await Clipboard.setData(ClipboardData(text: link!.url));
                        if (context.mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text(l10n.publishCopied)),
                          );
                        }
                      },
                      icon: const Icon(Icons.copy),
                      label: Text(l10n.publishCopyLink),
                    ),
                    OutlinedButton(
                      onPressed: () => context.read<PublishBloc>().add(
                        const PublishEvent.unpublish(),
                      ),
                      child: Text(l10n.publishUnpublish),
                    ),
                  ],
                )
              else
                FilledButton(
                  onPressed: view.refusal == null
                      ? () => context.read<PublishBloc>().add(
                          PublishEvent.publish(view.expiry),
                        )
                      : null,
                  child: Text(
                    view.link == null
                        ? (view.notebook
                              ? l10n.publishPublishSite
                              : l10n.publishPublish)
                        : (view.notebook
                              ? l10n.publishUpdateSite
                              : l10n.publishUpdate),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}
