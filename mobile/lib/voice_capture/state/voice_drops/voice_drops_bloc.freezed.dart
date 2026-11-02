// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'voice_drops_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$VoiceDropsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceDropsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'VoiceDropsEvent()';
}


}

/// @nodoc
class $VoiceDropsEventCopyWith<$Res>  {
$VoiceDropsEventCopyWith(VoiceDropsEvent _, $Res Function(VoiceDropsEvent) __);
}


/// Adds pattern-matching-related methods to [VoiceDropsEvent].
extension VoiceDropsEventPatterns on VoiceDropsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( VoiceDropsPassRequestedEvent value)?  passRequested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case VoiceDropsPassRequestedEvent() when passRequested != null:
return passRequested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( VoiceDropsPassRequestedEvent value)  passRequested,}){
final _that = this;
switch (_that) {
case VoiceDropsPassRequestedEvent():
return passRequested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( VoiceDropsPassRequestedEvent value)?  passRequested,}){
final _that = this;
switch (_that) {
case VoiceDropsPassRequestedEvent() when passRequested != null:
return passRequested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  passRequested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case VoiceDropsPassRequestedEvent() when passRequested != null:
return passRequested();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  passRequested,}) {final _that = this;
switch (_that) {
case VoiceDropsPassRequestedEvent():
return passRequested();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  passRequested,}) {final _that = this;
switch (_that) {
case VoiceDropsPassRequestedEvent() when passRequested != null:
return passRequested();case _:
  return null;

}
}

}

/// @nodoc


class VoiceDropsPassRequestedEvent implements VoiceDropsEvent {
  const VoiceDropsPassRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceDropsPassRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'VoiceDropsEvent.passRequested()';
}


}




/// @nodoc
mixin _$VoiceDropsState {

/// Whether a pass is running.
 bool get working;/// How many recordings still wait (the phone can't transcribe them yet).
 int get waiting;/// The notes the last pass made.
 List<String> get saved;/// How many notes passes have made since the app started.
 int get savedCount;/// What was refused, a sentence each.
 List<String> get problems;/// The tidy choice the last pass read.
 VoiceTidyMode get tidyMode;
/// Create a copy of VoiceDropsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VoiceDropsStateCopyWith<VoiceDropsState> get copyWith => _$VoiceDropsStateCopyWithImpl<VoiceDropsState>(this as VoiceDropsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceDropsState&&(identical(other.working, working) || other.working == working)&&(identical(other.waiting, waiting) || other.waiting == waiting)&&const DeepCollectionEquality().equals(other.saved, saved)&&(identical(other.savedCount, savedCount) || other.savedCount == savedCount)&&const DeepCollectionEquality().equals(other.problems, problems)&&(identical(other.tidyMode, tidyMode) || other.tidyMode == tidyMode));
}


@override
int get hashCode => Object.hash(runtimeType,working,waiting,const DeepCollectionEquality().hash(saved),savedCount,const DeepCollectionEquality().hash(problems),tidyMode);

@override
String toString() {
  return 'VoiceDropsState(working: $working, waiting: $waiting, saved: $saved, savedCount: $savedCount, problems: $problems, tidyMode: $tidyMode)';
}


}

/// @nodoc
abstract mixin class $VoiceDropsStateCopyWith<$Res>  {
  factory $VoiceDropsStateCopyWith(VoiceDropsState value, $Res Function(VoiceDropsState) _then) = _$VoiceDropsStateCopyWithImpl;
@useResult
$Res call({
 bool working, int waiting, List<String> saved, int savedCount, List<String> problems, VoiceTidyMode tidyMode
});




}
/// @nodoc
class _$VoiceDropsStateCopyWithImpl<$Res>
    implements $VoiceDropsStateCopyWith<$Res> {
  _$VoiceDropsStateCopyWithImpl(this._self, this._then);

  final VoiceDropsState _self;
  final $Res Function(VoiceDropsState) _then;

/// Create a copy of VoiceDropsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? working = null,Object? waiting = null,Object? saved = null,Object? savedCount = null,Object? problems = null,Object? tidyMode = null,}) {
  return _then(_self.copyWith(
working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as bool,waiting: null == waiting ? _self.waiting : waiting // ignore: cast_nullable_to_non_nullable
as int,saved: null == saved ? _self.saved : saved // ignore: cast_nullable_to_non_nullable
as List<String>,savedCount: null == savedCount ? _self.savedCount : savedCount // ignore: cast_nullable_to_non_nullable
as int,problems: null == problems ? _self.problems : problems // ignore: cast_nullable_to_non_nullable
as List<String>,tidyMode: null == tidyMode ? _self.tidyMode : tidyMode // ignore: cast_nullable_to_non_nullable
as VoiceTidyMode,
  ));
}

}


