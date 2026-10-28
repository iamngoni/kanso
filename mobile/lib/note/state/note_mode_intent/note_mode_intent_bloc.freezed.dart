// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_mode_intent_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteModeIntentEvent {

 String get noteId; NoteMode get mode;
/// Create a copy of NoteModeIntentEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteModeIntentEventCopyWith<NoteModeIntentEvent> get copyWith => _$NoteModeIntentEventCopyWithImpl<NoteModeIntentEvent>(this as NoteModeIntentEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteModeIntentEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.mode, mode) || other.mode == mode));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,mode);

@override
String toString() {
  return 'NoteModeIntentEvent(noteId: $noteId, mode: $mode)';
}


}

/// @nodoc
abstract mixin class $NoteModeIntentEventCopyWith<$Res>  {
  factory $NoteModeIntentEventCopyWith(NoteModeIntentEvent value, $Res Function(NoteModeIntentEvent) _then) = _$NoteModeIntentEventCopyWithImpl;
@useResult
$Res call({
 String noteId, NoteMode mode
});




}
/// @nodoc
class _$NoteModeIntentEventCopyWithImpl<$Res>
    implements $NoteModeIntentEventCopyWith<$Res> {
  _$NoteModeIntentEventCopyWithImpl(this._self, this._then);

  final NoteModeIntentEvent _self;
  final $Res Function(NoteModeIntentEvent) _then;

/// Create a copy of NoteModeIntentEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? mode = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as NoteMode,
  ));
}

}


/// Adds pattern-matching-related methods to [NoteModeIntentEvent].
extension NoteModeIntentEventPatterns on NoteModeIntentEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteModeIntentRequestedEvent value)?  requested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteModeIntentRequestedEvent() when requested != null:
return requested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteModeIntentRequestedEvent value)  requested,}){
final _that = this;
switch (_that) {
case NoteModeIntentRequestedEvent():
return requested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteModeIntentRequestedEvent value)?  requested,}){
final _that = this;
switch (_that) {
case NoteModeIntentRequestedEvent() when requested != null:
return requested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId,  NoteMode mode)?  requested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteModeIntentRequestedEvent() when requested != null:
return requested(_that.noteId,_that.mode);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId,  NoteMode mode)  requested,}) {final _that = this;
switch (_that) {
case NoteModeIntentRequestedEvent():
return requested(_that.noteId,_that.mode);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId,  NoteMode mode)?  requested,}) {final _that = this;
switch (_that) {
case NoteModeIntentRequestedEvent() when requested != null:
return requested(_that.noteId,_that.mode);case _:
  return null;

}
}

}

/// @nodoc


class NoteModeIntentRequestedEvent implements NoteModeIntentEvent {
  const NoteModeIntentRequestedEvent({required this.noteId, required this.mode});
  

@override final  String noteId;
@override final  NoteMode mode;

/// Create a copy of NoteModeIntentEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteModeIntentRequestedEventCopyWith<NoteModeIntentRequestedEvent> get copyWith => _$NoteModeIntentRequestedEventCopyWithImpl<NoteModeIntentRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteModeIntentRequestedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.mode, mode) || other.mode == mode));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,mode);

@override
String toString() {
  return 'NoteModeIntentEvent.requested(noteId: $noteId, mode: $mode)';
}


}

/// @nodoc
abstract mixin class $NoteModeIntentRequestedEventCopyWith<$Res> implements $NoteModeIntentEventCopyWith<$Res> {
  factory $NoteModeIntentRequestedEventCopyWith(NoteModeIntentRequestedEvent value, $Res Function(NoteModeIntentRequestedEvent) _then) = _$NoteModeIntentRequestedEventCopyWithImpl;
@override @useResult
$Res call({
 String noteId, NoteMode mode
});




}
/// @nodoc
class _$NoteModeIntentRequestedEventCopyWithImpl<$Res>
    implements $NoteModeIntentRequestedEventCopyWith<$Res> {
  _$NoteModeIntentRequestedEventCopyWithImpl(this._self, this._then);

  final NoteModeIntentRequestedEvent _self;
  final $Res Function(NoteModeIntentRequestedEvent) _then;

/// Create a copy of NoteModeIntentEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? mode = null,}) {
  return _then(NoteModeIntentRequestedEvent(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as NoteMode,
  ));
}


}

