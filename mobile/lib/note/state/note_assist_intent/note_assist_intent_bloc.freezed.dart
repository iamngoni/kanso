// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_assist_intent_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteAssistIntentEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteAssistIntentEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteAssistIntentEvent()';
}


}

/// @nodoc
class $NoteAssistIntentEventCopyWith<$Res>  {
$NoteAssistIntentEventCopyWith(NoteAssistIntentEvent _, $Res Function(NoteAssistIntentEvent) __);
}


/// Adds pattern-matching-related methods to [NoteAssistIntentEvent].
extension NoteAssistIntentEventPatterns on NoteAssistIntentEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteAssistIntentRequestedEvent value)?  requested,TResult Function( NoteAssistIntentConsumedEvent value)?  consumed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteAssistIntentRequestedEvent() when requested != null:
return requested(_that);case NoteAssistIntentConsumedEvent() when consumed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteAssistIntentRequestedEvent value)  requested,required TResult Function( NoteAssistIntentConsumedEvent value)  consumed,}){
final _that = this;
switch (_that) {
case NoteAssistIntentRequestedEvent():
return requested(_that);case NoteAssistIntentConsumedEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteAssistIntentRequestedEvent value)?  requested,TResult? Function( NoteAssistIntentConsumedEvent value)?  consumed,}){
final _that = this;
switch (_that) {
case NoteAssistIntentRequestedEvent() when requested != null:
return requested(_that);case NoteAssistIntentConsumedEvent() when consumed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId,  AssistAction? action,  bool summarizeNote)?  requested,TResult Function( int request)?  consumed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteAssistIntentRequestedEvent() when requested != null:
return requested(_that.noteId,_that.action,_that.summarizeNote);case NoteAssistIntentConsumedEvent() when consumed != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId,  AssistAction? action,  bool summarizeNote)  requested,required TResult Function( int request)  consumed,}) {final _that = this;
switch (_that) {
case NoteAssistIntentRequestedEvent():
return requested(_that.noteId,_that.action,_that.summarizeNote);case NoteAssistIntentConsumedEvent():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId,  AssistAction? action,  bool summarizeNote)?  requested,TResult? Function( int request)?  consumed,}) {final _that = this;
switch (_that) {
case NoteAssistIntentRequestedEvent() when requested != null:
return requested(_that.noteId,_that.action,_that.summarizeNote);case NoteAssistIntentConsumedEvent() when consumed != null:
return consumed(_that.request);case _:
  return null;

}
}

}

/// @nodoc


class NoteAssistIntentRequestedEvent implements NoteAssistIntentEvent {
  const NoteAssistIntentRequestedEvent({required this.noteId, this.action, this.summarizeNote = false});
  

 final  String noteId;
 final  AssistAction? action;
@JsonKey() final  bool summarizeNote;

/// Create a copy of NoteAssistIntentEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteAssistIntentRequestedEventCopyWith<NoteAssistIntentRequestedEvent> get copyWith => _$NoteAssistIntentRequestedEventCopyWithImpl<NoteAssistIntentRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteAssistIntentRequestedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.action, action) || other.action == action)&&(identical(other.summarizeNote, summarizeNote) || other.summarizeNote == summarizeNote));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,action,summarizeNote);

@override
String toString() {
  return 'NoteAssistIntentEvent.requested(noteId: $noteId, action: $action, summarizeNote: $summarizeNote)';
}


}

