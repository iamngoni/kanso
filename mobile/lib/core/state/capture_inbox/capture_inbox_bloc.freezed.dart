// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'capture_inbox_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CaptureInboxEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CaptureInboxEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CaptureInboxEvent()';
}


}

/// @nodoc
class $CaptureInboxEventCopyWith<$Res>  {
$CaptureInboxEventCopyWith(CaptureInboxEvent _, $Res Function(CaptureInboxEvent) __);
}


/// Adds pattern-matching-related methods to [CaptureInboxEvent].
extension CaptureInboxEventPatterns on CaptureInboxEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CaptureInboxSaveRequestedEvent value)?  saveRequested,TResult Function( CaptureInboxProblemsDismissedEvent value)?  problemsDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CaptureInboxSaveRequestedEvent() when saveRequested != null:
return saveRequested(_that);case CaptureInboxProblemsDismissedEvent() when problemsDismissed != null:
return problemsDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CaptureInboxSaveRequestedEvent value)  saveRequested,required TResult Function( CaptureInboxProblemsDismissedEvent value)  problemsDismissed,}){
final _that = this;
switch (_that) {
case CaptureInboxSaveRequestedEvent():
return saveRequested(_that);case CaptureInboxProblemsDismissedEvent():
return problemsDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CaptureInboxSaveRequestedEvent value)?  saveRequested,TResult? Function( CaptureInboxProblemsDismissedEvent value)?  problemsDismissed,}){
final _that = this;
switch (_that) {
case CaptureInboxSaveRequestedEvent() when saveRequested != null:
return saveRequested(_that);case CaptureInboxProblemsDismissedEvent() when problemsDismissed != null:
return problemsDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  saveRequested,TResult Function()?  problemsDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CaptureInboxSaveRequestedEvent() when saveRequested != null:
return saveRequested();case CaptureInboxProblemsDismissedEvent() when problemsDismissed != null:
return problemsDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  saveRequested,required TResult Function()  problemsDismissed,}) {final _that = this;
switch (_that) {
case CaptureInboxSaveRequestedEvent():
return saveRequested();case CaptureInboxProblemsDismissedEvent():
return problemsDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  saveRequested,TResult? Function()?  problemsDismissed,}) {final _that = this;
switch (_that) {
case CaptureInboxSaveRequestedEvent() when saveRequested != null:
return saveRequested();case CaptureInboxProblemsDismissedEvent() when problemsDismissed != null:
return problemsDismissed();case _:
  return null;

}
}

}

/// @nodoc


class CaptureInboxSaveRequestedEvent implements CaptureInboxEvent {
  const CaptureInboxSaveRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CaptureInboxSaveRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CaptureInboxEvent.saveRequested()';
}


}




/// @nodoc


class CaptureInboxProblemsDismissedEvent implements CaptureInboxEvent {
  const CaptureInboxProblemsDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CaptureInboxProblemsDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CaptureInboxEvent.problemsDismissed()';
}


}




/// @nodoc
mixin _$CaptureInboxState {

/// Whether a save is running.
 bool get saving;/// The IDs of the notes the last save made.
 List<String> get saved;/// How many notes saves have made since the app started.
 int get savedCount;/// How many items the last save kept to try again.
 int get kept;/// What was left out or refused since the person last dismissed them, a
/// sentence each, for the screen to show as a short notice.
 List<String> get problems;/// Why the last save failed; null once one works.
 ApplicationError? get error;
/// Create a copy of CaptureInboxState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CaptureInboxStateCopyWith<CaptureInboxState> get copyWith => _$CaptureInboxStateCopyWithImpl<CaptureInboxState>(this as CaptureInboxState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CaptureInboxState&&(identical(other.saving, saving) || other.saving == saving)&&const DeepCollectionEquality().equals(other.saved, saved)&&(identical(other.savedCount, savedCount) || other.savedCount == savedCount)&&(identical(other.kept, kept) || other.kept == kept)&&const DeepCollectionEquality().equals(other.problems, problems)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,saving,const DeepCollectionEquality().hash(saved),savedCount,kept,const DeepCollectionEquality().hash(problems),error);

@override
String toString() {
  return 'CaptureInboxState(saving: $saving, saved: $saved, savedCount: $savedCount, kept: $kept, problems: $problems, error: $error)';
}


}

