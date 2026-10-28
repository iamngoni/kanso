// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'opened_documents_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$OpenedDocumentsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OpenedDocumentsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'OpenedDocumentsEvent()';
}


}

/// @nodoc
class $OpenedDocumentsEventCopyWith<$Res>  {
$OpenedDocumentsEventCopyWith(OpenedDocumentsEvent _, $Res Function(OpenedDocumentsEvent) __);
}


/// Adds pattern-matching-related methods to [OpenedDocumentsEvent].
extension OpenedDocumentsEventPatterns on OpenedDocumentsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( OpenedDocumentsStartedEvent value)?  started,TResult Function( OpenedDocumentsArrivedEvent value)?  arrived,TResult Function( OpenedDocumentsDismissedEvent value)?  dismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case OpenedDocumentsStartedEvent() when started != null:
return started(_that);case OpenedDocumentsArrivedEvent() when arrived != null:
return arrived(_that);case OpenedDocumentsDismissedEvent() when dismissed != null:
return dismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( OpenedDocumentsStartedEvent value)  started,required TResult Function( OpenedDocumentsArrivedEvent value)  arrived,required TResult Function( OpenedDocumentsDismissedEvent value)  dismissed,}){
final _that = this;
switch (_that) {
case OpenedDocumentsStartedEvent():
return started(_that);case OpenedDocumentsArrivedEvent():
return arrived(_that);case OpenedDocumentsDismissedEvent():
return dismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( OpenedDocumentsStartedEvent value)?  started,TResult? Function( OpenedDocumentsArrivedEvent value)?  arrived,TResult? Function( OpenedDocumentsDismissedEvent value)?  dismissed,}){
final _that = this;
switch (_that) {
case OpenedDocumentsStartedEvent() when started != null:
return started(_that);case OpenedDocumentsArrivedEvent() when arrived != null:
return arrived(_that);case OpenedDocumentsDismissedEvent() when dismissed != null:
return dismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  arrived,TResult Function()?  dismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case OpenedDocumentsStartedEvent() when started != null:
return started();case OpenedDocumentsArrivedEvent() when arrived != null:
return arrived();case OpenedDocumentsDismissedEvent() when dismissed != null:
return dismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  arrived,required TResult Function()  dismissed,}) {final _that = this;
switch (_that) {
case OpenedDocumentsStartedEvent():
return started();case OpenedDocumentsArrivedEvent():
return arrived();case OpenedDocumentsDismissedEvent():
return dismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  arrived,TResult? Function()?  dismissed,}) {final _that = this;
switch (_that) {
case OpenedDocumentsStartedEvent() when started != null:
return started();case OpenedDocumentsArrivedEvent() when arrived != null:
return arrived();case OpenedDocumentsDismissedEvent() when dismissed != null:
return dismissed();case _:
  return null;

}
}

}

/// @nodoc


class OpenedDocumentsStartedEvent implements OpenedDocumentsEvent {
  const OpenedDocumentsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OpenedDocumentsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'OpenedDocumentsEvent.started()';
}


}




/// @nodoc


class OpenedDocumentsArrivedEvent implements OpenedDocumentsEvent {
  const OpenedDocumentsArrivedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OpenedDocumentsArrivedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'OpenedDocumentsEvent.arrived()';
}


}




/// @nodoc


class OpenedDocumentsDismissedEvent implements OpenedDocumentsEvent {
  const OpenedDocumentsDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OpenedDocumentsDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'OpenedDocumentsEvent.dismissed()';
}


}




/// @nodoc
mixin _$OpenedDocumentsState {

/// Documents are being opened.
 bool get opening;/// The note to show: the last document opened. Set once, then
/// [OpenedDocumentsEvent.dismissed] clears it.
 String? get noteId;/// What went wrong with the others, a sentence each.
 List<String> get problems;/// Opening failed as a whole: the library could not be written.
 bool get failed;
/// Create a copy of OpenedDocumentsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OpenedDocumentsStateCopyWith<OpenedDocumentsState> get copyWith => _$OpenedDocumentsStateCopyWithImpl<OpenedDocumentsState>(this as OpenedDocumentsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OpenedDocumentsState&&(identical(other.opening, opening) || other.opening == opening)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&const DeepCollectionEquality().equals(other.problems, problems)&&(identical(other.failed, failed) || other.failed == failed));
}


@override
int get hashCode => Object.hash(runtimeType,opening,noteId,const DeepCollectionEquality().hash(problems),failed);

@override
String toString() {
  return 'OpenedDocumentsState(opening: $opening, noteId: $noteId, problems: $problems, failed: $failed)';
}


}

