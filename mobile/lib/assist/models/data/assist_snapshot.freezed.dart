// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'assist_snapshot.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AssistSnapshot {

/// The note the editor was showing.
 String get noteId;/// Changes on every edit, reload, or selection update.
 int get revision;/// The current note-window origin, in UTF-16 units.
 int get windowOrigin;/// The editor's whole text then.
 String get text;/// The selection then; a caret when nothing was selected.
 TextRange get selection;
/// Create a copy of AssistSnapshot
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistSnapshotCopyWith<AssistSnapshot> get copyWith => _$AssistSnapshotCopyWithImpl<AssistSnapshot>(this as AssistSnapshot, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistSnapshot&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.revision, revision) || other.revision == revision)&&(identical(other.windowOrigin, windowOrigin) || other.windowOrigin == windowOrigin)&&(identical(other.text, text) || other.text == text)&&(identical(other.selection, selection) || other.selection == selection));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,revision,windowOrigin,text,selection);

@override
String toString() {
  return 'AssistSnapshot(noteId: $noteId, revision: $revision, windowOrigin: $windowOrigin, text: $text, selection: $selection)';
}


}

/// @nodoc
abstract mixin class $AssistSnapshotCopyWith<$Res>  {
  factory $AssistSnapshotCopyWith(AssistSnapshot value, $Res Function(AssistSnapshot) _then) = _$AssistSnapshotCopyWithImpl;
@useResult
$Res call({
 String noteId, int revision, int windowOrigin, String text, TextRange selection
});




}
/// @nodoc
class _$AssistSnapshotCopyWithImpl<$Res>
    implements $AssistSnapshotCopyWith<$Res> {
  _$AssistSnapshotCopyWithImpl(this._self, this._then);

  final AssistSnapshot _self;
  final $Res Function(AssistSnapshot) _then;

/// Create a copy of AssistSnapshot
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? revision = null,Object? windowOrigin = null,Object? text = null,Object? selection = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,revision: null == revision ? _self.revision : revision // ignore: cast_nullable_to_non_nullable
as int,windowOrigin: null == windowOrigin ? _self.windowOrigin : windowOrigin // ignore: cast_nullable_to_non_nullable
as int,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,selection: null == selection ? _self.selection : selection // ignore: cast_nullable_to_non_nullable
as TextRange,
  ));
}

}


/// Adds pattern-matching-related methods to [AssistSnapshot].
extension AssistSnapshotPatterns on AssistSnapshot {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AssistSnapshot value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AssistSnapshot() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AssistSnapshot value)  $default,){
final _that = this;
switch (_that) {
case _AssistSnapshot():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AssistSnapshot value)?  $default,){
final _that = this;
switch (_that) {
case _AssistSnapshot() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  int revision,  int windowOrigin,  String text,  TextRange selection)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AssistSnapshot() when $default != null:
return $default(_that.noteId,_that.revision,_that.windowOrigin,_that.text,_that.selection);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  int revision,  int windowOrigin,  String text,  TextRange selection)  $default,) {final _that = this;
switch (_that) {
case _AssistSnapshot():
return $default(_that.noteId,_that.revision,_that.windowOrigin,_that.text,_that.selection);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  int revision,  int windowOrigin,  String text,  TextRange selection)?  $default,) {final _that = this;
switch (_that) {
case _AssistSnapshot() when $default != null:
return $default(_that.noteId,_that.revision,_that.windowOrigin,_that.text,_that.selection);case _:
  return null;

}
}

}

/// @nodoc


class _AssistSnapshot extends AssistSnapshot {
  const _AssistSnapshot({required this.noteId, required this.revision, required this.windowOrigin, required this.text, required this.selection}): super._();
  

/// The note the editor was showing.
@override final  String noteId;
/// Changes on every edit, reload, or selection update.
@override final  int revision;
/// The current note-window origin, in UTF-16 units.
@override final  int windowOrigin;
/// The editor's whole text then.
@override final  String text;
/// The selection then; a caret when nothing was selected.
@override final  TextRange selection;

/// Create a copy of AssistSnapshot
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AssistSnapshotCopyWith<_AssistSnapshot> get copyWith => __$AssistSnapshotCopyWithImpl<_AssistSnapshot>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AssistSnapshot&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.revision, revision) || other.revision == revision)&&(identical(other.windowOrigin, windowOrigin) || other.windowOrigin == windowOrigin)&&(identical(other.text, text) || other.text == text)&&(identical(other.selection, selection) || other.selection == selection));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,revision,windowOrigin,text,selection);

@override
String toString() {
  return 'AssistSnapshot(noteId: $noteId, revision: $revision, windowOrigin: $windowOrigin, text: $text, selection: $selection)';
}


}

/// @nodoc
abstract mixin class _$AssistSnapshotCopyWith<$Res> implements $AssistSnapshotCopyWith<$Res> {
  factory _$AssistSnapshotCopyWith(_AssistSnapshot value, $Res Function(_AssistSnapshot) _then) = __$AssistSnapshotCopyWithImpl;
@override @useResult
$Res call({
 String noteId, int revision, int windowOrigin, String text, TextRange selection
});




}
/// @nodoc
class __$AssistSnapshotCopyWithImpl<$Res>
    implements _$AssistSnapshotCopyWith<$Res> {
  __$AssistSnapshotCopyWithImpl(this._self, this._then);

  final _AssistSnapshot _self;
  final $Res Function(_AssistSnapshot) _then;

/// Create a copy of AssistSnapshot
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? revision = null,Object? windowOrigin = null,Object? text = null,Object? selection = null,}) {
  return _then(_AssistSnapshot(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,revision: null == revision ? _self.revision : revision // ignore: cast_nullable_to_non_nullable
as int,windowOrigin: null == windowOrigin ? _self.windowOrigin : windowOrigin // ignore: cast_nullable_to_non_nullable
as int,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,selection: null == selection ? _self.selection : selection // ignore: cast_nullable_to_non_nullable
as TextRange,
  ));
}


}

// dart format on
