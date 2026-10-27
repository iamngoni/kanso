// Kanso: import_flow_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'import_flow_bloc.dart';

@freezed
abstract class ImportFlowState with _$ImportFlowState {
  const factory ImportFlowState({
    @Default(ImportStage.sources) ImportStage stage,

    /// Every source a phone can import from, in the order to list them.
    @Default(<ImportSource>[]) List<ImportSource> sources,

    /// The kinds a given file could be, best first; empty when the list is
    /// not narrowed.
    @Default(<String>[]) List<String> only,

    /// The source being read or imported.
    ImportSource? source,

    /// What is being read or imported.
    ImportPick? pick,

    /// Whether [pick] is a copy the pickers made (and so is removed when the
    /// flow ends) and not a file somebody else handed over.
    @Default(false) bool picked,

    /// The system's picker is open.
    @Default(false) bool picking,

    /// How far reading or importing has got.
    ImportProgress? progress,

    /// What the export holds ([ImportStage.summary]) or what the import did
    /// ([ImportStage.done]).
    ImportSummary? summary,

    /// Why a step did not happen.
    ImportProblem? problem,
  }) = _ImportFlowState;

  const ImportFlowState._();

  /// The sources the list shows: the narrowed ones when the flow was opened
  /// with a file, else all of them.
  List<ImportSource> get listed => only.isEmpty
      ? sources
      : <ImportSource>[
          for (final kind in only)
            for (final source in sources)
              if (source.kind == kind) source,
        ];
}
