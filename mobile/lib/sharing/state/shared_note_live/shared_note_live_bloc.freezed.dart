// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'shared_note_live_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SharedNoteLiveEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNoteLiveEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SharedNoteLiveEvent()';
}


}

/// @nodoc
class $SharedNoteLiveEventCopyWith<$Res>  {
$SharedNoteLiveEventCopyWith(SharedNoteLiveEvent _, $Res Function(SharedNoteLiveEvent) __);
}


/// Adds pattern-matching-related methods to [SharedNoteLiveEvent].
extension SharedNoteLiveEventPatterns on SharedNoteLiveEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SharedNoteLiveOpenedEvent value)?  opened,TResult Function( SharedNoteLiveResumedEvent value)?  resumed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SharedNoteLiveOpenedEvent() when opened != null:
return opened(_that);case SharedNoteLiveResumedEvent() when resumed != null:
return resumed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SharedNoteLiveOpenedEvent value)  opened,required TResult Function( SharedNoteLiveResumedEvent value)  resumed,}){
final _that = this;
switch (_that) {
case SharedNoteLiveOpenedEvent():
return opened(_that);case SharedNoteLiveResumedEvent():
return resumed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SharedNoteLiveOpenedEvent value)?  opened,TResult? Function( SharedNoteLiveResumedEvent value)?  resumed,}){
final _that = this;
switch (_that) {
case SharedNoteLiveOpenedEvent() when opened != null:
return opened(_that);case SharedNoteLiveResumedEvent() when resumed != null:
return resumed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  opened,TResult Function()?  resumed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SharedNoteLiveOpenedEvent() when opened != null:
return opened(_that.noteId);case SharedNoteLiveResumedEvent() when resumed != null:
return resumed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  opened,required TResult Function()  resumed,}) {final _that = this;
switch (_that) {
case SharedNoteLiveOpenedEvent():
return opened(_that.noteId);case SharedNoteLiveResumedEvent():
return resumed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  opened,TResult? Function()?  resumed,}) {final _that = this;
switch (_that) {
case SharedNoteLiveOpenedEvent() when opened != null:
return opened(_that.noteId);case SharedNoteLiveResumedEvent() when resumed != null:
return resumed();case _:
  return null;

}
}

}

/// @nodoc


class SharedNoteLiveOpenedEvent implements SharedNoteLiveEvent {
  const SharedNoteLiveOpenedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of SharedNoteLiveEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharedNoteLiveOpenedEventCopyWith<SharedNoteLiveOpenedEvent> get copyWith => _$SharedNoteLiveOpenedEventCopyWithImpl<SharedNoteLiveOpenedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNoteLiveOpenedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'SharedNoteLiveEvent.opened(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $SharedNoteLiveOpenedEventCopyWith<$Res> implements $SharedNoteLiveEventCopyWith<$Res> {
  factory $SharedNoteLiveOpenedEventCopyWith(SharedNoteLiveOpenedEvent value, $Res Function(SharedNoteLiveOpenedEvent) _then) = _$SharedNoteLiveOpenedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$SharedNoteLiveOpenedEventCopyWithImpl<$Res>
    implements $SharedNoteLiveOpenedEventCopyWith<$Res> {
  _$SharedNoteLiveOpenedEventCopyWithImpl(this._self, this._then);

  final SharedNoteLiveOpenedEvent _self;
  final $Res Function(SharedNoteLiveOpenedEvent) _then;

/// Create a copy of SharedNoteLiveEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(SharedNoteLiveOpenedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SharedNoteLiveResumedEvent implements SharedNoteLiveEvent {
  const SharedNoteLiveResumedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNoteLiveResumedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SharedNoteLiveEvent.resumed()';
}


}




/// @nodoc
mixin _$SharedNoteLiveState {

/// The note whose room this is.
 String get noteId;/// The room is open.
 bool get live;
/// Create a copy of SharedNoteLiveState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharedNoteLiveStateCopyWith<SharedNoteLiveState> get copyWith => _$SharedNoteLiveStateCopyWithImpl<SharedNoteLiveState>(this as SharedNoteLiveState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNoteLiveState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.live, live) || other.live == live));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,live);

@override
String toString() {
  return 'SharedNoteLiveState(noteId: $noteId, live: $live)';
}


}

/// @nodoc
abstract mixin class $SharedNoteLiveStateCopyWith<$Res>  {
  factory $SharedNoteLiveStateCopyWith(SharedNoteLiveState value, $Res Function(SharedNoteLiveState) _then) = _$SharedNoteLiveStateCopyWithImpl;
@useResult
$Res call({
 String noteId, bool live
});




}
/// @nodoc
class _$SharedNoteLiveStateCopyWithImpl<$Res>
    implements $SharedNoteLiveStateCopyWith<$Res> {
  _$SharedNoteLiveStateCopyWithImpl(this._self, this._then);

  final SharedNoteLiveState _self;
  final $Res Function(SharedNoteLiveState) _then;

/// Create a copy of SharedNoteLiveState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? live = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,live: null == live ? _self.live : live // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [SharedNoteLiveState].
extension SharedNoteLiveStatePatterns on SharedNoteLiveState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SharedNoteLiveState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SharedNoteLiveState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SharedNoteLiveState value)  $default,){
final _that = this;
switch (_that) {
case _SharedNoteLiveState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SharedNoteLiveState value)?  $default,){
final _that = this;
switch (_that) {
case _SharedNoteLiveState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  bool live)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SharedNoteLiveState() when $default != null:
return $default(_that.noteId,_that.live);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  bool live)  $default,) {final _that = this;
switch (_that) {
case _SharedNoteLiveState():
return $default(_that.noteId,_that.live);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  bool live)?  $default,) {final _that = this;
switch (_that) {
case _SharedNoteLiveState() when $default != null:
return $default(_that.noteId,_that.live);case _:
  return null;

}
}

}

/// @nodoc


class _SharedNoteLiveState implements SharedNoteLiveState {
  const _SharedNoteLiveState({this.noteId = '', this.live = false});
  

/// The note whose room this is.
@override@JsonKey() final  String noteId;
/// The room is open.
@override@JsonKey() final  bool live;

/// Create a copy of SharedNoteLiveState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SharedNoteLiveStateCopyWith<_SharedNoteLiveState> get copyWith => __$SharedNoteLiveStateCopyWithImpl<_SharedNoteLiveState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SharedNoteLiveState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.live, live) || other.live == live));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,live);

@override
String toString() {
  return 'SharedNoteLiveState(noteId: $noteId, live: $live)';
}


}

/// @nodoc
abstract mixin class _$SharedNoteLiveStateCopyWith<$Res> implements $SharedNoteLiveStateCopyWith<$Res> {
  factory _$SharedNoteLiveStateCopyWith(_SharedNoteLiveState value, $Res Function(_SharedNoteLiveState) _then) = __$SharedNoteLiveStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, bool live
});




}
/// @nodoc
class __$SharedNoteLiveStateCopyWithImpl<$Res>
    implements _$SharedNoteLiveStateCopyWith<$Res> {
  __$SharedNoteLiveStateCopyWithImpl(this._self, this._then);

  final _SharedNoteLiveState _self;
  final $Res Function(_SharedNoteLiveState) _then;

/// Create a copy of SharedNoteLiveState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? live = null,}) {
  return _then(_SharedNoteLiveState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,live: null == live ? _self.live : live // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
