// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_opening.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteOpening {

/// The new note.
 String get noteId;/// The place to open it in: the list it was made from, or the notebook
/// it went into when that list was the trash.
 ListPlace get place;
/// Create a copy of NoteOpening
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteOpeningCopyWith<NoteOpening> get copyWith => _$NoteOpeningCopyWithImpl<NoteOpening>(this as NoteOpening, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteOpening&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.place, place) || other.place == place));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,place);

@override
String toString() {
  return 'NoteOpening(noteId: $noteId, place: $place)';
}


}

/// @nodoc
abstract mixin class $NoteOpeningCopyWith<$Res>  {
  factory $NoteOpeningCopyWith(NoteOpening value, $Res Function(NoteOpening) _then) = _$NoteOpeningCopyWithImpl;
@useResult
$Res call({
 String noteId, ListPlace place
});


$ListPlaceCopyWith<$Res> get place;

}
/// @nodoc
class _$NoteOpeningCopyWithImpl<$Res>
    implements $NoteOpeningCopyWith<$Res> {
  _$NoteOpeningCopyWithImpl(this._self, this._then);

  final NoteOpening _self;
  final $Res Function(NoteOpening) _then;

/// Create a copy of NoteOpening
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? place = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,place: null == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as ListPlace,
  ));
}
/// Create a copy of NoteOpening
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListPlaceCopyWith<$Res> get place {
  
  return $ListPlaceCopyWith<$Res>(_self.place, (value) {
    return _then(_self.copyWith(place: value));
  });
}
}


/// Adds pattern-matching-related methods to [NoteOpening].
extension NoteOpeningPatterns on NoteOpening {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteOpening value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteOpening() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteOpening value)  $default,){
final _that = this;
switch (_that) {
case _NoteOpening():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteOpening value)?  $default,){
final _that = this;
switch (_that) {
case _NoteOpening() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  ListPlace place)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteOpening() when $default != null:
return $default(_that.noteId,_that.place);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  ListPlace place)  $default,) {final _that = this;
switch (_that) {
case _NoteOpening():
return $default(_that.noteId,_that.place);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  ListPlace place)?  $default,) {final _that = this;
switch (_that) {
case _NoteOpening() when $default != null:
return $default(_that.noteId,_that.place);case _:
  return null;

}
}

}

/// @nodoc


class _NoteOpening implements NoteOpening {
  const _NoteOpening({required this.noteId, required this.place});
  

/// The new note.
@override final  String noteId;
/// The place to open it in: the list it was made from, or the notebook
/// it went into when that list was the trash.
@override final  ListPlace place;

/// Create a copy of NoteOpening
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteOpeningCopyWith<_NoteOpening> get copyWith => __$NoteOpeningCopyWithImpl<_NoteOpening>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteOpening&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.place, place) || other.place == place));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,place);

@override
String toString() {
  return 'NoteOpening(noteId: $noteId, place: $place)';
}


}

/// @nodoc
abstract mixin class _$NoteOpeningCopyWith<$Res> implements $NoteOpeningCopyWith<$Res> {
  factory _$NoteOpeningCopyWith(_NoteOpening value, $Res Function(_NoteOpening) _then) = __$NoteOpeningCopyWithImpl;
@override @useResult
$Res call({
 String noteId, ListPlace place
});


@override $ListPlaceCopyWith<$Res> get place;

}
/// @nodoc
class __$NoteOpeningCopyWithImpl<$Res>
    implements _$NoteOpeningCopyWith<$Res> {
  __$NoteOpeningCopyWithImpl(this._self, this._then);

  final _NoteOpening _self;
  final $Res Function(_NoteOpening) _then;

/// Create a copy of NoteOpening
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? place = null,}) {
  return _then(_NoteOpening(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,place: null == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as ListPlace,
  ));
}

/// Create a copy of NoteOpening
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListPlaceCopyWith<$Res> get place {
  
  return $ListPlaceCopyWith<$Res>(_self.place, (value) {
    return _then(_self.copyWith(place: value));
  });
}
}

// dart format on
