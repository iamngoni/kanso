// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'editor_destination.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$EditorDestination {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorDestination);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorDestination()';
}


}

/// @nodoc
class $EditorDestinationCopyWith<$Res>  {
$EditorDestinationCopyWith(EditorDestination _, $Res Function(EditorDestination) __);
}


/// Adds pattern-matching-related methods to [EditorDestination].
extension EditorDestinationPatterns on EditorDestination {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteDestination value)?  note,TResult Function( NewNoteDestination value)?  newNote,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteDestination() when note != null:
return note(_that);case NewNoteDestination() when newNote != null:
return newNote(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteDestination value)  note,required TResult Function( NewNoteDestination value)  newNote,}){
final _that = this;
switch (_that) {
case NoteDestination():
return note(_that);case NewNoteDestination():
return newNote(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteDestination value)?  note,TResult? Function( NewNoteDestination value)?  newNote,}){
final _that = this;
switch (_that) {
case NoteDestination() when note != null:
return note(_that);case NewNoteDestination() when newNote != null:
return newNote(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  note,TResult Function( String title)?  newNote,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteDestination() when note != null:
return note(_that.noteId);case NewNoteDestination() when newNote != null:
return newNote(_that.title);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  note,required TResult Function( String title)  newNote,}) {final _that = this;
switch (_that) {
case NoteDestination():
return note(_that.noteId);case NewNoteDestination():
return newNote(_that.title);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  note,TResult? Function( String title)?  newNote,}) {final _that = this;
switch (_that) {
case NoteDestination() when note != null:
return note(_that.noteId);case NewNoteDestination() when newNote != null:
return newNote(_that.title);case _:
  return null;

}
}

}

/// @nodoc


class NoteDestination implements EditorDestination {
  const NoteDestination(this.noteId);
  

 final  String noteId;

/// Create a copy of EditorDestination
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteDestinationCopyWith<NoteDestination> get copyWith => _$NoteDestinationCopyWithImpl<NoteDestination>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteDestination&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'EditorDestination.note(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $NoteDestinationCopyWith<$Res> implements $EditorDestinationCopyWith<$Res> {
  factory $NoteDestinationCopyWith(NoteDestination value, $Res Function(NoteDestination) _then) = _$NoteDestinationCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$NoteDestinationCopyWithImpl<$Res>
    implements $NoteDestinationCopyWith<$Res> {
  _$NoteDestinationCopyWithImpl(this._self, this._then);

  final NoteDestination _self;
  final $Res Function(NoteDestination) _then;

/// Create a copy of EditorDestination
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(NoteDestination(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NewNoteDestination implements EditorDestination {
  const NewNoteDestination(this.title);
  

 final  String title;

/// Create a copy of EditorDestination
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NewNoteDestinationCopyWith<NewNoteDestination> get copyWith => _$NewNoteDestinationCopyWithImpl<NewNoteDestination>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NewNoteDestination&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,title);

@override
String toString() {
  return 'EditorDestination.newNote(title: $title)';
}


}

/// @nodoc
abstract mixin class $NewNoteDestinationCopyWith<$Res> implements $EditorDestinationCopyWith<$Res> {
  factory $NewNoteDestinationCopyWith(NewNoteDestination value, $Res Function(NewNoteDestination) _then) = _$NewNoteDestinationCopyWithImpl;
@useResult
$Res call({
 String title
});




}
/// @nodoc
class _$NewNoteDestinationCopyWithImpl<$Res>
    implements $NewNoteDestinationCopyWith<$Res> {
  _$NewNoteDestinationCopyWithImpl(this._self, this._then);

  final NewNoteDestination _self;
  final $Res Function(NewNoteDestination) _then;

/// Create a copy of EditorDestination
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? title = null,}) {
  return _then(NewNoteDestination(
null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