/// Adds pattern-matching-related methods to [VoiceDropsState].
extension VoiceDropsStatePatterns on VoiceDropsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _VoiceDropsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _VoiceDropsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _VoiceDropsState value)  $default,){
final _that = this;
switch (_that) {
case _VoiceDropsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _VoiceDropsState value)?  $default,){
final _that = this;
switch (_that) {
case _VoiceDropsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool working,  int waiting,  List<String> saved,  int savedCount,  List<String> problems,  VoiceTidyMode tidyMode)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _VoiceDropsState() when $default != null:
return $default(_that.working,_that.waiting,_that.saved,_that.savedCount,_that.problems,_that.tidyMode);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool working,  int waiting,  List<String> saved,  int savedCount,  List<String> problems,  VoiceTidyMode tidyMode)  $default,) {final _that = this;
switch (_that) {
case _VoiceDropsState():
return $default(_that.working,_that.waiting,_that.saved,_that.savedCount,_that.problems,_that.tidyMode);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool working,  int waiting,  List<String> saved,  int savedCount,  List<String> problems,  VoiceTidyMode tidyMode)?  $default,) {final _that = this;
switch (_that) {
case _VoiceDropsState() when $default != null:
return $default(_that.working,_that.waiting,_that.saved,_that.savedCount,_that.problems,_that.tidyMode);case _:
  return null;

}
}

}

/// @nodoc


class _VoiceDropsState implements VoiceDropsState {
  const _VoiceDropsState({this.working = false, this.waiting = 0, final  List<String> saved = const <String>[], this.savedCount = 0, final  List<String> problems = const <String>[], this.tidyMode = VoiceTidyMode.ask}): _saved = saved,_problems = problems;
  

/// Whether a pass is running.
@override@JsonKey() final  bool working;
/// How many recordings still wait (the phone can't transcribe them yet).
@override@JsonKey() final  int waiting;
/// The notes the last pass made.
 final  List<String> _saved;
/// The notes the last pass made.
@override@JsonKey() List<String> get saved {
  if (_saved is EqualUnmodifiableListView) return _saved;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_saved);
}

/// How many notes passes have made since the app started.
@override@JsonKey() final  int savedCount;
/// What was refused, a sentence each.
 final  List<String> _problems;
/// What was refused, a sentence each.
@override@JsonKey() List<String> get problems {
  if (_problems is EqualUnmodifiableListView) return _problems;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_problems);
}

/// The tidy choice the last pass read.
@override@JsonKey() final  VoiceTidyMode tidyMode;

/// Create a copy of VoiceDropsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$VoiceDropsStateCopyWith<_VoiceDropsState> get copyWith => __$VoiceDropsStateCopyWithImpl<_VoiceDropsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _VoiceDropsState&&(identical(other.working, working) || other.working == working)&&(identical(other.waiting, waiting) || other.waiting == waiting)&&const DeepCollectionEquality().equals(other._saved, _saved)&&(identical(other.savedCount, savedCount) || other.savedCount == savedCount)&&const DeepCollectionEquality().equals(other._problems, _problems)&&(identical(other.tidyMode, tidyMode) || other.tidyMode == tidyMode));
}


@override
int get hashCode => Object.hash(runtimeType,working,waiting,const DeepCollectionEquality().hash(_saved),savedCount,const DeepCollectionEquality().hash(_problems),tidyMode);

@override
String toString() {
  return 'VoiceDropsState(working: $working, waiting: $waiting, saved: $saved, savedCount: $savedCount, problems: $problems, tidyMode: $tidyMode)';
}


}

/// @nodoc
abstract mixin class _$VoiceDropsStateCopyWith<$Res> implements $VoiceDropsStateCopyWith<$Res> {
  factory _$VoiceDropsStateCopyWith(_VoiceDropsState value, $Res Function(_VoiceDropsState) _then) = __$VoiceDropsStateCopyWithImpl;
@override @useResult
$Res call({
 bool working, int waiting, List<String> saved, int savedCount, List<String> problems, VoiceTidyMode tidyMode
});




}
/// @nodoc
class __$VoiceDropsStateCopyWithImpl<$Res>
    implements _$VoiceDropsStateCopyWith<$Res> {
  __$VoiceDropsStateCopyWithImpl(this._self, this._then);

  final _VoiceDropsState _self;
  final $Res Function(_VoiceDropsState) _then;

/// Create a copy of VoiceDropsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? working = null,Object? waiting = null,Object? saved = null,Object? savedCount = null,Object? problems = null,Object? tidyMode = null,}) {
  return _then(_VoiceDropsState(
working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as bool,waiting: null == waiting ? _self.waiting : waiting // ignore: cast_nullable_to_non_nullable
as int,saved: null == saved ? _self._saved : saved // ignore: cast_nullable_to_non_nullable
as List<String>,savedCount: null == savedCount ? _self.savedCount : savedCount // ignore: cast_nullable_to_non_nullable
as int,problems: null == problems ? _self._problems : problems // ignore: cast_nullable_to_non_nullable
as List<String>,tidyMode: null == tidyMode ? _self.tidyMode : tidyMode // ignore: cast_nullable_to_non_nullable
as VoiceTidyMode,
  ));
}


}

// dart format on
