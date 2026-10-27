// Kanso — fonts_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/kanso_font_family.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_scroll_host.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import 'font_choice_row.dart';
import 'text_size_section.dart';

/// Settings › Fonts: the editor font, the preview font and the code font,
/// each a list in which every family is shown in itself with the one in
/// use checked, then the text size and the preview's text size.
///
/// The choices are the [AppearanceBloc]'s, so choosing here restyles every
/// note in the app at once. [startAtEnd] shows the page scrolled to its
/// foot.
class FontsPane extends StatelessWidget {
  const FontsPane({this.startAtEnd = false, super.key});

  final bool startAtEnd;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    final bloc = context.read<AppearanceBloc>();
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            leading: const ShellBackButton(),
            title: l10n.fontsTitle,
          ),
          Expanded(
            child: KansoScrollHost(
              startAtEnd: startAtEnd,
              builder: (context, controller) {
                return BlocBuilder<AppearanceBloc, AppearanceState>(
                  builder: (context, state) {
                    return ListView(
                      controller: controller,
                      padding: EdgeInsets.fromLTRB(
                        KansoSpacing.md,
                        KansoSpacing.xs,
                        KansoSpacing.md,
                        bottom + KansoSpacing.lg,
                      ),
                      children: <Widget>[
                        KansoGroupedSection(
                          title: l10n.fontsEditorSection,
                          footer: l10n.fontsEditorFooter,
                          children: <Widget>[
                            for (final font in KansoFontFamily.editorChoices)
                              FontChoiceRow(
                                font: font,
                                sample: l10n.fontsSampleText,
                                selected: font == state.fonts.editorFont,
                                onTap: () => bloc.add(
                                  AppearanceEvent.editorFontChanged(font),
                                ),
                              ),
                          ],
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          title: l10n.fontsPreviewSection,
                          footer: l10n.fontsPreviewFooter,
                          children: <Widget>[
                            for (final font in KansoFontFamily.previewChoices)
                              FontChoiceRow(
                                font: font,
                                sample: l10n.fontsSampleText,
                                selected: font == state.fonts.previewFont,
                                onTap: () => bloc.add(
                                  AppearanceEvent.previewFontChanged(font),
                                ),
                              ),
                          ],
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          title: l10n.fontsCodeSection,
                          footer: l10n.fontsCodeFooter,
                          children: <Widget>[
                            for (final font in KansoFontFamily.codeChoices)
                              FontChoiceRow(
                                font: font,
                                sample: l10n.fontsSampleCode,
                                selected: font == state.fonts.codeFont,
                                onTap: () => bloc.add(
                                  AppearanceEvent.codeFontChanged(font),
                                ),
                              ),
                          ],
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        TextSizeSection(
                          selected: state.textSize,
                          onChanged: (size) => bloc.add(
                            AppearanceEvent.textSizeChanged(size),
                          ),
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        TextSizeSection(
                          title: l10n.fontsPreviewTextSizeSection,
                          footer: l10n.fontsPreviewTextSizeFooter,
                          selected: state.previewTextSize,
                          onChanged: (size) => bloc.add(
                            AppearanceEvent.previewTextSizeChanged(size),
                          ),
                        ),
                      ],
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
