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
 KansoFonts get fonts;/// The size of the text of notes.
@JsonKey(unknownEnumValue: KansoTextSize.standard) KansoTextSize get textSize;/// The icon theme the app is drawn in, by its ID. A theme this version
/// does not know is Kanso's own icons.
 String get iconTheme;/// What note lists are sorted by.
@JsonKey(unknownEnumValue: SortKey.updated) SortKey get sort;/// Whether lists run from the largest to the smallest sort value (the
/// newest first, Z to A).
 bool get descending;/// Whether the notice that this phone's notes are not in device backups
/// has been shown.
 bool get backupNoticeSeen;/// Whether the offer to unlock locked notes with a biometric, made once
/// after the first password unlock, has been shown.
 bool get biometricOfferSeen;/// The notebook new notes go in, unless they are made from a notebook's
/// own list. Null, or one that is gone or locked, is the first notebook.
 String? get defaultNotebookId;
/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreferencesCopyWith<Preferences> get copyWith => _$PreferencesCopyWithImpl<Preferences>(this as Preferences, _$identity);

  /// Serializes this Preferences to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Preferences&&(identical(other.themeSelection, themeSelection) || other.themeSelection == themeSelection)&&(identical(other.fonts, fonts) || other.fonts == fonts)&&(identical(other.textSize, textSize) || other.textSize == textSize)&&(identical(other.iconTheme, iconTheme) || other.iconTheme == iconTheme)&&(identical(other.sort, sort) || other.sort == sort)&&(identical(other.descending, descending) || other.descending == descending)&&(identical(other.backupNoticeSeen, backupNoticeSeen) || other.backupNoticeSeen == backupNoticeSeen)&&(identical(other.biometricOfferSeen, biometricOfferSeen) || other.biometricOfferSeen == biometricOfferSeen)&&(identical(other.defaultNotebookId, defaultNotebookId) || other.defaultNotebookId == defaultNotebookId));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,themeSelection,fonts,textSize,iconTheme,sort,descending,backupNoticeSeen,biometricOfferSeen,defaultNotebookId);

@override
String toString() {
  return 'Preferences(themeSelection: $themeSelection, fonts: $fonts, textSize: $textSize, iconTheme: $iconTheme, sort: $sort, descending: $descending, backupNoticeSeen: $backupNoticeSeen, biometricOfferSeen: $biometricOfferSeen, defaultNotebookId: $defaultNotebookId)';
}


}

