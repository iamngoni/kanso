// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'preferences.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$Preferences {

/// The themes the interface is drawn in.
 ThemeSelection get themeSelection;/// The fonts of notes.
 KansoFonts get fonts;/// The size of the text of notes in the editor, from 0.8 to 1.8.
@JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0) double get textSize;/// The size of the rendered preview's text, from 0.8 to 1.8.
@JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0) double get previewTextSize;/// The size of the interface's labels and text, from 0.9 to 1.3.
@JsonKey(fromJson: interfaceTextScaleFromJson, toJson: interfaceTextScaleToJson, defaultValue: 1.0) double get interfaceTextSize;/// The icon theme the app is drawn in, by its ID. A theme this version
/// does not know is Kanso's own icons.
 String get iconTheme;/// What draws Mermaid diagrams.
@JsonKey(unknownEnumValue: DiagramRenderer.pele) DiagramRenderer get diagramRenderer;/// Whether overflowing list scrollbars stay visible between scrolls.
 bool get alwaysShowScrollbars;/// What note lists are sorted by.
@JsonKey(unknownEnumValue: SortKey.updated) SortKey get sort;/// Whether lists run from the largest to the smallest sort value (the
/// newest first, Z to A).
 bool get descending;/// Whether the notice that this phone's notes are not in device backups
/// has been shown.
 bool get backupNoticeSeen;/// Whether the offer to unlock locked notes with a biometric, made once
/// after the first password unlock, has been shown.
 bool get biometricOfferSeen;/// The notebook new notes go in, unless they are made from a notebook's
/// own list. Null, or one that is gone or locked, is the first notebook.
 String? get defaultNotebookId;/// The notes open as tabs on a tablet, left to right (the desktop keeps
/// them in its settings too). Notes that are gone by the next start are
/// dropped then. Empty on a phone, which has no tabs.
 List<String> get openTabs;/// The index in [openTabs] of the tab that was showing.
 int get activeTab;/// Whether pictures attached to notes are read on this device for search.
 bool get searchImageText;/// Whether this phone may show reminders for tasks with a due time.
/// Notification permission is asked separately by the operating system.
 bool get remindersEnabled;/// Whether the open note is shown by itself, without the library, the
/// list, tabs or the note's bars.
 bool get focusMode;/// Whether focus mode shows the open note in Live, and leaving it goes
/// back to the mode the note was in.
 bool get livePreviewInFocus;/// Whether the editor offers past notes about the paragraph being
/// written, in a pill above the keyboard (when related notes are on).
 bool get resurface;/// How close a past note must be to be offered: "closer", "default" or
/// "looser". A value this version does not know is "default".
 String get resurfaceCloseness;/// Whether a daily note lists the notes written on its date in earlier
/// years.
 bool get onThisDay;/// What decides whether to ask for a review of the app: when it was
/// first opened, how often it is used, and when it last asked.
 ReviewPromptRecord get reviewPrompt;/// Whether the capture sheet was last left in Task mode, so it opens
/// there again.
 bool get captureAsTask;/// Where a captured task is written (Settings › Captured tasks go to);
/// the same two places as the desktop's setting.
@JsonKey(unknownEnumValue: CaptureTarget.dailyNote) CaptureTarget get captureTaskTarget;/// The PDFs whose pages are shown as printed rather than smart-inverted
/// in a dark theme, keyed by attachment ID.
 Map<String, bool> get pdfOriginalPages;