/// @nodoc
mixin _$NoteModeIntentState {

 String get noteId; NoteMode? get mode;
/// Create a copy of NoteModeIntentState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteModeIntentStateCopyWith<NoteModeIntentState> get copyWith => _$NoteModeIntentStateCopyWithImpl<NoteModeIntentState>(this as NoteModeIntentState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteModeIntentState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.mode, mode) || other.mode == mode));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,mode);

@override
String toString() {
  return 'NoteModeIntentState(noteId: $noteId, mode: $mode)';
}


}

/// @nodoc
abstract mixin class $NoteModeIntentStateCopyWith<$Res>  {
  factory $NoteModeIntentStateCopyWith(NoteModeIntentState value, $Res Function(NoteModeIntentState) _then) = _$NoteModeIntentStateCopyWithImpl;
@useResult
$Res call({
 String noteId, NoteMode? mode
});




}
/// @nodoc
class _$NoteModeIntentStateCopyWithImpl<$Res>
    implements $NoteModeIntentStateCopyWith<$Res> {
  _$NoteModeIntentStateCopyWithImpl(this._self, this._then);

  final NoteModeIntentState _self;
  final $Res Function(NoteModeIntentState) _then;

/// Create a copy of NoteModeIntentState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? mode = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,mode: freezed == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as NoteMode?,
  ));
}

}


/// Adds pattern-matching-related methods to [NoteModeIntentState].
extension NoteModeIntentStatePatterns on NoteModeIntentState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteModeIntentState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteModeIntentState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteModeIntentState value)  $default,){
final _that = this;
switch (_that) {
case _NoteModeIntentState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteModeIntentState value)?  $default,){
final _that = this;
switch (_that) {
case _NoteModeIntentState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  NoteMode? mode)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteModeIntentState() when $default != null:
return $default(_that.noteId,_that.mode);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  NoteMode? mode)  $default,) {final _that = this;
switch (_that) {
case _NoteModeIntentState():
return $default(_that.noteId,_that.mode);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  NoteMode? mode)?  $default,) {final _that = this;
switch (_that) {
case _NoteModeIntentState() when $default != null:
return $default(_that.noteId,_that.mode);case _:
  return null;

}
}

}

/// @nodoc


class _NoteModeIntentState implements NoteModeIntentState {
  const _NoteModeIntentState({this.noteId = '', this.mode});
  

@override@JsonKey() final  String noteId;
@override final  NoteMode? mode;

/// Create a copy of NoteModeIntentState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteModeIntentStateCopyWith<_NoteModeIntentState> get copyWith => __$NoteModeIntentStateCopyWithImpl<_NoteModeIntentState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteModeIntentState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.mode, mode) || other.mode == mode));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,mode);

@override
String toString() {
  return 'NoteModeIntentState(noteId: $noteId, mode: $mode)';
}


}

/// @nodoc
abstract mixin class _$NoteModeIntentStateCopyWith<$Res> implements $NoteModeIntentStateCopyWith<$Res> {
  factory _$NoteModeIntentStateCopyWith(_NoteModeIntentState value, $Res Function(_NoteModeIntentState) _then) = __$NoteModeIntentStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, NoteMode? mode
});




}
/// @nodoc
class __$NoteModeIntentStateCopyWithImpl<$Res>
    implements _$NoteModeIntentStateCopyWith<$Res> {
  __$NoteModeIntentStateCopyWithImpl(this._self, this._then);

  final _NoteModeIntentState _self;
  final $Res Function(_NoteModeIntentState) _then;

/// Create a copy of NoteModeIntentState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? mode = freezed,}) {
  return _then(_NoteModeIntentState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,mode: freezed == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as NoteMode?,
  ));
}


}

// dart format on
