// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'preferences.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_Preferences _$PreferencesFromJson(Map<String, dynamic> json) => _Preferences(
  themeSelection: json['themeSelection'] == null
      ? const ThemeSelection()
      : ThemeSelection.fromJson(json['themeSelection'] as Map<String, dynamic>),
  fonts: json['fonts'] == null
      ? const KansoFonts()
      : KansoFonts.fromJson(json['fonts'] as Map<String, dynamic>),
  textSize: json['textSize'] == null
      ? 1.0
      : textScaleFromJson(json['textSize']),
  previewTextSize: json['previewTextSize'] == null
      ? 1.0
      : textScaleFromJson(json['previewTextSize']),
  interfaceTextSize: json['interfaceTextSize'] == null
      ? 1.0
      : interfaceTextScaleFromJson(json['interfaceTextSize']),
  iconTheme: json['iconTheme'] as String? ?? KansoIconSet.lucideId,
  diagramRenderer:
      $enumDecodeNullable(
        _$DiagramRendererEnumMap,
        json['diagramRenderer'],
        unknownValue: DiagramRenderer.pele,
      ) ??
      DiagramRenderer.pele,
  sort:
      $enumDecodeNullable(
        _$SortKeyEnumMap,
        json['sort'],
        unknownValue: SortKey.updated,
      ) ??
      SortKey.updated,
  descending: json['descending'] as bool? ?? true,
  backupNoticeSeen: json['backupNoticeSeen'] as bool? ?? false,
  biometricOfferSeen: json['biometricOfferSeen'] as bool? ?? false,
  defaultNotebookId: json['defaultNotebookId'] as String?,
  openTabs:
      (json['openTabs'] as List<dynamic>?)?.map((e) => e as String).toList() ??
      const <String>[],
  activeTab: (json['activeTab'] as num?)?.toInt() ?? 0,
  searchImageText: json['searchImageText'] as bool? ?? false,
  remindersEnabled: json['remindersEnabled'] as bool? ?? true,
  focusMode: json['focusMode'] as bool? ?? false,
  livePreviewInFocus: json['livePreviewInFocus'] as bool? ?? false,
  resurface: json['resurface'] as bool? ?? true,
  resurfaceCloseness: json['resurfaceCloseness'] as String? ?? 'default',
  onThisDay: json['onThisDay'] as bool? ?? true,
  reviewPrompt: json['reviewPrompt'] == null
      ? const ReviewPromptRecord()
      : ReviewPromptRecord.fromJson(
          json['reviewPrompt'] as Map<String, dynamic>,
        ),
);

Map<String, dynamic> _$PreferencesToJson(_Preferences instance) =>
    <String, dynamic>{
      'themeSelection': instance.themeSelection.toJson(),
      'fonts': instance.fonts.toJson(),
      'textSize': textScaleToJson(instance.textSize),
      'previewTextSize': textScaleToJson(instance.previewTextSize),
      'interfaceTextSize': interfaceTextScaleToJson(instance.interfaceTextSize),
      'iconTheme': instance.iconTheme,
      'diagramRenderer': _$DiagramRendererEnumMap[instance.diagramRenderer]!,
      'sort': _$SortKeyEnumMap[instance.sort]!,
      'descending': instance.descending,
      'backupNoticeSeen': instance.backupNoticeSeen,
      'biometricOfferSeen': instance.biometricOfferSeen,
      'defaultNotebookId': ?instance.defaultNotebookId,
      'openTabs': instance.openTabs,
      'activeTab': instance.activeTab,
      'searchImageText': instance.searchImageText,
      'remindersEnabled': instance.remindersEnabled,
      'focusMode': instance.focusMode,
      'livePreviewInFocus': instance.livePreviewInFocus,
      'resurface': instance.resurface,
      'resurfaceCloseness': instance.resurfaceCloseness,
      'onThisDay': instance.onThisDay,
      'reviewPrompt': instance.reviewPrompt.toJson(),
    };

const _$DiagramRendererEnumMap = {
  DiagramRenderer.pele: 'pele',
  DiagramRenderer.kanso: 'kanso',
};

const _$SortKeyEnumMap = {
  SortKey.updated: 'updated',
  SortKey.created: 'created',
  SortKey.title: 'title',
};