/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreferencesCopyWith<Preferences> get copyWith => _$PreferencesCopyWithImpl<Preferences>(this as Preferences, _$identity);

  /// Serializes this Preferences to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Preferences&&(identical(other.themeSelection, themeSelection) || other.themeSelection == themeSelection)&&(identical(other.fonts, fonts) || other.fonts == fonts)&&(identical(other.textSize, textSize) || other.textSize == textSize)&&(identical(other.previewTextSize, previewTextSize) || other.previewTextSize == previewTextSize)&&(identical(other.interfaceTextSize, interfaceTextSize) || other.interfaceTextSize == interfaceTextSize)&&(identical(other.iconTheme, iconTheme) || other.iconTheme == iconTheme)&&(identical(other.diagramRenderer, diagramRenderer) || other.diagramRenderer == diagramRenderer)&&(identical(other.alwaysShowScrollbars, alwaysShowScrollbars) || other.alwaysShowScrollbars == alwaysShowScrollbars)&&(identical(other.sort, sort) || other.sort == sort)&&(identical(other.descending, descending) || other.descending == descending)&&(identical(other.backupNoticeSeen, backupNoticeSeen) || other.backupNoticeSeen == backupNoticeSeen)&&(identical(other.biometricOfferSeen, biometricOfferSeen) || other.biometricOfferSeen == biometricOfferSeen)&&(identical(other.defaultNotebookId, defaultNotebookId) || other.defaultNotebookId == defaultNotebookId)&&const DeepCollectionEquality().equals(other.openTabs, openTabs)&&(identical(other.activeTab, activeTab) || other.activeTab == activeTab)&&(identical(other.searchImageText, searchImageText) || other.searchImageText == searchImageText)&&(identical(other.remindersEnabled, remindersEnabled) || other.remindersEnabled == remindersEnabled)&&(identical(other.focusMode, focusMode) || other.focusMode == focusMode)&&(identical(other.livePreviewInFocus, livePreviewInFocus) || other.livePreviewInFocus == livePreviewInFocus)&&(identical(other.resurface, resurface) || other.resurface == resurface)&&(identical(other.resurfaceCloseness, resurfaceCloseness) || other.resurfaceCloseness == resurfaceCloseness)&&(identical(other.onThisDay, onThisDay) || other.onThisDay == onThisDay)&&(identical(other.reviewPrompt, reviewPrompt) || other.reviewPrompt == reviewPrompt)&&(identical(other.captureAsTask, captureAsTask) || other.captureAsTask == captureAsTask)&&(identical(other.captureTaskTarget, captureTaskTarget) || other.captureTaskTarget == captureTaskTarget)&&const DeepCollectionEquality().equals(other.pdfOriginalPages, pdfOriginalPages));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hashAll([runtimeType,themeSelection,fonts,textSize,previewTextSize,interfaceTextSize,iconTheme,diagramRenderer,alwaysShowScrollbars,sort,descending,backupNoticeSeen,biometricOfferSeen,defaultNotebookId,const DeepCollectionEquality().hash(openTabs),activeTab,searchImageText,remindersEnabled,focusMode,livePreviewInFocus,resurface,resurfaceCloseness,onThisDay,reviewPrompt,captureAsTask,captureTaskTarget,const DeepCollectionEquality().hash(pdfOriginalPages)]);

@override
String toString() {
  return 'Preferences(themeSelection: $themeSelection, fonts: $fonts, textSize: $textSize, previewTextSize: $previewTextSize, interfaceTextSize: $interfaceTextSize, iconTheme: $iconTheme, diagramRenderer: $diagramRenderer, alwaysShowScrollbars: $alwaysShowScrollbars, sort: $sort, descending: $descending, backupNoticeSeen: $backupNoticeSeen, biometricOfferSeen: $biometricOfferSeen, defaultNotebookId: $defaultNotebookId, openTabs: $openTabs, activeTab: $activeTab, searchImageText: $searchImageText, remindersEnabled: $remindersEnabled, focusMode: $focusMode, livePreviewInFocus: $livePreviewInFocus, resurface: $resurface, resurfaceCloseness: $resurfaceCloseness, onThisDay: $onThisDay, reviewPrompt: $reviewPrompt, captureAsTask: $captureAsTask, captureTaskTarget: $captureTaskTarget, pdfOriginalPages: $pdfOriginalPages)';
}


}

/// @nodoc
abstract mixin class $PreferencesCopyWith<$Res>  {
  factory $PreferencesCopyWith(Preferences value, $Res Function(Preferences) _then) = _$PreferencesCopyWithImpl;
@useResult
$Res call({
 ThemeSelection themeSelection, KansoFonts fonts,@JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0) double textSize,@JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0) double previewTextSize,@JsonKey(fromJson: interfaceTextScaleFromJson, toJson: interfaceTextScaleToJson, defaultValue: 1.0) double interfaceTextSize, String iconTheme,@JsonKey(unknownEnumValue: DiagramRenderer.pele) DiagramRenderer diagramRenderer, bool alwaysShowScrollbars,@JsonKey(unknownEnumValue: SortKey.updated) SortKey sort, bool descending, bool backupNoticeSeen, bool biometricOfferSeen, String? defaultNotebookId, List<String> openTabs, int activeTab, bool searchImageText, bool remindersEnabled, bool focusMode, bool livePreviewInFocus, bool resurface, String resurfaceCloseness, bool onThisDay, ReviewPromptRecord reviewPrompt, bool captureAsTask,@JsonKey(unknownEnumValue: CaptureTarget.dailyNote) CaptureTarget captureTaskTarget, Map<String, bool> pdfOriginalPages
});


