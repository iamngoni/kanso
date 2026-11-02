// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'triage_card_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$TriageChipState {

 TriageChip get chip; ChipMark get mark;/// Whether the person changed the value.
 bool get edited;
/// Create a copy of TriageChipState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageChipStateCopyWith<TriageChipState> get copyWith => _$TriageChipStateCopyWithImpl<TriageChipState>(this as TriageChipState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageChipState&&(identical(other.chip, chip) || other.chip == chip)&&(identical(other.mark, mark) || other.mark == mark)&&(identical(other.edited, edited) || other.edited == edited));
}


@override
int get hashCode => Object.hash(runtimeType,chip,mark,edited);

@override
String toString() {
  return 'TriageChipState(chip: $chip, mark: $mark, edited: $edited)';
}


}

/// @nodoc
abstract mixin class $TriageChipStateCopyWith<$Res>  {
  factory $TriageChipStateCopyWith(TriageChipState value, $Res Function(TriageChipState) _then) = _$TriageChipStateCopyWithImpl;
@useResult
$Res call({
 TriageChip chip, ChipMark mark, bool edited
});




}
/// @nodoc
class _$TriageChipStateCopyWithImpl<$Res>
    implements $TriageChipStateCopyWith<$Res> {
  _$TriageChipStateCopyWithImpl(this._self, this._then);

  final TriageChipState _self;
  final $Res Function(TriageChipState) _then;

/// Create a copy of TriageChipState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? chip = null,Object? mark = null,Object? edited = null,}) {
  return _then(_self.copyWith(
chip: null == chip ? _self.chip : chip // ignore: cast_nullable_to_non_nullable
as TriageChip,mark: null == mark ? _self.mark : mark // ignore: cast_nullable_to_non_nullable
as ChipMark,edited: null == edited ? _self.edited : edited // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [TriageChipState].
extension TriageChipStatePatterns on TriageChipState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TriageChipState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TriageChipState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TriageChipState value)  $default,){
final _that = this;
switch (_that) {
case _TriageChipState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TriageChipState value)?  $default,){
final _that = this;
switch (_that) {
case _TriageChipState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( TriageChip chip,  ChipMark mark,  bool edited)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TriageChipState() when $default != null:
return $default(_that.chip,_that.mark,_that.edited);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( TriageChip chip,  ChipMark mark,  bool edited)  $default,) {final _that = this;
switch (_that) {
case _TriageChipState():
return $default(_that.chip,_that.mark,_that.edited);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( TriageChip chip,  ChipMark mark,  bool edited)?  $default,) {final _that = this;
switch (_that) {
case _TriageChipState() when $default != null:
return $default(_that.chip,_that.mark,_that.edited);case _:
  return null;

}
}

}

/// @nodoc


class _TriageChipState implements TriageChipState {
  const _TriageChipState({required this.chip, this.mark = ChipMark.untouched, this.edited = false});
  

@override final  TriageChip chip;
@override@JsonKey() final  ChipMark mark;
/// Whether the person changed the value.
@override@JsonKey() final  bool edited;

/// Create a copy of TriageChipState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TriageChipStateCopyWith<_TriageChipState> get copyWith => __$TriageChipStateCopyWithImpl<_TriageChipState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TriageChipState&&(identical(other.chip, chip) || other.chip == chip)&&(identical(other.mark, mark) || other.mark == mark)&&(identical(other.edited, edited) || other.edited == edited));
}


@override
int get hashCode => Object.hash(runtimeType,chip,mark,edited);

@override
String toString() {
  return 'TriageChipState(chip: $chip, mark: $mark, edited: $edited)';
}


}

/// @nodoc
abstract mixin class _$TriageChipStateCopyWith<$Res> implements $TriageChipStateCopyWith<$Res> {
  factory _$TriageChipStateCopyWith(_TriageChipState value, $Res Function(_TriageChipState) _then) = __$TriageChipStateCopyWithImpl;
@override @useResult
$Res call({
 TriageChip chip, ChipMark mark, bool edited
});




}
/// @nodoc
class __$TriageChipStateCopyWithImpl<$Res>
    implements _$TriageChipStateCopyWith<$Res> {
  __$TriageChipStateCopyWithImpl(this._self, this._then);

  final _TriageChipState _self;
  final $Res Function(_TriageChipState) _then;

/// Create a copy of TriageChipState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? chip = null,Object? mark = null,Object? edited = null,}) {
  return _then(_TriageChipState(
chip: null == chip ? _self.chip : chip // ignore: cast_nullable_to_non_nullable
as TriageChip,mark: null == mark ? _self.mark : mark // ignore: cast_nullable_to_non_nullable
as ChipMark,edited: null == edited ? _self.edited : edited // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc
mixin _$TriageCardState {

 String get noteId;/// The revision the suggestions were made from.
 String get rev; String get title; String get snippet; int get updatedMs; List<TriageChipState> get chips; TriageMissing get missing;/// The note changed on another device: the card is read again and File
/// waits until it has been.
 bool get changed;/// Hidden for this visit.
 bool get skipped;/// Why the AI's suggestions are missing, when they are.
 String? get aiError;
/// Create a copy of TriageCardState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageCardStateCopyWith<TriageCardState> get copyWith => _$TriageCardStateCopyWithImpl<TriageCardState>(this as TriageCardState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageCardState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.title, title) || other.title == title)&&(identical(other.snippet, snippet) || other.snippet == snippet)&&(identical(other.updatedMs, updatedMs) || other.updatedMs == updatedMs)&&const DeepCollectionEquality().equals(other.chips, chips)&&(identical(other.missing, missing) || other.missing == missing)&&(identical(other.changed, changed) || other.changed == changed)&&(identical(other.skipped, skipped) || other.skipped == skipped)&&(identical(other.aiError, aiError) || other.aiError == aiError));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,rev,title,snippet,updatedMs,const DeepCollectionEquality().hash(chips),missing,changed,skipped,aiError);

@override
String toString() {
  return 'TriageCardState(noteId: $noteId, rev: $rev, title: $title, snippet: $snippet, updatedMs: $updatedMs, chips: $chips, missing: $missing, changed: $changed, skipped: $skipped, aiError: $aiError)';
}


}

/// @nodoc
abstract mixin class $TriageCardStateCopyWith<$Res>  {
  factory $TriageCardStateCopyWith(TriageCardState value, $Res Function(TriageCardState) _then) = _$TriageCardStateCopyWithImpl;
@useResult
$Res call({
 String noteId, String rev, String title, String snippet, int updatedMs, List<TriageChipState> chips, TriageMissing missing, bool changed, bool skipped, String? aiError
});




}
/// @nodoc
class _$TriageCardStateCopyWithImpl<$Res>
    implements $TriageCardStateCopyWith<$Res> {
  _$TriageCardStateCopyWithImpl(this._self, this._then);

  final TriageCardState _self;
  final $Res Function(TriageCardState) _then;

/// Create a copy of TriageCardState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? rev = null,Object? title = null,Object? snippet = null,Object? updatedMs = null,Object? chips = null,Object? missing = null,Object? changed = null,Object? skipped = null,Object? aiError = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,snippet: null == snippet ? _self.snippet : snippet // ignore: cast_nullable_to_non_nullable
as String,updatedMs: null == updatedMs ? _self.updatedMs : updatedMs // ignore: cast_nullable_to_non_nullable
as int,chips: null == chips ? _self.chips : chips // ignore: cast_nullable_to_non_nullable
as List<TriageChipState>,missing: null == missing ? _self.missing : missing // ignore: cast_nullable_to_non_nullable
as TriageMissing,changed: null == changed ? _self.changed : changed // ignore: cast_nullable_to_non_nullable
as bool,skipped: null == skipped ? _self.skipped : skipped // ignore: cast_nullable_to_non_nullable
as bool,aiError: freezed == aiError ? _self.aiError : aiError // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [TriageCardState].
extension TriageCardStatePatterns on TriageCardState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TriageCardState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TriageCardState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TriageCardState value)  $default,){
final _that = this;
switch (_that) {
case _TriageCardState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TriageCardState value)?  $default,){
final _that = this;
switch (_that) {
case _TriageCardState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  String rev,  String title,  String snippet,  int updatedMs,  List<TriageChipState> chips,  TriageMissing missing,  bool changed,  bool skipped,  String? aiError)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TriageCardState() when $default != null:
return $default(_that.noteId,_that.rev,_that.title,_that.snippet,_that.updatedMs,_that.chips,_that.missing,_that.changed,_that.skipped,_that.aiError);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  String rev,  String title,  String snippet,  int updatedMs,  List<TriageChipState> chips,  TriageMissing missing,  bool changed,  bool skipped,  String? aiError)  $default,) {final _that = this;
switch (_that) {
case _TriageCardState():
return $default(_that.noteId,_that.rev,_that.title,_that.snippet,_that.updatedMs,_that.chips,_that.missing,_that.changed,_that.skipped,_that.aiError);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  String rev,  String title,  String snippet,  int updatedMs,  List<TriageChipState> chips,  TriageMissing missing,  bool changed,  bool skipped,  String? aiError)?  $default,) {final _that = this;
switch (_that) {
case _TriageCardState() when $default != null:
return $default(_that.noteId,_that.rev,_that.title,_that.snippet,_that.updatedMs,_that.chips,_that.missing,_that.changed,_that.skipped,_that.aiError);case _:
  return null;

}
}

}

/// @nodoc


class _TriageCardState extends TriageCardState {
  const _TriageCardState({required this.noteId, required this.rev, required this.title, required this.snippet, required this.updatedMs, required final  List<TriageChipState> chips, this.missing = TriageMissing.nothing, this.changed = false, this.skipped = false, this.aiError}): _chips = chips,super._();
  

@override final  String noteId;
/// The revision the suggestions were made from.
@override final  String rev;
@override final  String title;
@override final  String snippet;
@override final  int updatedMs;
 final  List<TriageChipState> _chips;
@override List<TriageChipState> get chips {
  if (_chips is EqualUnmodifiableListView) return _chips;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_chips);
}

@override@JsonKey() final  TriageMissing missing;
/// The note changed on another device: the card is read again and File
/// waits until it has been.
@override@JsonKey() final  bool changed;
/// Hidden for this visit.
@override@JsonKey() final  bool skipped;
/// Why the AI's suggestions are missing, when they are.
@override final  String? aiError;

/// Create a copy of TriageCardState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TriageCardStateCopyWith<_TriageCardState> get copyWith => __$TriageCardStateCopyWithImpl<_TriageCardState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TriageCardState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.title, title) || other.title == title)&&(identical(other.snippet, snippet) || other.snippet == snippet)&&(identical(other.updatedMs, updatedMs) || other.updatedMs == updatedMs)&&const DeepCollectionEquality().equals(other._chips, _chips)&&(identical(other.missing, missing) || other.missing == missing)&&(identical(other.changed, changed) || other.changed == changed)&&(identical(other.skipped, skipped) || other.skipped == skipped)&&(identical(other.aiError, aiError) || other.aiError == aiError));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,rev,title,snippet,updatedMs,const DeepCollectionEquality().hash(_chips),missing,changed,skipped,aiError);

@override
String toString() {
  return 'TriageCardState(noteId: $noteId, rev: $rev, title: $title, snippet: $snippet, updatedMs: $updatedMs, chips: $chips, missing: $missing, changed: $changed, skipped: $skipped, aiError: $aiError)';
}


}