/// @nodoc
abstract mixin class $NoteAssistIntentRequestedEventCopyWith<$Res> implements $NoteAssistIntentEventCopyWith<$Res> {
  factory $NoteAssistIntentRequestedEventCopyWith(NoteAssistIntentRequestedEvent value, $Res Function(NoteAssistIntentRequestedEvent) _then) = _$NoteAssistIntentRequestedEventCopyWithImpl;
@useResult
$Res call({
 String noteId, AssistAction? action, bool summarizeNote
});




}
/// @nodoc
class _$NoteAssistIntentRequestedEventCopyWithImpl<$Res>
    implements $NoteAssistIntentRequestedEventCopyWith<$Res> {
  _$NoteAssistIntentRequestedEventCopyWithImpl(this._self, this._then);

  final NoteAssistIntentRequestedEvent _self;
  final $Res Function(NoteAssistIntentRequestedEvent) _then;

/// Create a copy of NoteAssistIntentEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? action = freezed,Object? summarizeNote = null,}) {
  return _then(NoteAssistIntentRequestedEvent(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as AssistAction?,summarizeNote: null == summarizeNote ? _self.summarizeNote : summarizeNote // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class NoteAssistIntentConsumedEvent implements NoteAssistIntentEvent {
  const NoteAssistIntentConsumedEvent({required this.request});
  

 final  int request;

/// Create a copy of NoteAssistIntentEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteAssistIntentConsumedEventCopyWith<NoteAssistIntentConsumedEvent> get copyWith => _$NoteAssistIntentConsumedEventCopyWithImpl<NoteAssistIntentConsumedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteAssistIntentConsumedEvent&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'NoteAssistIntentEvent.consumed(request: $request)';
}


}

/// @nodoc
abstract mixin class $NoteAssistIntentConsumedEventCopyWith<$Res> implements $NoteAssistIntentEventCopyWith<$Res> {
  factory $NoteAssistIntentConsumedEventCopyWith(NoteAssistIntentConsumedEvent value, $Res Function(NoteAssistIntentConsumedEvent) _then) = _$NoteAssistIntentConsumedEventCopyWithImpl;
@useResult
$Res call({
 int request
});




}
/// @nodoc
class _$NoteAssistIntentConsumedEventCopyWithImpl<$Res>
    implements $NoteAssistIntentConsumedEventCopyWith<$Res> {
  _$NoteAssistIntentConsumedEventCopyWithImpl(this._self, this._then);

  final NoteAssistIntentConsumedEvent _self;
  final $Res Function(NoteAssistIntentConsumedEvent) _then;

/// Create a copy of NoteAssistIntentEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(NoteAssistIntentConsumedEvent(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc
mixin _$NoteAssistIntentState {

 int get request; String get noteId; AssistAction? get action; bool get summarizeNote;
/// Create a copy of NoteAssistIntentState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteAssistIntentStateCopyWith<NoteAssistIntentState> get copyWith => _$NoteAssistIntentStateCopyWithImpl<NoteAssistIntentState>(this as NoteAssistIntentState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteAssistIntentState&&(identical(other.request, request) || other.request == request)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.action, action) || other.action == action)&&(identical(other.summarizeNote, summarizeNote) || other.summarizeNote == summarizeNote));
}


@override
int get hashCode => Object.hash(runtimeType,request,noteId,action,summarizeNote);

@override
String toString() {
  return 'NoteAssistIntentState(request: $request, noteId: $noteId, action: $action, summarizeNote: $summarizeNote)';
}


}

/// @nodoc
abstract mixin class $NoteAssistIntentStateCopyWith<$Res>  {
  factory $NoteAssistIntentStateCopyWith(NoteAssistIntentState value, $Res Function(NoteAssistIntentState) _then) = _$NoteAssistIntentStateCopyWithImpl;
@useResult
$Res call({
 int request, String noteId, AssistAction? action, bool summarizeNote
});




}
/// @nodoc
class _$NoteAssistIntentStateCopyWithImpl<$Res>
    implements $NoteAssistIntentStateCopyWith<$Res> {
  _$NoteAssistIntentStateCopyWithImpl(this._self, this._then);

  final NoteAssistIntentState _self;
  final $Res Function(NoteAssistIntentState) _then;

/// Create a copy of NoteAssistIntentState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? request = null,Object? noteId = null,Object? action = freezed,Object? summarizeNote = null,}) {
  return _then(_self.copyWith(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as int,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as AssistAction?,summarizeNote: null == summarizeNote ? _self.summarizeNote : summarizeNote // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [NoteAssistIntentState].
extension NoteAssistIntentStatePatterns on NoteAssistIntentState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteAssistIntentState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteAssistIntentState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteAssistIntentState value)  $default,){
final _that = this;
switch (_that) {
case _NoteAssistIntentState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteAssistIntentState value)?  $default,){
final _that = this;
switch (_that) {
case _NoteAssistIntentState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int request,  String noteId,  AssistAction? action,  bool summarizeNote)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteAssistIntentState() when $default != null:
return $default(_that.request,_that.noteId,_that.action,_that.summarizeNote);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int request,  String noteId,  AssistAction? action,  bool summarizeNote)  $default,) {final _that = this;
switch (_that) {
case _NoteAssistIntentState():
return $default(_that.request,_that.noteId,_that.action,_that.summarizeNote);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int request,  String noteId,  AssistAction? action,  bool summarizeNote)?  $default,) {final _that = this;
switch (_that) {
case _NoteAssistIntentState() when $default != null:
return $default(_that.request,_that.noteId,_that.action,_that.summarizeNote);case _:
  return null;

}
}

}

/// @nodoc


class _NoteAssistIntentState extends NoteAssistIntentState {
  const _NoteAssistIntentState({this.request = 0, this.noteId = '', this.action, this.summarizeNote = false}): super._();
  

@override@JsonKey() final  int request;
@override@JsonKey() final  String noteId;
@override final  AssistAction? action;
@override@JsonKey() final  bool summarizeNote;

/// Create a copy of NoteAssistIntentState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteAssistIntentStateCopyWith<_NoteAssistIntentState> get copyWith => __$NoteAssistIntentStateCopyWithImpl<_NoteAssistIntentState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteAssistIntentState&&(identical(other.request, request) || other.request == request)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.action, action) || other.action == action)&&(identical(other.summarizeNote, summarizeNote) || other.summarizeNote == summarizeNote));
}


@override
int get hashCode => Object.hash(runtimeType,request,noteId,action,summarizeNote);

@override
String toString() {
  return 'NoteAssistIntentState(request: $request, noteId: $noteId, action: $action, summarizeNote: $summarizeNote)';
}


}

/// @nodoc
abstract mixin class _$NoteAssistIntentStateCopyWith<$Res> implements $NoteAssistIntentStateCopyWith<$Res> {
  factory _$NoteAssistIntentStateCopyWith(_NoteAssistIntentState value, $Res Function(_NoteAssistIntentState) _then) = __$NoteAssistIntentStateCopyWithImpl;
@override @useResult
$Res call({
 int request, String noteId, AssistAction? action, bool summarizeNote
});




}
/// @nodoc
class __$NoteAssistIntentStateCopyWithImpl<$Res>
    implements _$NoteAssistIntentStateCopyWith<$Res> {
  __$NoteAssistIntentStateCopyWithImpl(this._self, this._then);

  final _NoteAssistIntentState _self;
  final $Res Function(_NoteAssistIntentState) _then;

/// Create a copy of NoteAssistIntentState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? request = null,Object? noteId = null,Object? action = freezed,Object? summarizeNote = null,}) {
  return _then(_NoteAssistIntentState(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as int,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as AssistAction?,summarizeNote: null == summarizeNote ? _self.summarizeNote : summarizeNote // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