$ThemeSelectionCopyWith<$Res> get themeSelection;$KansoFontsCopyWith<$Res> get fonts;$ReviewPromptRecordCopyWith<$Res> get reviewPrompt;

}
/// @nodoc
class _$PreferencesCopyWithImpl<$Res>
    implements $PreferencesCopyWith<$Res> {
  _$PreferencesCopyWithImpl(this._self, this._then);

  final Preferences _self;
  final $Res Function(Preferences) _then;

/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? themeSelection = null,Object? fonts = null,Object? textSize = null,Object? previewTextSize = null,Object? interfaceTextSize = null,Object? iconTheme = null,Object? diagramRenderer = null,Object? alwaysShowScrollbars = null,Object? sort = null,Object? descending = null,Object? backupNoticeSeen = null,Object? biometricOfferSeen = null,Object? defaultNotebookId = freezed,Object? openTabs = null,Object? activeTab = null,Object? searchImageText = null,Object? remindersEnabled = null,Object? focusMode = null,Object? livePreviewInFocus = null,Object? resurface = null,Object? resurfaceCloseness = null,Object? onThisDay = null,Object? reviewPrompt = null,Object? captureAsTask = null,Object? captureTaskTarget = null,Object? pdfOriginalPages = null,}) {
  return _then(_self.copyWith(
themeSelection: null == themeSelection ? _self.themeSelection : themeSelection // ignore: cast_nullable_to_non_nullable
as ThemeSelection,fonts: null == fonts ? _self.fonts : fonts // ignore: cast_nullable_to_non_nullable
as KansoFonts,textSize: null == textSize ? _self.textSize : textSize // ignore: cast_nullable_to_non_nullable
as double,previewTextSize: null == previewTextSize ? _self.previewTextSize : previewTextSize // ignore: cast_nullable_to_non_nullable
as double,interfaceTextSize: null == interfaceTextSize ? _self.interfaceTextSize : interfaceTextSize // ignore: cast_nullable_to_non_nullable
as double,iconTheme: null == iconTheme ? _self.iconTheme : iconTheme // ignore: cast_nullable_to_non_nullable
as String,diagramRenderer: null == diagramRenderer ? _self.diagramRenderer : diagramRenderer // ignore: cast_nullable_to_non_nullable
as DiagramRenderer,alwaysShowScrollbars: null == alwaysShowScrollbars ? _self.alwaysShowScrollbars : alwaysShowScrollbars // ignore: cast_nullable_to_non_nullable
as bool,sort: null == sort ? _self.sort : sort // ignore: cast_nullable_to_non_nullable
as SortKey,descending: null == descending ? _self.descending : descending // ignore: cast_nullable_to_non_nullable
as bool,backupNoticeSeen: null == backupNoticeSeen ? _self.backupNoticeSeen : backupNoticeSeen // ignore: cast_nullable_to_non_nullable
as bool,biometricOfferSeen: null == biometricOfferSeen ? _self.biometricOfferSeen : biometricOfferSeen // ignore: cast_nullable_to_non_nullable
as bool,defaultNotebookId: freezed == defaultNotebookId ? _self.defaultNotebookId : defaultNotebookId // ignore: cast_nullable_to_non_nullable
as String?,openTabs: null == openTabs ? _self.openTabs : openTabs // ignore: cast_nullable_to_non_nullable
as List<String>,activeTab: null == activeTab ? _self.activeTab : activeTab // ignore: cast_nullable_to_non_nullable
as int,searchImageText: null == searchImageText ? _self.searchImageText : searchImageText // ignore: cast_nullable_to_non_nullable
as bool,remindersEnabled: null == remindersEnabled ? _self.remindersEnabled : remindersEnabled // ignore: cast_nullable_to_non_nullable
as bool,focusMode: null == focusMode ? _self.focusMode : focusMode // ignore: cast_nullable_to_non_nullable
as bool,livePreviewInFocus: null == livePreviewInFocus ? _self.livePreviewInFocus : livePreviewInFocus // ignore: cast_nullable_to_non_nullable
as bool,resurface: null == resurface ? _self.resurface : resurface // ignore: cast_nullable_to_non_nullable
as bool,resurfaceCloseness: null == resurfaceCloseness ? _self.resurfaceCloseness : resurfaceCloseness // ignore: cast_nullable_to_non_nullable
as String,onThisDay: null == onThisDay ? _self.onThisDay : onThisDay // ignore: cast_nullable_to_non_nullable
as bool,reviewPrompt: null == reviewPrompt ? _self.reviewPrompt : reviewPrompt // ignore: cast_nullable_to_non_nullable
as ReviewPromptRecord,captureAsTask: null == captureAsTask ? _self.captureAsTask : captureAsTask // ignore: cast_nullable_to_non_nullable
as bool,captureTaskTarget: null == captureTaskTarget ? _self.captureTaskTarget : captureTaskTarget // ignore: cast_nullable_to_non_nullable
as CaptureTarget,pdfOriginalPages: null == pdfOriginalPages ? _self.pdfOriginalPages : pdfOriginalPages // ignore: cast_nullable_to_non_nullable
as Map<String, bool>,
  ));
}
/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ThemeSelectionCopyWith<$Res> get themeSelection {
  
  return $ThemeSelectionCopyWith<$Res>(_self.themeSelection, (value) {
    return _then(_self.copyWith(themeSelection: value));
  });
}/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoFontsCopyWith<$Res> get fonts {
  
  return $KansoFontsCopyWith<$Res>(_self.fonts, (value) {
    return _then(_self.copyWith(fonts: value));
  });
}/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ReviewPromptRecordCopyWith<$Res> get reviewPrompt {
  
  return $ReviewPromptRecordCopyWith<$Res>(_self.reviewPrompt, (value) {
    return _then(_self.copyWith(reviewPrompt: value));
  });
}
}


