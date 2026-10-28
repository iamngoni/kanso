// Kanso — graph_page.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../state/graph/graph_bloc.dart';
import '../widgets/graph_canvas.dart';
import '../widgets/graph_filter_picker.dart';

/// The full library graph or a note's two-hop neighbourhood.
class GraphPage extends StatelessWidget {
  const GraphPage({this.centerId, this.hops = 2, super.key});

  final String? centerId;
  final int hops;

  @override
  Widget build(BuildContext context) {
    final request = centerId == null
        ? const GraphRequest.library_(filter: GraphFilter.all())
        : GraphRequest.local(center: centerId!, hops: hops);
    return BlocProvider<GraphBloc>(
      create: (_) => getIt<GraphBloc>()..add(GraphEvent.started(request)),
      child: Scaffold(
        backgroundColor: context.colors.app,
        body: Column(
          children: <Widget>[
            KansoTopBar(
              title: context.l10n.graphTitle,
              leading: KansoIconButton(
                icon: KansoIcons.arrowLeft,
                semanticLabel: context.l10n.settingsBack,
                onPressed: () => context.pop(),
              ),
            ),
            Expanded(
              child: BlocBuilder<GraphBloc, GraphState>(
                builder: (context, state) => Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: <Widget>[
                    if (centerId == null)
                      Padding(
                        padding: const EdgeInsets.all(KansoSpacing.md),
                        child: GraphFilterPicker(
                          request: state.request,
                          onSelected: (filter) => context.read<GraphBloc>().add(
                            GraphEvent.filterSelected(filter),
                          ),
                        ),
                      ),
                    Expanded(
                      child: state.loading && state.layout == null
                          ? const Center(child: CircularProgressIndicator())
                          : state.failed
                          ? Center(
                              child: Column(
                                mainAxisSize: MainAxisSize.min,
                                children: <Widget>[
                                  Text(context.l10n.graphFailed),
                                  KansoButton(
                                    label: context.l10n.retry,
                                    onPressed: () => context
                                        .read<GraphBloc>()
                                        .add(GraphEvent.started(state.request)),
                                  ),
                                ],
                              ),
                            )
                          : state.layout == null || state.layout!.nodes.isEmpty
                          ? Center(
                              child: Text(
                                context.l10n.graphEmpty,
                                style: context.typography.secondary,
                              ),
                            )
                          : GraphCanvas(
                              layout: state.layout!,
                              onNodeSelected: (node) =>
                                  GoRouter.of(context).push(
                                    '${KansoRoutes.noteById}/${node.id}',
                                  ),
                            ),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