/// @nodoc
abstract mixin class _$TriageCardStateCopyWith<$Res> implements $TriageCardStateCopyWith<$Res> {
  factory _$TriageCardStateCopyWith(_TriageCardState value, $Res Function(_TriageCardState) _then) = __$TriageCardStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, String rev, String title, String snippet, int updatedMs, List<TriageChipState> chips, TriageMissing missing, bool changed, bool skipped, String? aiError
});




}
/// @nodoc
class __$TriageCardStateCopyWithImpl<$Res>
    implements _$TriageCardStateCopyWith<$Res> {
  __$TriageCardStateCopyWithImpl(this._self, this._then);

  final _TriageCardState _self;
  final $Res Function(_TriageCardState) _then;

/// Create a copy of TriageCardState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? rev = null,Object? title = null,Object? snippet = null,Object? updatedMs = null,Object? chips = null,Object? missing = null,Object? changed = null,Object? skipped = null,Object? aiError = freezed,}) {
  return _then(_TriageCardState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,snippet: null == snippet ? _self.snippet : snippet // ignore: cast_nullable_to_non_nullable
as String,updatedMs: null == updatedMs ? _self.updatedMs : updatedMs // ignore: cast_nullable_to_non_nullable
as int,chips: null == chips ? _self._chips : chips // ignore: cast_nullable_to_non_nullable
as List<TriageChipState>,missing: null == missing ? _self.missing : missing // ignore: cast_nullable_to_non_nullable
as TriageMissing,changed: null == changed ? _self.changed : changed // ignore: cast_nullable_to_non_nullable
as bool,skipped: null == skipped ? _self.skipped : skipped // ignore: cast_nullable_to_non_nullable
as bool,aiError: freezed == aiError ? _self.aiError : aiError // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