/// Adds pattern-matching-related methods to [Preferences].
extension PreferencesPatterns on Preferences {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _Preferences value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _Preferences() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _Preferences value)  $default,){
final _that = this;
switch (_that) {
case _Preferences():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _Preferences value)?  $default,){
final _that = this;
switch (_that) {
case _Preferences() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( ThemeSelection themeSelection,  KansoFonts fonts, @JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0)  double textSize, @JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0)  double previewTextSize, @JsonKey(fromJson: interfaceTextScaleFromJson, toJson: interfaceTextScaleToJson, defaultValue: 1.0)  double interfaceTextSize,  String iconTheme, @JsonKey(unknownEnumValue: DiagramRenderer.pele)  DiagramRenderer diagramRenderer,  bool alwaysShowScrollbars, @JsonKey(unknownEnumValue: SortKey.updated)  SortKey sort,  bool descending,  bool backupNoticeSeen,  bool biometricOfferSeen,  String? defaultNotebookId,  List<String> openTabs,  int activeTab,  bool searchImageText,  bool remindersEnabled,  bool focusMode,  bool livePreviewInFocus,  bool resurface,  String resurfaceCloseness,  bool onThisDay,  ReviewPromptRecord reviewPrompt,  bool captureAsTask, @JsonKey(unknownEnumValue: CaptureTarget.dailyNote)  CaptureTarget captureTaskTarget,  Map<String, bool> pdfOriginalPages)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _Preferences() when $default != null:
return $default(_that.themeSelection,_that.fonts,_that.textSize,_that.previewTextSize,_that.interfaceTextSize,_that.iconTheme,_that.diagramRenderer,_that.alwaysShowScrollbars,_that.sort,_that.descending,_that.backupNoticeSeen,_that.biometricOfferSeen,_that.defaultNotebookId,_that.openTabs,_that.activeTab,_that.searchImageText,_that.remindersEnabled,_that.focusMode,_that.livePreviewInFocus,_that.resurface,_that.resurfaceCloseness,_that.onThisDay,_that.reviewPrompt,_that.captureAsTask,_that.captureTaskTarget,_that.pdfOriginalPages);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( ThemeSelection themeSelection,  KansoFonts fonts, @JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0)  double textSize, @JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0)  double previewTextSize, @JsonKey(fromJson: interfaceTextScaleFromJson, toJson: interfaceTextScaleToJson, defaultValue: 1.0)  double interfaceTextSize,  String iconTheme, @JsonKey(unknownEnumValue: DiagramRenderer.pele)  DiagramRenderer diagramRenderer,  bool alwaysShowScrollbars, @JsonKey(unknownEnumValue: SortKey.updated)  SortKey sort,  bool descending,  bool backupNoticeSeen,  bool biometricOfferSeen,  String? defaultNotebookId,  List<String> openTabs,  int activeTab,  bool searchImageText,  bool remindersEnabled,  bool focusMode,  bool livePreviewInFocus,  bool resurface,  String resurfaceCloseness,  bool onThisDay,  ReviewPromptRecord reviewPrompt,  bool captureAsTask, @JsonKey(unknownEnumValue: CaptureTarget.dailyNote)  CaptureTarget captureTaskTarget,  Map<String, bool> pdfOriginalPages)  $default,) {final _that = this;
switch (_that) {
case _Preferences():
return $default(_that.themeSelection,_that.fonts,_that.textSize,_that.previewTextSize,_that.interfaceTextSize,_that.iconTheme,_that.diagramRenderer,_that.alwaysShowScrollbars,_that.sort,_that.descending,_that.backupNoticeSeen,_that.biometricOfferSeen,_that.defaultNotebookId,_that.openTabs,_that.activeTab,_that.searchImageText,_that.remindersEnabled,_that.focusMode,_that.livePreviewInFocus,_that.resurface,_that.resurfaceCloseness,_that.onThisDay,_that.reviewPrompt,_that.captureAsTask,_that.captureTaskTarget,_that.pdfOriginalPages);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( ThemeSelection themeSelection,  KansoFonts fonts, @JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0)  double textSize, @JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0)  double previewTextSize, @JsonKey(fromJson: interfaceTextScaleFromJson, toJson: interfaceTextScaleToJson, defaultValue: 1.0)  double interfaceTextSize,  String iconTheme, @JsonKey(unknownEnumValue: DiagramRenderer.pele)  DiagramRenderer diagramRenderer,  bool alwaysShowScrollbars, @JsonKey(unknownEnumValue: SortKey.updated)  SortKey sort,  bool descending,  bool backupNoticeSeen,  bool biometricOfferSeen,  String? defaultNotebookId,  List<String> openTabs,  int activeTab,  bool searchImageText,  bool remindersEnabled,  bool focusMode,  bool livePreviewInFocus,  bool resurface,  String resurfaceCloseness,  bool onThisDay,  ReviewPromptRecord reviewPrompt,  bool captureAsTask, @JsonKey(unknownEnumValue: CaptureTarget.dailyNote)  CaptureTarget captureTaskTarget,  Map<String, bool> pdfOriginalPages)?  $default,) {final _that = this;
switch (_that) {
case _Preferences() when $default != null:
return $default(_that.themeSelection,_that.fonts,_that.textSize,_that.previewTextSize,_that.interfaceTextSize,_that.iconTheme,_that.diagramRenderer,_that.alwaysShowScrollbars,_that.sort,_that.descending,_that.backupNoticeSeen,_that.biometricOfferSeen,_that.defaultNotebookId,_that.openTabs,_that.activeTab,_that.searchImageText,_that.remindersEnabled,_that.focusMode,_that.livePreviewInFocus,_that.resurface,_that.resurfaceCloseness,_that.onThisDay,_that.reviewPrompt,_that.captureAsTask,_that.captureTaskTarget,_that.pdfOriginalPages);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _Preferences implements Preferences {
  const _Preferences({this.themeSelection = const ThemeSelection(), this.fonts = const KansoFonts(), @JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0) this.textSize = 1.0, @JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0) this.previewTextSize = 1.0, @JsonKey(fromJson: interfaceTextScaleFromJson, toJson: interfaceTextScaleToJson, defaultValue: 1.0) this.interfaceTextSize = 1.0, this.iconTheme = KansoIconSet.lucideId, @JsonKey(unknownEnumValue: DiagramRenderer.pele) this.diagramRenderer = DiagramRenderer.pele, this.alwaysShowScrollbars = false, @JsonKey(unknownEnumValue: SortKey.updated) this.sort = SortKey.updated, this.descending = true, this.backupNoticeSeen = false, this.biometricOfferSeen = false, this.defaultNotebookId, final  List<String> openTabs = const <String>[], this.activeTab = 0, this.searchImageText = false, this.remindersEnabled = true, this.focusMode = false, this.livePreviewInFocus = false, this.resurface = true, this.resurfaceCloseness = 'default', this.onThisDay = true, this.reviewPrompt = const ReviewPromptRecord(), this.captureAsTask = false, @JsonKey(unknownEnumValue: CaptureTarget.dailyNote) this.captureTaskTarget = CaptureTarget.dailyNote, final  Map<String, bool> pdfOriginalPages = const <String, bool>{}}): _openTabs = openTabs,_pdfOriginalPages = pdfOriginalPages;
  factory _Preferences.fromJson(Map<String, dynamic> json) => _$PreferencesFromJson(json);

/// The themes the interface is drawn in.
@override@JsonKey() final  ThemeSelection themeSelection;
/// The fonts of notes.
@override@JsonKey() final  KansoFonts fonts;
/// The size of the text of notes in the editor, from 0.8 to 1.8.
@override@JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0) final  double textSize;
/// The size of the rendered preview's text, from 0.8 to 1.8.
@override@JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0) final  double previewTextSize;
/// The size of the interface's labels and text, from 0.9 to 1.3.
@override@JsonKey(fromJson: interfaceTextScaleFromJson, toJson: interfaceTextScaleToJson, defaultValue: 1.0) final  double interfaceTextSize;
/// The icon theme the app is drawn in, by its ID. A theme this version
/// does not know is Kanso's own icons.
@override@JsonKey() final  String iconTheme;
/// What draws Mermaid diagrams.
@override@JsonKey(unknownEnumValue: DiagramRenderer.pele) final  DiagramRenderer diagramRenderer;
/// Whether overflowing list scrollbars stay visible between scrolls.
@override@JsonKey() final  bool alwaysShowScrollbars;
/// What note lists are sorted by.
@override@JsonKey(unknownEnumValue: SortKey.updated) final  SortKey sort;
/// Whether lists run from the largest to the smallest sort value (the
/// newest first, Z to A).
@override@JsonKey() final  bool descending;
/// Whether the notice that this phone's notes are not in device backups
/// has been shown.
@override@JsonKey() final  bool backupNoticeSeen;
/// Whether the offer to unlock locked notes with a biometric, made once
/// after the first password unlock, has been shown.
@override@JsonKey() final  bool biometricOfferSeen;
/// The notebook new notes go in, unless they are made from a notebook's
/// own list. Null, or one that is gone or locked, is the first notebook.
@override final  String? defaultNotebookId;
/// The notes open as tabs on a tablet, left to right (the desktop keeps
/// them in its settings too). Notes that are gone by the next start are
/// dropped then. Empty on a phone, which has no tabs.
 final  List<String> _openTabs;
