// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'word_goal_settings_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$WordGoalSettingsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WordGoalSettingsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WordGoalSettingsEvent()';
}


}




/// Adds pattern-matching-related methods to [WordGoalSettingsEvent].
extension WordGoalSettingsEventPatterns on WordGoalSettingsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WordGoalSettingsStartedEvent value)?  started,TResult Function( WordGoalDefaultChangedEvent value)?  defaultChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WordGoalSettingsStartedEvent() when started != null:
return started(_that);case WordGoalDefaultChangedEvent() when defaultChanged != null:
return defaultChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WordGoalSettingsStartedEvent value)  started,required TResult Function( WordGoalDefaultChangedEvent value)  defaultChanged,}){
final _that = this;
switch (_that) {
case WordGoalSettingsStartedEvent():
return started(_that);case WordGoalDefaultChangedEvent():
return defaultChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WordGoalSettingsStartedEvent value)?  started,TResult? Function( WordGoalDefaultChangedEvent value)?  defaultChanged,}){
final _that = this;
switch (_that) {
case WordGoalSettingsStartedEvent() when started != null:
return started(_that);case WordGoalDefaultChangedEvent() when defaultChanged != null:
return defaultChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( int? words)?  defaultChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WordGoalSettingsStartedEvent() when started != null:
return started();case WordGoalDefaultChangedEvent() when defaultChanged != null:
return defaultChanged(_that.words);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( int? words)  defaultChanged,}) {final _that = this;
switch (_that) {
case WordGoalSettingsStartedEvent():
return started();case WordGoalDefaultChangedEvent():
return defaultChanged(_that.words);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( int? words)?  defaultChanged,}) {final _that = this;
switch (_that) {
case WordGoalSettingsStartedEvent() when started != null:
return started();case WordGoalDefaultChangedEvent() when defaultChanged != null:
return defaultChanged(_that.words);case _:
  return null;

}
}

}

/// @nodoc


class WordGoalSettingsStartedEvent implements WordGoalSettingsEvent {
  const WordGoalSettingsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WordGoalSettingsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'WordGoalSettingsEvent.started()';
}


}




/// @nodoc


class WordGoalDefaultChangedEvent implements WordGoalSettingsEvent {
  const WordGoalDefaultChangedEvent(this.words);
  

 final  int? words;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WordGoalDefaultChangedEvent&&(identical(other.words, words) || other.words == words));
}


@override
int get hashCode => Object.hash(runtimeType,words);

@override
String toString() {
  return 'WordGoalSettingsEvent.defaultChanged(words: $words)';
}


}




/// @nodoc
mixin _$WordGoalSettingsState {

 LoadStatus get status; int? get words; ApplicationError? get error;
/// Create a copy of WordGoalSettingsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WordGoalSettingsStateCopyWith<WordGoalSettingsState> get copyWith => _$WordGoalSettingsStateCopyWithImpl<WordGoalSettingsState>(this as WordGoalSettingsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WordGoalSettingsState&&(identical(other.status, status) || other.status == status)&&(identical(other.words, words) || other.words == words)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,words,error);

@override
String toString() {
  return 'WordGoalSettingsState(status: $status, words: $words, error: $error)';
}


}

/// @nodoc
abstract mixin class $WordGoalSettingsStateCopyWith<$Res>  {
  factory $WordGoalSettingsStateCopyWith(WordGoalSettingsState value, $Res Function(WordGoalSettingsState) _then) = _$WordGoalSettingsStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, int? words, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$WordGoalSettingsStateCopyWithImpl<$Res>
    implements $WordGoalSettingsStateCopyWith<$Res> {
  _$WordGoalSettingsStateCopyWithImpl(this._self, this._then);

  final WordGoalSettingsState _self;
  final $Res Function(WordGoalSettingsState) _then;

/// Create a copy of WordGoalSettingsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? words = freezed,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,words: freezed == words ? _self.words : words // ignore: cast_nullable_to_non_nullable
as int?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of WordGoalSettingsState
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


/// Adds pattern-matching-related methods to [WordGoalSettingsState].
extension WordGoalSettingsStatePatterns on WordGoalSettingsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _WordGoalSettingsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _WordGoalSettingsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _WordGoalSettingsState value)  $default,){
final _that = this;
switch (_that) {
case _WordGoalSettingsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _WordGoalSettingsState value)?  $default,){
final _that = this;
switch (_that) {
case _WordGoalSettingsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  int? words,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _WordGoalSettingsState() when $default != null:
return $default(_that.status,_that.words,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  int? words,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _WordGoalSettingsState():
return $default(_that.status,_that.words,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  int? words,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _WordGoalSettingsState() when $default != null:
return $default(_that.status,_that.words,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _WordGoalSettingsState implements WordGoalSettingsState {
  const _WordGoalSettingsState({this.status = LoadStatus.loading, this.words, this.error});
  

@override@JsonKey() final  LoadStatus status;
@override final  int? words;
@override final  ApplicationError? error;

/// Create a copy of WordGoalSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$WordGoalSettingsStateCopyWith<_WordGoalSettingsState> get copyWith => __$WordGoalSettingsStateCopyWithImpl<_WordGoalSettingsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _WordGoalSettingsState&&(identical(other.status, status) || other.status == status)&&(identical(other.words, words) || other.words == words)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,words,error);

@override
String toString() {
  return 'WordGoalSettingsState(status: $status, words: $words, error: $error)';
}


}

/// @nodoc
abstract mixin class _$WordGoalSettingsStateCopyWith<$Res> implements $WordGoalSettingsStateCopyWith<$Res> {
  factory _$WordGoalSettingsStateCopyWith(_WordGoalSettingsState value, $Res Function(_WordGoalSettingsState) _then) = __$WordGoalSettingsStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, int? words, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$WordGoalSettingsStateCopyWithImpl<$Res>
    implements _$WordGoalSettingsStateCopyWith<$Res> {
  __$WordGoalSettingsStateCopyWithImpl(this._self, this._then);

  final _WordGoalSettingsState _self;
  final $Res Function(_WordGoalSettingsState) _then;

/// Create a copy of WordGoalSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? words = freezed,Object? error = freezed,}) {
  return _then(_WordGoalSettingsState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,words: freezed == words ? _self.words : words // ignore: cast_nullable_to_non_nullable
as int?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of WordGoalSettingsState
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