/// @nodoc
abstract mixin class $CaptureInboxStateCopyWith<$Res>  {
  factory $CaptureInboxStateCopyWith(CaptureInboxState value, $Res Function(CaptureInboxState) _then) = _$CaptureInboxStateCopyWithImpl;
@useResult
$Res call({
 bool saving, List<String> saved, int savedCount, int kept, List<String> problems, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$CaptureInboxStateCopyWithImpl<$Res>
    implements $CaptureInboxStateCopyWith<$Res> {
  _$CaptureInboxStateCopyWithImpl(this._self, this._then);

  final CaptureInboxState _self;
  final $Res Function(CaptureInboxState) _then;

/// Create a copy of CaptureInboxState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? saving = null,Object? saved = null,Object? savedCount = null,Object? kept = null,Object? problems = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
saving: null == saving ? _self.saving : saving // ignore: cast_nullable_to_non_nullable
as bool,saved: null == saved ? _self.saved : saved // ignore: cast_nullable_to_non_nullable
as List<String>,savedCount: null == savedCount ? _self.savedCount : savedCount // ignore: cast_nullable_to_non_nullable
as int,kept: null == kept ? _self.kept : kept // ignore: cast_nullable_to_non_nullable
as int,problems: null == problems ? _self.problems : problems // ignore: cast_nullable_to_non_nullable
as List<String>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of CaptureInboxState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}


/// Adds pattern-matching-related methods to [CaptureInboxState].
extension CaptureInboxStatePatterns on CaptureInboxState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CaptureInboxState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CaptureInboxState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CaptureInboxState value)  $default,){
final _that = this;
switch (_that) {
case _CaptureInboxState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CaptureInboxState value)?  $default,){
final _that = this;
switch (_that) {
case _CaptureInboxState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool saving,  List<String> saved,  int savedCount,  int kept,  List<String> problems,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CaptureInboxState() when $default != null:
return $default(_that.saving,_that.saved,_that.savedCount,_that.kept,_that.problems,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool saving,  List<String> saved,  int savedCount,  int kept,  List<String> problems,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _CaptureInboxState():
return $default(_that.saving,_that.saved,_that.savedCount,_that.kept,_that.problems,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool saving,  List<String> saved,  int savedCount,  int kept,  List<String> problems,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _CaptureInboxState() when $default != null:
return $default(_that.saving,_that.saved,_that.savedCount,_that.kept,_that.problems,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _CaptureInboxState implements CaptureInboxState {
  const _CaptureInboxState({this.saving = false, final  List<String> saved = const <String>[], this.savedCount = 0, this.kept = 0, final  List<String> problems = const <String>[], this.error}): _saved = saved,_problems = problems;
  

/// Whether a save is running.
@override@JsonKey() final  bool saving;
/// The IDs of the notes the last save made.
 final  List<String> _saved;
/// The IDs of the notes the last save made.
@override@JsonKey() List<String> get saved {
  if (_saved is EqualUnmodifiableListView) return _saved;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_saved);
}

/// How many notes saves have made since the app started.
@override@JsonKey() final  int savedCount;
/// How many items the last save kept to try again.
@override@JsonKey() final  int kept;
/// What was left out or refused since the person last dismissed them, a
/// sentence each, for the screen to show as a short notice.
 final  List<String> _problems;
/// What was left out or refused since the person last dismissed them, a
/// sentence each, for the screen to show as a short notice.
@override@JsonKey() List<String> get problems {
  if (_problems is EqualUnmodifiableListView) return _problems;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_problems);
}

/// Why the last save failed; null once one works.
@override final  ApplicationError? error;

/// Create a copy of CaptureInboxState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CaptureInboxStateCopyWith<_CaptureInboxState> get copyWith => __$CaptureInboxStateCopyWithImpl<_CaptureInboxState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CaptureInboxState&&(identical(other.saving, saving) || other.saving == saving)&&const DeepCollectionEquality().equals(other._saved, _saved)&&(identical(other.savedCount, savedCount) || other.savedCount == savedCount)&&(identical(other.kept, kept) || other.kept == kept)&&const DeepCollectionEquality().equals(other._problems, _problems)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,saving,const DeepCollectionEquality().hash(_saved),savedCount,kept,const DeepCollectionEquality().hash(_problems),error);

@override
String toString() {
  return 'CaptureInboxState(saving: $saving, saved: $saved, savedCount: $savedCount, kept: $kept, problems: $problems, error: $error)';
}


}

/// @nodoc
abstract mixin class _$CaptureInboxStateCopyWith<$Res> implements $CaptureInboxStateCopyWith<$Res> {
  factory _$CaptureInboxStateCopyWith(_CaptureInboxState value, $Res Function(_CaptureInboxState) _then) = __$CaptureInboxStateCopyWithImpl;
@override @useResult
$Res call({
 bool saving, List<String> saved, int savedCount, int kept, List<String> problems, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$CaptureInboxStateCopyWithImpl<$Res>
    implements _$CaptureInboxStateCopyWith<$Res> {
  __$CaptureInboxStateCopyWithImpl(this._self, this._then);

  final _CaptureInboxState _self;
  final $Res Function(_CaptureInboxState) _then;

/// Create a copy of CaptureInboxState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? saving = null,Object? saved = null,Object? savedCount = null,Object? kept = null,Object? problems = null,Object? error = freezed,}) {
  return _then(_CaptureInboxState(
saving: null == saving ? _self.saving : saving // ignore: cast_nullable_to_non_nullable
as bool,saved: null == saved ? _self._saved : saved // ignore: cast_nullable_to_non_nullable
as List<String>,savedCount: null == savedCount ? _self.savedCount : savedCount // ignore: cast_nullable_to_non_nullable
as int,kept: null == kept ? _self.kept : kept // ignore: cast_nullable_to_non_nullable
as int,problems: null == problems ? _self._problems : problems // ignore: cast_nullable_to_non_nullable
as List<String>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of CaptureInboxState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}

// dart format on
