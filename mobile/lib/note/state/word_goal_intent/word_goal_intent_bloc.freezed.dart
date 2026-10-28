// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'word_goal_intent_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$WordGoalIntentEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WordGoalIntentEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WordGoalIntentEvent()';
}


}

/// @nodoc
class $WordGoalIntentEventCopyWith<$Res>  {
$WordGoalIntentEventCopyWith(WordGoalIntentEvent _, $Res Function(WordGoalIntentEvent) __);
}


/// Adds pattern-matching-related methods to [WordGoalIntentEvent].
extension WordGoalIntentEventPatterns on WordGoalIntentEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WordGoalIntentRequestedEvent value)?  requested,TResult Function( WordGoalIntentConsumedEvent value)?  consumed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WordGoalIntentRequestedEvent() when requested != null:
return requested(_that);case WordGoalIntentConsumedEvent() when consumed != null:
return consumed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WordGoalIntentRequestedEvent value)  requested,required TResult Function( WordGoalIntentConsumedEvent value)  consumed,}){
final _that = this;
switch (_that) {
case WordGoalIntentRequestedEvent():
return requested(_that);case WordGoalIntentConsumedEvent():
return consumed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WordGoalIntentRequestedEvent value)?  requested,TResult? Function( WordGoalIntentConsumedEvent value)?  consumed,}){
final _that = this;
switch (_that) {
case WordGoalIntentRequestedEvent() when requested != null:
return requested(_that);case WordGoalIntentConsumedEvent() when consumed != null:
return consumed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  requested,TResult Function( int request)?  consumed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WordGoalIntentRequestedEvent() when requested != null:
return requested(_that.noteId);case WordGoalIntentConsumedEvent() when consumed != null:
return consumed(_that.request);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  requested,required TResult Function( int request)  consumed,}) {final _that = this;
switch (_that) {
case WordGoalIntentRequestedEvent():
return requested(_that.noteId);case WordGoalIntentConsumedEvent():
return consumed(_that.request);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  requested,TResult? Function( int request)?  consumed,}) {final _that = this;
switch (_that) {
case WordGoalIntentRequestedEvent() when requested != null:
return requested(_that.noteId);case WordGoalIntentConsumedEvent() when consumed != null:
return consumed(_that.request);case _:
  return null;

}
}

}

/// @nodoc


class WordGoalIntentRequestedEvent implements WordGoalIntentEvent {
  const WordGoalIntentRequestedEvent({required this.noteId});
  

 final  String noteId;

/// Create a copy of WordGoalIntentEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WordGoalIntentRequestedEventCopyWith<WordGoalIntentRequestedEvent> get copyWith => _$WordGoalIntentRequestedEventCopyWithImpl<WordGoalIntentRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WordGoalIntentRequestedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'WordGoalIntentEvent.requested(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $WordGoalIntentRequestedEventCopyWith<$Res> implements $WordGoalIntentEventCopyWith<$Res> {
  factory $WordGoalIntentRequestedEventCopyWith(WordGoalIntentRequestedEvent value, $Res Function(WordGoalIntentRequestedEvent) _then) = _$WordGoalIntentRequestedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$WordGoalIntentRequestedEventCopyWithImpl<$Res>
    implements $WordGoalIntentRequestedEventCopyWith<$Res> {
  _$WordGoalIntentRequestedEventCopyWithImpl(this._self, this._then);

  final WordGoalIntentRequestedEvent _self;
  final $Res Function(WordGoalIntentRequestedEvent) _then;

/// Create a copy of WordGoalIntentEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(WordGoalIntentRequestedEvent(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class WordGoalIntentConsumedEvent implements WordGoalIntentEvent {
  const WordGoalIntentConsumedEvent({required this.request});
  

 final  int request;

/// Create a copy of WordGoalIntentEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WordGoalIntentConsumedEventCopyWith<WordGoalIntentConsumedEvent> get copyWith => _$WordGoalIntentConsumedEventCopyWithImpl<WordGoalIntentConsumedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WordGoalIntentConsumedEvent&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'WordGoalIntentEvent.consumed(request: $request)';
}


}