/// The notes open as tabs on a tablet, left to right (the desktop keeps
/// them in its settings too). Notes that are gone by the next start are
/// dropped then. Empty on a phone, which has no tabs.
@override@JsonKey() List<String> get openTabs {
  if (_openTabs is EqualUnmodifiableListView) return _openTabs;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_openTabs);
}

/// The index in [openTabs] of the tab that was showing.
@override@JsonKey() final  int activeTab;
/// Whether pictures attached to notes are read on this device for search.
@override@JsonKey() final  bool searchImageText;
/// Whether this phone may show reminders for tasks with a due time.
/// Notification permission is asked separately by the operating system.
@override@JsonKey() final  bool remindersEnabled;
/// Whether the open note is shown by itself, without the library, the
/// list, tabs or the note's bars.
@override@JsonKey() final  bool focusMode;
/// Whether focus mode shows the open note in Live, and leaving it goes
/// back to the mode the note was in.
@override@JsonKey() final  bool livePreviewInFocus;
/// Whether the editor offers past notes about the paragraph being
/// written, in a pill above the keyboard (when related notes are on).
@override@JsonKey() final  bool resurface;
/// How close a past note must be to be offered: "closer", "default" or
/// "looser". A value this version does not know is "default".
@override@JsonKey() final  String resurfaceCloseness;
/// Whether a daily note lists the notes written on its date in earlier
/// years.
@override@JsonKey() final  bool onThisDay;
/// What decides whether to ask for a review of the app: when it was
/// first opened, how often it is used, and when it last asked.
@override@JsonKey() final  ReviewPromptRecord reviewPrompt;
/// Whether the capture sheet was last left in Task mode, so it opens
/// there again.
@override@JsonKey() final  bool captureAsTask;
/// Where a captured task is written (Settings › Captured tasks go to);
/// the same two places as the desktop's setting.
@override@JsonKey(unknownEnumValue: CaptureTarget.dailyNote) final  CaptureTarget captureTaskTarget;
/// The PDFs whose pages are shown as printed rather than smart-inverted
/// in a dark theme, keyed by attachment ID.
 final  Map<String, bool> _pdfOriginalPages;