/// @nodoc
abstract mixin class $OpenedDocumentsStateCopyWith<$Res>  {
  factory $OpenedDocumentsStateCopyWith(OpenedDocumentsState value, $Res Function(OpenedDocumentsState) _then) = _$OpenedDocumentsStateCopyWithImpl;
@useResult
$Res call({
 bool opening, String? noteId, List<String> problems, bool failed
});




}
/// @nodoc
class _$OpenedDocumentsStateCopyWithImpl<$Res>
    implements $OpenedDocumentsStateCopyWith<$Res> {
  _$OpenedDocumentsStateCopyWithImpl(this._self, this._then);

  final OpenedDocumentsState _self;
  final $Res Function(OpenedDocumentsState) _then;

/// Create a copy of OpenedDocumentsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? opening = null,Object? noteId = freezed,Object? problems = null,Object? failed = null,}) {
  return _then(_self.copyWith(
opening: null == opening ? _self.opening : opening // ignore: cast_nullable_to_non_nullable
as bool,noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,problems: null == problems ? _self.problems : problems // ignore: cast_nullable_to_non_nullable
as List<String>,failed: null == failed ? _self.failed : failed // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [OpenedDocumentsState].
extension OpenedDocumentsStatePatterns on OpenedDocumentsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _OpenedDocumentsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _OpenedDocumentsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _OpenedDocumentsState value)  $default,){
final _that = this;
switch (_that) {
case _OpenedDocumentsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _OpenedDocumentsState value)?  $default,){
final _that = this;
switch (_that) {
case _OpenedDocumentsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool opening,  String? noteId,  List<String> problems,  bool failed)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _OpenedDocumentsState() when $default != null:
return $default(_that.opening,_that.noteId,_that.problems,_that.failed);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool opening,  String? noteId,  List<String> problems,  bool failed)  $default,) {final _that = this;
switch (_that) {
case _OpenedDocumentsState():
return $default(_that.opening,_that.noteId,_that.problems,_that.failed);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool opening,  String? noteId,  List<String> problems,  bool failed)?  $default,) {final _that = this;
switch (_that) {
case _OpenedDocumentsState() when $default != null:
return $default(_that.opening,_that.noteId,_that.problems,_that.failed);case _:
  return null;

}
}

}

/// @nodoc


class _OpenedDocumentsState extends OpenedDocumentsState {
  const _OpenedDocumentsState({this.opening = false, this.noteId, final  List<String> problems = const <String>[], this.failed = false}): _problems = problems,super._();
  

/// Documents are being opened.
@override@JsonKey() final  bool opening;
/// The note to show: the last document opened. Set once, then
/// [OpenedDocumentsEvent.dismissed] clears it.
@override final  String? noteId;
/// What went wrong with the others, a sentence each.
 final  List<String> _problems;
/// What went wrong with the others, a sentence each.
@override@JsonKey() List<String> get problems {
  if (_problems is EqualUnmodifiableListView) return _problems;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_problems);
}

/// Opening failed as a whole: the library could not be written.
@override@JsonKey() final  bool failed;

/// Create a copy of OpenedDocumentsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$OpenedDocumentsStateCopyWith<_OpenedDocumentsState> get copyWith => __$OpenedDocumentsStateCopyWithImpl<_OpenedDocumentsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _OpenedDocumentsState&&(identical(other.opening, opening) || other.opening == opening)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&const DeepCollectionEquality().equals(other._problems, _problems)&&(identical(other.failed, failed) || other.failed == failed));
}


@override
int get hashCode => Object.hash(runtimeType,opening,noteId,const DeepCollectionEquality().hash(_problems),failed);

@override
String toString() {
  return 'OpenedDocumentsState(opening: $opening, noteId: $noteId, problems: $problems, failed: $failed)';
}


}

/// @nodoc
abstract mixin class _$OpenedDocumentsStateCopyWith<$Res> implements $OpenedDocumentsStateCopyWith<$Res> {
  factory _$OpenedDocumentsStateCopyWith(_OpenedDocumentsState value, $Res Function(_OpenedDocumentsState) _then) = __$OpenedDocumentsStateCopyWithImpl;
@override @useResult
$Res call({
 bool opening, String? noteId, List<String> problems, bool failed
});




}
/// @nodoc
class __$OpenedDocumentsStateCopyWithImpl<$Res>
    implements _$OpenedDocumentsStateCopyWith<$Res> {
  __$OpenedDocumentsStateCopyWithImpl(this._self, this._then);

  final _OpenedDocumentsState _self;
  final $Res Function(_OpenedDocumentsState) _then;

/// Create a copy of OpenedDocumentsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? opening = null,Object? noteId = freezed,Object? problems = null,Object? failed = null,}) {
  return _then(_OpenedDocumentsState(
opening: null == opening ? _self.opening : opening // ignore: cast_nullable_to_non_nullable
as bool,noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,problems: null == problems ? _self._problems : problems // ignore: cast_nullable_to_non_nullable
as List<String>,failed: null == failed ? _self.failed : failed // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
