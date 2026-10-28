// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_window_request.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteWindowRequest {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteWindowRequest);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteWindowRequest()';
}


}

/// @nodoc
class $NoteWindowRequestCopyWith<$Res>  {
$NoteWindowRequestCopyWith(NoteWindowRequest _, $Res Function(NoteWindowRequest) __);
}


/// Adds pattern-matching-related methods to [NoteWindowRequest].
extension NoteWindowRequestPatterns on NoteWindowRequest {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NewNoteRequest value)?  newNote,TResult Function( NewNoteTitledRequest value)?  newNoteTitled,TResult Function( SearchRequest value)?  search,TResult Function( PaletteRequest value)?  palette,TResult Function( OpenNoteRequest value)?  openNote,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NewNoteRequest() when newNote != null:
return newNote(_that);case NewNoteTitledRequest() when newNoteTitled != null:
return newNoteTitled(_that);case SearchRequest() when search != null:
return search(_that);case PaletteRequest() when palette != null:
return palette(_that);case OpenNoteRequest() when openNote != null:
return openNote(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NewNoteRequest value)  newNote,required TResult Function( NewNoteTitledRequest value)  newNoteTitled,required TResult Function( SearchRequest value)  search,required TResult Function( PaletteRequest value)  palette,required TResult Function( OpenNoteRequest value)  openNote,}){
final _that = this;
switch (_that) {
case NewNoteRequest():
return newNote(_that);case NewNoteTitledRequest():
return newNoteTitled(_that);case SearchRequest():
return search(_that);case PaletteRequest():
return palette(_that);case OpenNoteRequest():
return openNote(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NewNoteRequest value)?  newNote,TResult? Function( NewNoteTitledRequest value)?  newNoteTitled,TResult? Function( SearchRequest value)?  search,TResult? Function( PaletteRequest value)?  palette,TResult? Function( OpenNoteRequest value)?  openNote,}){
final _that = this;
switch (_that) {
case NewNoteRequest() when newNote != null:
return newNote(_that);case NewNoteTitledRequest() when newNoteTitled != null:
return newNoteTitled(_that);case SearchRequest() when search != null:
return search(_that);case PaletteRequest() when palette != null:
return palette(_that);case OpenNoteRequest() when openNote != null:
return openNote(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  newNote,TResult Function( String title)?  newNoteTitled,TResult Function()?  search,TResult Function()?  palette,TResult Function( String noteId)?  openNote,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NewNoteRequest() when newNote != null:
return newNote();case NewNoteTitledRequest() when newNoteTitled != null:
return newNoteTitled(_that.title);case SearchRequest() when search != null:
return search();case PaletteRequest() when palette != null:
return palette();case OpenNoteRequest() when openNote != null:
return openNote(_that.noteId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  newNote,required TResult Function( String title)  newNoteTitled,required TResult Function()  search,required TResult Function()  palette,required TResult Function( String noteId)  openNote,}) {final _that = this;
switch (_that) {
case NewNoteRequest():
return newNote();case NewNoteTitledRequest():
return newNoteTitled(_that.title);case SearchRequest():
return search();case PaletteRequest():
return palette();case OpenNoteRequest():
return openNote(_that.noteId);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  newNote,TResult? Function( String title)?  newNoteTitled,TResult? Function()?  search,TResult? Function()?  palette,TResult? Function( String noteId)?  openNote,}) {final _that = this;
switch (_that) {
case NewNoteRequest() when newNote != null:
return newNote();case NewNoteTitledRequest() when newNoteTitled != null:
return newNoteTitled(_that.title);case SearchRequest() when search != null:
return search();case PaletteRequest() when palette != null:
return palette();case OpenNoteRequest() when openNote != null:
return openNote(_that.noteId);case _:
  return null;

}
}

}

/// @nodoc


class NewNoteRequest implements NoteWindowRequest {
  const NewNoteRequest();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NewNoteRequest);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteWindowRequest.newNote()';
}


}




/// @nodoc


class NewNoteTitledRequest implements NoteWindowRequest {
  const NewNoteTitledRequest(this.title);
  

 final  String title;

/// Create a copy of NoteWindowRequest
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NewNoteTitledRequestCopyWith<NewNoteTitledRequest> get copyWith => _$NewNoteTitledRequestCopyWithImpl<NewNoteTitledRequest>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NewNoteTitledRequest&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,title);

@override
String toString() {
  return 'NoteWindowRequest.newNoteTitled(title: $title)';
}


}

/// @nodoc
abstract mixin class $NewNoteTitledRequestCopyWith<$Res> implements $NoteWindowRequestCopyWith<$Res> {
  factory $NewNoteTitledRequestCopyWith(NewNoteTitledRequest value, $Res Function(NewNoteTitledRequest) _then) = _$NewNoteTitledRequestCopyWithImpl;
@useResult
$Res call({
 String title
});




}
/// @nodoc
class _$NewNoteTitledRequestCopyWithImpl<$Res>
    implements $NewNoteTitledRequestCopyWith<$Res> {
  _$NewNoteTitledRequestCopyWithImpl(this._self, this._then);

  final NewNoteTitledRequest _self;
  final $Res Function(NewNoteTitledRequest) _then;

/// Create a copy of NoteWindowRequest
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? title = null,}) {
  return _then(NewNoteTitledRequest(
null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SearchRequest implements NoteWindowRequest {
  const SearchRequest();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchRequest);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteWindowRequest.search()';
}


}




/// @nodoc


class PaletteRequest implements NoteWindowRequest {
  const PaletteRequest();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteRequest);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteWindowRequest.palette()';
}


}




/// @nodoc


class OpenNoteRequest implements NoteWindowRequest {
  const OpenNoteRequest(this.noteId);
  

 final  String noteId;

/// Create a copy of NoteWindowRequest
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OpenNoteRequestCopyWith<OpenNoteRequest> get copyWith => _$OpenNoteRequestCopyWithImpl<OpenNoteRequest>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OpenNoteRequest&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'NoteWindowRequest.openNote(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $OpenNoteRequestCopyWith<$Res> implements $NoteWindowRequestCopyWith<$Res> {
  factory $OpenNoteRequestCopyWith(OpenNoteRequest value, $Res Function(OpenNoteRequest) _then) = _$OpenNoteRequestCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$OpenNoteRequestCopyWithImpl<$Res>
    implements $OpenNoteRequestCopyWith<$Res> {
  _$OpenNoteRequestCopyWithImpl(this._self, this._then);

  final OpenNoteRequest _self;
  final $Res Function(OpenNoteRequest) _then;

/// Create a copy of NoteWindowRequest
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(OpenNoteRequest(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