/// The PDFs whose pages are shown as printed rather than smart-inverted
/// in a dark theme, keyed by attachment ID.
@override@JsonKey() Map<String, bool> get pdfOriginalPages {
  if (_pdfOriginalPages is EqualUnmodifiableMapView) return _pdfOriginalPages;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_pdfOriginalPages);
}


/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PreferencesCopyWith<_Preferences> get copyWith => __$PreferencesCopyWithImpl<_Preferences>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$PreferencesToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _Preferences&&(identical(other.themeSelection, themeSelection) || other.themeSelection == themeSelection)&&(identical(other.fonts, fonts) || other.fonts == fonts)&&(identical(other.textSize, textSize) || other.textSize == textSize)&&(identical(other.previewTextSize, previewTextSize) || other.previewTextSize == previewTextSize)&&(identical(other.interfaceTextSize, interfaceTextSize) || other.interfaceTextSize == interfaceTextSize)&&(identical(other.iconTheme, iconTheme) || other.iconTheme == iconTheme)&&(identical(other.diagramRenderer, diagramRenderer) || other.diagramRenderer == diagramRenderer)&&(identical(other.alwaysShowScrollbars, alwaysShowScrollbars) || other.alwaysShowScrollbars == alwaysShowScrollbars)&&(identical(other.sort, sort) || other.sort == sort)&&(identical(other.descending, descending) || other.descending == descending)&&(identical(other.backupNoticeSeen, backupNoticeSeen) || other.backupNoticeSeen == backupNoticeSeen)&&(identical(other.biometricOfferSeen, biometricOfferSeen) || other.biometricOfferSeen == biometricOfferSeen)&&(identical(other.defaultNotebookId, defaultNotebookId) || other.defaultNotebookId == defaultNotebookId)&&const DeepCollectionEquality().equals(other._openTabs, _openTabs)&&(identical(other.activeTab, activeTab) || other.activeTab == activeTab)&&(identical(other.searchImageText, searchImageText) || other.searchImageText == searchImageText)&&(identical(other.remindersEnabled, remindersEnabled) || other.remindersEnabled == remindersEnabled)&&(identical(other.focusMode, focusMode) || other.focusMode == focusMode)&&(identical(other.livePreviewInFocus, livePreviewInFocus) || other.livePreviewInFocus == livePreviewInFocus)&&(identical(other.resurface, resurface) || other.resurface == resurface)&&(identical(other.resurfaceCloseness, resurfaceCloseness) || other.resurfaceCloseness == resurfaceCloseness)&&(identical(other.onThisDay, onThisDay) || other.onThisDay == onThisDay)&&(identical(other.reviewPrompt, reviewPrompt) || other.reviewPrompt == reviewPrompt)&&(identical(other.captureAsTask, captureAsTask) || other.captureAsTask == captureAsTask)&&(identical(other.captureTaskTarget, captureTaskTarget) || other.captureTaskTarget == captureTaskTarget)&&const DeepCollectionEquality().equals(other._pdfOriginalPages, _pdfOriginalPages));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hashAll([runtimeType,themeSelection,fonts,textSize,previewTextSize,interfaceTextSize,iconTheme,diagramRenderer,alwaysShowScrollbars,sort,descending,backupNoticeSeen,biometricOfferSeen,defaultNotebookId,const DeepCollectionEquality().hash(_openTabs),activeTab,searchImageText,remindersEnabled,focusMode,livePreviewInFocus,resurface,resurfaceCloseness,onThisDay,reviewPrompt,captureAsTask,captureTaskTarget,const DeepCollectionEquality().hash(_pdfOriginalPages)]);