/// @nodoc
abstract mixin class $WordGoalIntentConsumedEventCopyWith<$Res> implements $WordGoalIntentEventCopyWith<$Res> {
  factory $WordGoalIntentConsumedEventCopyWith(WordGoalIntentConsumedEvent value, $Res Function(WordGoalIntentConsumedEvent) _then) = _$WordGoalIntentConsumedEventCopyWithImpl;
@useResult
$Res call({
 int request
});




}
/// @nodoc
class _$WordGoalIntentConsumedEventCopyWithImpl<$Res>
    implements $WordGoalIntentConsumedEventCopyWith<$Res> {
  _$WordGoalIntentConsumedEventCopyWithImpl(this._self, this._then);

  final WordGoalIntentConsumedEvent _self;
  final $Res Function(WordGoalIntentConsumedEvent) _then;

/// Create a copy of WordGoalIntentEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(WordGoalIntentConsumedEvent(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc
mixin _$WordGoalIntentState {

 int get request; String get noteId;
/// Create a copy of WordGoalIntentState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WordGoalIntentStateCopyWith<WordGoalIntentState> get copyWith => _$WordGoalIntentStateCopyWithImpl<WordGoalIntentState>(this as WordGoalIntentState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WordGoalIntentState&&(identical(other.request, request) || other.request == request)&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,request,noteId);

@override
String toString() {
  return 'WordGoalIntentState(request: $request, noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $WordGoalIntentStateCopyWith<$Res>  {
  factory $WordGoalIntentStateCopyWith(WordGoalIntentState value, $Res Function(WordGoalIntentState) _then) = _$WordGoalIntentStateCopyWithImpl;
@useResult
$Res call({
 int request, String noteId
});




}
/// @nodoc
class _$WordGoalIntentStateCopyWithImpl<$Res>
    implements $WordGoalIntentStateCopyWith<$Res> {
  _$WordGoalIntentStateCopyWithImpl(this._self, this._then);

  final WordGoalIntentState _self;
  final $Res Function(WordGoalIntentState) _then;

/// Create a copy of WordGoalIntentState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? request = null,Object? noteId = null,}) {
  return _then(_self.copyWith(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as int,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [WordGoalIntentState].
extension WordGoalIntentStatePatterns on WordGoalIntentState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _WordGoalIntentState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _WordGoalIntentState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _WordGoalIntentState value)  $default,){
final _that = this;
switch (_that) {
case _WordGoalIntentState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _WordGoalIntentState value)?  $default,){
final _that = this;
switch (_that) {
case _WordGoalIntentState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int request,  String noteId)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _WordGoalIntentState() when $default != null:
return $default(_that.request,_that.noteId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int request,  String noteId)  $default,) {final _that = this;
switch (_that) {
case _WordGoalIntentState():
return $default(_that.request,_that.noteId);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int request,  String noteId)?  $default,) {final _that = this;
switch (_that) {
case _WordGoalIntentState() when $default != null:
return $default(_that.request,_that.noteId);case _:
  return null;

}
}

}

/// @nodoc


class _WordGoalIntentState extends WordGoalIntentState {
  const _WordGoalIntentState({this.request = 0, this.noteId = ''}): super._();
  

@override@JsonKey() final  int request;
@override@JsonKey() final  String noteId;

/// Create a copy of WordGoalIntentState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$WordGoalIntentStateCopyWith<_WordGoalIntentState> get copyWith => __$WordGoalIntentStateCopyWithImpl<_WordGoalIntentState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _WordGoalIntentState&&(identical(other.request, request) || other.request == request)&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,request,noteId);

@override
String toString() {
  return 'WordGoalIntentState(request: $request, noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class _$WordGoalIntentStateCopyWith<$Res> implements $WordGoalIntentStateCopyWith<$Res> {
  factory _$WordGoalIntentStateCopyWith(_WordGoalIntentState value, $Res Function(_WordGoalIntentState) _then) = __$WordGoalIntentStateCopyWithImpl;
@override @useResult
$Res call({
 int request, String noteId
});




}
/// @nodoc
class __$WordGoalIntentStateCopyWithImpl<$Res>
    implements _$WordGoalIntentStateCopyWith<$Res> {
  __$WordGoalIntentStateCopyWithImpl(this._self, this._then);

  final _WordGoalIntentState _self;
  final $Res Function(_WordGoalIntentState) _then;

/// Create a copy of WordGoalIntentState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? request = null,Object? noteId = null,}) {
  return _then(_WordGoalIntentState(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as int,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
