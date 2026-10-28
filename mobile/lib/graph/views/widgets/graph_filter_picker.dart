// Kanso — graph_filter_picker.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';

/// Chooses all notes, a notebook, or a tag for the library graph.
class GraphFilterPicker extends StatelessWidget {
  const GraphFilterPicker({
    required this.request,
    required this.onSelected,
    super.key,
  });

  final GraphRequest request;
  final ValueChanged<GraphFilter> onSelected;

  @override
  Widget build(BuildContext context) {
    final index = context.watch<LibraryIndexBloc>().state;
    final selected = switch (request) {
      GraphRequest_Library(filter: GraphFilter_All()) => 'all',
      GraphRequest_Library(filter: GraphFilter_Book(:final id)) => 'book:$id',
      GraphRequest_Library(filter: GraphFilter_Tag(:final id)) => 'tag:$id',
      GraphRequest_Local() => 'all',
    };
    final items = <DropdownMenuItem<String>>[
      DropdownMenuItem<String>(
        value: 'all',
        child: Text(context.l10n.graphFilterAll),
      ),
      for (final notebook in index.notebooks.where((row) => !row.locked))
        DropdownMenuItem<String>(
          value: 'book:${notebook.id}',
          child: Text('${context.l10n.graphFilterNotebook}: ${notebook.name}'),
        ),
      for (final tag in index.tags)
        DropdownMenuItem<String>(
          value: 'tag:${tag.id}',
          child: Text('${context.l10n.graphFilterTag}: ${tag.name}'),
        ),
    ];
    return DropdownButton<String>(
      value: items.any((item) => item.value == selected) ? selected : 'all',
      isExpanded: true,
      items: items,
      onChanged: (value) {
        if (value == null || value == 'all') {
          onSelected(const GraphFilter.all());
        } else if (value.startsWith('book:')) {
          onSelected(GraphFilter.book(id: value.substring(5)));
        } else if (value.startsWith('tag:')) {
          onSelected(GraphFilter.tag(id: value.substring(4)));
        }
      },
    );
  }
}