@override
String toString() {
  return 'Preferences(themeSelection: $themeSelection, fonts: $fonts, textSize: $textSize, previewTextSize: $previewTextSize, interfaceTextSize: $interfaceTextSize, iconTheme: $iconTheme, diagramRenderer: $diagramRenderer, alwaysShowScrollbars: $alwaysShowScrollbars, sort: $sort, descending: $descending, backupNoticeSeen: $backupNoticeSeen, biometricOfferSeen: $biometricOfferSeen, defaultNotebookId: $defaultNotebookId, openTabs: $openTabs, activeTab: $activeTab, searchImageText: $searchImageText, remindersEnabled: $remindersEnabled, focusMode: $focusMode, livePreviewInFocus: $livePreviewInFocus, resurface: $resurface, resurfaceCloseness: $resurfaceCloseness, onThisDay: $onThisDay, reviewPrompt: $reviewPrompt, captureAsTask: $captureAsTask, captureTaskTarget: $captureTaskTarget, pdfOriginalPages: $pdfOriginalPages)';
}


}

/// @nodoc
abstract mixin class _$PreferencesCopyWith<$Res> implements $PreferencesCopyWith<$Res> {
  factory _$PreferencesCopyWith(_Preferences value, $Res Function(_Preferences) _then) = __$PreferencesCopyWithImpl;
@override @useResult
$Res call({
 ThemeSelection themeSelection, KansoFonts fonts,@JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0) double textSize,@JsonKey(fromJson: textScaleFromJson, toJson: textScaleToJson, defaultValue: 1.0) double previewTextSize,@JsonKey(fromJson: interfaceTextScaleFromJson, toJson: interfaceTextScaleToJson, defaultValue: 1.0) double interfaceTextSize, String iconTheme,@JsonKey(unknownEnumValue: DiagramRenderer.pele) DiagramRenderer diagramRenderer, bool alwaysShowScrollbars,@JsonKey(unknownEnumValue: SortKey.updated) SortKey sort, bool descending, bool backupNoticeSeen, bool biometricOfferSeen, String? defaultNotebookId, List<String> openTabs, int activeTab, bool searchImageText, bool remindersEnabled, bool focusMode, bool livePreviewInFocus, bool resurface, String resurfaceCloseness, bool onThisDay, ReviewPromptRecord reviewPrompt, bool captureAsTask,@JsonKey(unknownEnumValue: CaptureTarget.dailyNote) CaptureTarget captureTaskTarget, Map<String, bool> pdfOriginalPages
});