/// @nodoc
abstract mixin class $PreferencesCopyWith<$Res>  {
  factory $PreferencesCopyWith(Preferences value, $Res Function(Preferences) _then) = _$PreferencesCopyWithImpl;
@useResult
$Res call({
 ThemeSelection themeSelection, KansoFonts fonts,@JsonKey(unknownEnumValue: KansoTextSize.standard) KansoTextSize textSize, String iconTheme,@JsonKey(unknownEnumValue: SortKey.updated) SortKey sort, bool descending, bool backupNoticeSeen, bool biometricOfferSeen, String? defaultNotebookId
});


$ThemeSelectionCopyWith<$Res> get themeSelection;$KansoFontsCopyWith<$Res> get fonts;

}
/// @nodoc
class _$PreferencesCopyWithImpl<$Res>
    implements $PreferencesCopyWith<$Res> {
  _$PreferencesCopyWithImpl(this._self, this._then);

  final Preferences _self;
  final $Res Function(Preferences) _then;

/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? themeSelection = null,Object? fonts = null,Object? textSize = null,Object? iconTheme = null,Object? sort = null,Object? descending = null,Object? backupNoticeSeen = null,Object? biometricOfferSeen = null,Object? defaultNotebookId = freezed,}) {
  return _then(_self.copyWith(
themeSelection: null == themeSelection ? _self.themeSelection : themeSelection // ignore: cast_nullable_to_non_nullable
as ThemeSelection,fonts: null == fonts ? _self.fonts : fonts // ignore: cast_nullable_to_non_nullable
as KansoFonts,textSize: null == textSize ? _self.textSize : textSize // ignore: cast_nullable_to_non_nullable
as KansoTextSize,iconTheme: null == iconTheme ? _self.iconTheme : iconTheme // ignore: cast_nullable_to_non_nullable
as String,sort: null == sort ? _self.sort : sort // ignore: cast_nullable_to_non_nullable
as SortKey,descending: null == descending ? _self.descending : descending // ignore: cast_nullable_to_non_nullable
as bool,backupNoticeSeen: null == backupNoticeSeen ? _self.backupNoticeSeen : backupNoticeSeen // ignore: cast_nullable_to_non_nullable
as bool,biometricOfferSeen: null == biometricOfferSeen ? _self.biometricOfferSeen : biometricOfferSeen // ignore: cast_nullable_to_non_nullable
as bool,defaultNotebookId: freezed == defaultNotebookId ? _self.defaultNotebookId : defaultNotebookId // ignore: cast_nullable_to_non_nullable
as String?,
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( ThemeSelection themeSelection,  KansoFonts fonts, @JsonKey(unknownEnumValue: KansoTextSize.standard)  KansoTextSize textSize,  String iconTheme, @JsonKey(unknownEnumValue: SortKey.updated)  SortKey sort,  bool descending,  bool backupNoticeSeen,  bool biometricOfferSeen,  String? defaultNotebookId)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _Preferences() when $default != null:
return $default(_that.themeSelection,_that.fonts,_that.textSize,_that.iconTheme,_that.sort,_that.descending,_that.backupNoticeSeen,_that.biometricOfferSeen,_that.defaultNotebookId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( ThemeSelection themeSelection,  KansoFonts fonts, @JsonKey(unknownEnumValue: KansoTextSize.standard)  KansoTextSize textSize,  String iconTheme, @JsonKey(unknownEnumValue: SortKey.updated)  SortKey sort,  bool descending,  bool backupNoticeSeen,  bool biometricOfferSeen,  String? defaultNotebookId)  $default,) {final _that = this;
switch (_that) {
case _Preferences():
return $default(_that.themeSelection,_that.fonts,_that.textSize,_that.iconTheme,_that.sort,_that.descending,_that.backupNoticeSeen,_that.biometricOfferSeen,_that.defaultNotebookId);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( ThemeSelection themeSelection,  KansoFonts fonts, @JsonKey(unknownEnumValue: KansoTextSize.standard)  KansoTextSize textSize,  String iconTheme, @JsonKey(unknownEnumValue: SortKey.updated)  SortKey sort,  bool descending,  bool backupNoticeSeen,  bool biometricOfferSeen,  String? defaultNotebookId)?  $default,) {final _that = this;
switch (_that) {
case _Preferences() when $default != null:
return $default(_that.themeSelection,_that.fonts,_that.textSize,_that.iconTheme,_that.sort,_that.descending,_that.backupNoticeSeen,_that.biometricOfferSeen,_that.defaultNotebookId);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _Preferences implements Preferences {
  const _Preferences({this.themeSelection = const ThemeSelection(), this.fonts = const KansoFonts(), @JsonKey(unknownEnumValue: KansoTextSize.standard) this.textSize = KansoTextSize.standard, this.iconTheme = KansoIconSet.lucideId, @JsonKey(unknownEnumValue: SortKey.updated) this.sort = SortKey.updated, this.descending = true, this.backupNoticeSeen = false, this.biometricOfferSeen = false, this.defaultNotebookId});
  factory _Preferences.fromJson(Map<String, dynamic> json) => _$PreferencesFromJson(json);

/// The themes the interface is drawn in.
@override@JsonKey() final  ThemeSelection themeSelection;
/// The fonts of notes.
@override@JsonKey() final  KansoFonts fonts;
/// The size of the text of notes in the editor.
@override@JsonKey(unknownEnumValue: KansoTextSize.standard) final  KansoTextSize textSize;
/// The icon theme the app is drawn in, by its ID. A theme this version
/// does not know is Kanso's own icons.
@override@JsonKey() final  String iconTheme;
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
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _Preferences&&(identical(other.themeSelection, themeSelection) || other.themeSelection == themeSelection)&&(identical(other.fonts, fonts) || other.fonts == fonts)&&(identical(other.textSize, textSize) || other.textSize == textSize)&&(identical(other.iconTheme, iconTheme) || other.iconTheme == iconTheme)&&(identical(other.sort, sort) || other.sort == sort)&&(identical(other.descending, descending) || other.descending == descending)&&(identical(other.backupNoticeSeen, backupNoticeSeen) || other.backupNoticeSeen == backupNoticeSeen)&&(identical(other.biometricOfferSeen, biometricOfferSeen) || other.biometricOfferSeen == biometricOfferSeen)&&(identical(other.defaultNotebookId, defaultNotebookId) || other.defaultNotebookId == defaultNotebookId));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,themeSelection,fonts,textSize,iconTheme,sort,descending,backupNoticeSeen,biometricOfferSeen,defaultNotebookId);

@override
String toString() {
  return 'Preferences(themeSelection: $themeSelection, fonts: $fonts, textSize: $textSize, iconTheme: $iconTheme, sort: $sort, descending: $descending, backupNoticeSeen: $backupNoticeSeen, biometricOfferSeen: $biometricOfferSeen, defaultNotebookId: $defaultNotebookId)';
}


}

/// @nodoc
abstract mixin class _$PreferencesCopyWith<$Res> implements $PreferencesCopyWith<$Res> {
  factory _$PreferencesCopyWith(_Preferences value, $Res Function(_Preferences) _then) = __$PreferencesCopyWithImpl;
@override @useResult
$Res call({
 ThemeSelection themeSelection, KansoFonts fonts,@JsonKey(unknownEnumValue: KansoTextSize.standard) KansoTextSize textSize, String iconTheme,@JsonKey(unknownEnumValue: SortKey.updated) SortKey sort, bool descending, bool backupNoticeSeen, bool biometricOfferSeen, String? defaultNotebookId
});


@override $ThemeSelectionCopyWith<$Res> get themeSelection;@override $KansoFontsCopyWith<$Res> get fonts;

}
/// @nodoc
class __$PreferencesCopyWithImpl<$Res>
    implements _$PreferencesCopyWith<$Res> {
  __$PreferencesCopyWithImpl(this._self, this._then);

  final _Preferences _self;
  final $Res Function(_Preferences) _then;

/// Create a copy of Preferences
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? themeSelection = null,Object? fonts = null,Object? textSize = null,Object? iconTheme = null,Object? sort = null,Object? descending = null,Object? backupNoticeSeen = null,Object? biometricOfferSeen = null,Object? defaultNotebookId = freezed,}) {
  return _then(_Preferences(
themeSelection: null == themeSelection ? _self.themeSelection : themeSelection // ignore: cast_nullable_to_non_nullable
as ThemeSelection,fonts: null == fonts ? _self.fonts : fonts // ignore: cast_nullable_to_non_nullable
as KansoFonts,textSize: null == textSize ? _self.textSize : textSize // ignore: cast_nullable_to_non_nullable
as KansoTextSize,iconTheme: null == iconTheme ? _self.iconTheme : iconTheme // ignore: cast_nullable_to_non_nullable
as String,sort: null == sort ? _self.sort : sort // ignore: cast_nullable_to_non_nullable
as SortKey,descending: null == descending ? _self.descending : descending // ignore: cast_nullable_to_non_nullable
as bool,backupNoticeSeen: null == backupNoticeSeen ? _self.backupNoticeSeen : backupNoticeSeen // ignore: cast_nullable_to_non_nullable
as bool,biometricOfferSeen: null == biometricOfferSeen ? _self.biometricOfferSeen : biometricOfferSeen // ignore: cast_nullable_to_non_nullable
as bool,defaultNotebookId: freezed == defaultNotebookId ? _self.defaultNotebookId : defaultNotebookId // ignore: cast_nullable_to_non_nullable
as String?,
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
}
}

// dart format on