@override $ThemeSelectionCopyWith<$Res> get themeSelection;@override $KansoFontsCopyWith<$Res> get fonts;@override $ReviewPromptRecordCopyWith<$Res> get reviewPrompt;

}
/// @nodoc
class __$PreferencesCopyWithImpl<$Res>
    implements _$PreferencesCopyWith<$Res> {
  __$PreferencesCopyWithImpl(this._self, this._then);

  final _Preferences _self;
  final $Res Function(_Preferences) _then;

/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? themeSelection = null,Object? fonts = null,Object? textSize = null,Object? previewTextSize = null,Object? interfaceTextSize = null,Object? iconTheme = null,Object? diagramRenderer = null,Object? alwaysShowScrollbars = null,Object? sort = null,Object? descending = null,Object? backupNoticeSeen = null,Object? biometricOfferSeen = null,Object? defaultNotebookId = freezed,Object? openTabs = null,Object? activeTab = null,Object? searchImageText = null,Object? remindersEnabled = null,Object? focusMode = null,Object? livePreviewInFocus = null,Object? resurface = null,Object? resurfaceCloseness = null,Object? onThisDay = null,Object? reviewPrompt = null,Object? captureAsTask = null,Object? captureTaskTarget = null,Object? pdfOriginalPages = null,}) {
  return _then(_Preferences(
themeSelection: null == themeSelection ? _self.themeSelection : themeSelection // ignore: cast_nullable_to_non_nullable
as ThemeSelection,fonts: null == fonts ? _self.fonts : fonts // ignore: cast_nullable_to_non_nullable
as KansoFonts,textSize: null == textSize ? _self.textSize : textSize // ignore: cast_nullable_to_non_nullable
as double,previewTextSize: null == previewTextSize ? _self.previewTextSize : previewTextSize // ignore: cast_nullable_to_non_nullable
as double,interfaceTextSize: null == interfaceTextSize ? _self.interfaceTextSize : interfaceTextSize // ignore: cast_nullable_to_non_nullable
as double,iconTheme: null == iconTheme ? _self.iconTheme : iconTheme // ignore: cast_nullable_to_non_nullable
as String,diagramRenderer: null == diagramRenderer ? _self.diagramRenderer : diagramRenderer // ignore: cast_nullable_to_non_nullable
as DiagramRenderer,alwaysShowScrollbars: null == alwaysShowScrollbars ? _self.alwaysShowScrollbars : alwaysShowScrollbars // ignore: cast_nullable_to_non_nullable
as bool,sort: null == sort ? _self.sort : sort // ignore: cast_nullable_to_non_nullable
as SortKey,descending: null == descending ? _self.descending : descending // ignore: cast_nullable_to_non_nullable
as bool,backupNoticeSeen: null == backupNoticeSeen ? _self.backupNoticeSeen : backupNoticeSeen // ignore: cast_nullable_to_non_nullable
as bool,biometricOfferSeen: null == biometricOfferSeen ? _self.biometricOfferSeen : biometricOfferSeen // ignore: cast_nullable_to_non_nullable
as bool,defaultNotebookId: freezed == defaultNotebookId ? _self.defaultNotebookId : defaultNotebookId // ignore: cast_nullable_to_non_nullable
as String?,openTabs: null == openTabs ? _self._openTabs : openTabs // ignore: cast_nullable_to_non_nullable
as List<String>,activeTab: null == activeTab ? _self.activeTab : activeTab // ignore: cast_nullable_to_non_nullable
as int,searchImageText: null == searchImageText ? _self.searchImageText : searchImageText // ignore: cast_nullable_to_non_nullable
as bool,remindersEnabled: null == remindersEnabled ? _self.remindersEnabled : remindersEnabled // ignore: cast_nullable_to_non_nullable
as bool,focusMode: null == focusMode ? _self.focusMode : focusMode // ignore: cast_nullable_to_non_nullable
as bool,livePreviewInFocus: null == livePreviewInFocus ? _self.livePreviewInFocus : livePreviewInFocus // ignore: cast_nullable_to_non_nullable
as bool,resurface: null == resurface ? _self.resurface : resurface // ignore: cast_nullable_to_non_nullable
as bool,resurfaceCloseness: null == resurfaceCloseness ? _self.resurfaceCloseness : resurfaceCloseness // ignore: cast_nullable_to_non_nullable
as String,onThisDay: null == onThisDay ? _self.onThisDay : onThisDay // ignore: cast_nullable_to_non_nullable
as bool,reviewPrompt: null == reviewPrompt ? _self.reviewPrompt : reviewPrompt // ignore: cast_nullable_to_non_nullable
as ReviewPromptRecord,captureAsTask: null == captureAsTask ? _self.captureAsTask : captureAsTask // ignore: cast_nullable_to_non_nullable
as bool,captureTaskTarget: null == captureTaskTarget ? _self.captureTaskTarget : captureTaskTarget // ignore: cast_nullable_to_non_nullable
as CaptureTarget,pdfOriginalPages: null == pdfOriginalPages ? _self._pdfOriginalPages : pdfOriginalPages // ignore: cast_nullable_to_non_nullable
as Map<String, bool>,
  ));
}

/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ThemeSelectionCopyWith<$Res> get themeSelection {
  
  return $ThemeSelectionCopyWith<$Res>(_self.themeSelection, (value) {
    return _then(_self.copyWith(themeSelection: value));
  });
}/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoFontsCopyWith<$Res> get fonts {
  
  return $KansoFontsCopyWith<$Res>(_self.fonts, (value) {
    return _then(_self.copyWith(fonts: value));
  });
}/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ReviewPromptRecordCopyWith<$Res> get reviewPrompt {
  
  return $ReviewPromptRecordCopyWith<$Res>(_self.reviewPrompt, (value) {
    return _then(_self.copyWith(reviewPrompt: value));
  });
}
}

// dart format on
