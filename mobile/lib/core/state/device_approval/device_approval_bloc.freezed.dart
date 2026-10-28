// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'device_approval_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DeviceApprovalEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeviceApprovalEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DeviceApprovalEvent()';
}


}

/// @nodoc
class $DeviceApprovalEventCopyWith<$Res>  {
$DeviceApprovalEventCopyWith(DeviceApprovalEvent _, $Res Function(DeviceApprovalEvent) __);
}


/// Adds pattern-matching-related methods to [DeviceApprovalEvent].
extension DeviceApprovalEventPatterns on DeviceApprovalEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DeviceApprovalConditionsChangedEvent value)?  conditionsChanged,TResult Function( DeviceApprovalApprovedEvent value)?  approved,TResult Function( DeviceApprovalDeclinedEvent value)?  declined,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DeviceApprovalConditionsChangedEvent() when conditionsChanged != null:
return conditionsChanged(_that);case DeviceApprovalApprovedEvent() when approved != null:
return approved(_that);case DeviceApprovalDeclinedEvent() when declined != null:
return declined(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DeviceApprovalConditionsChangedEvent value)  conditionsChanged,required TResult Function( DeviceApprovalApprovedEvent value)  approved,required TResult Function( DeviceApprovalDeclinedEvent value)  declined,}){
final _that = this;
switch (_that) {
case DeviceApprovalConditionsChangedEvent():
return conditionsChanged(_that);case DeviceApprovalApprovedEvent():
return approved(_that);case DeviceApprovalDeclinedEvent():
return declined(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DeviceApprovalConditionsChangedEvent value)?  conditionsChanged,TResult? Function( DeviceApprovalApprovedEvent value)?  approved,TResult? Function( DeviceApprovalDeclinedEvent value)?  declined,}){
final _that = this;
switch (_that) {
case DeviceApprovalConditionsChangedEvent() when conditionsChanged != null:
return conditionsChanged(_that);case DeviceApprovalApprovedEvent() when approved != null:
return approved(_that);case DeviceApprovalDeclinedEvent() when declined != null:
return declined(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( bool signedIn,  bool foreground)?  conditionsChanged,TResult Function( String id)?  approved,TResult Function( String id)?  declined,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DeviceApprovalConditionsChangedEvent() when conditionsChanged != null:
return conditionsChanged(_that.signedIn,_that.foreground);case DeviceApprovalApprovedEvent() when approved != null:
return approved(_that.id);case DeviceApprovalDeclinedEvent() when declined != null:
return declined(_that.id);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( bool signedIn,  bool foreground)  conditionsChanged,required TResult Function( String id)  approved,required TResult Function( String id)  declined,}) {final _that = this;
switch (_that) {
case DeviceApprovalConditionsChangedEvent():
return conditionsChanged(_that.signedIn,_that.foreground);case DeviceApprovalApprovedEvent():
return approved(_that.id);case DeviceApprovalDeclinedEvent():
return declined(_that.id);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( bool signedIn,  bool foreground)?  conditionsChanged,TResult? Function( String id)?  approved,TResult? Function( String id)?  declined,}) {final _that = this;
switch (_that) {
case DeviceApprovalConditionsChangedEvent() when conditionsChanged != null:
return conditionsChanged(_that.signedIn,_that.foreground);case DeviceApprovalApprovedEvent() when approved != null:
return approved(_that.id);case DeviceApprovalDeclinedEvent() when declined != null:
return declined(_that.id);case _:
  return null;

}
}

}

/// @nodoc


class DeviceApprovalConditionsChangedEvent implements DeviceApprovalEvent {
  const DeviceApprovalConditionsChangedEvent({required this.signedIn, required this.foreground});
  

 final  bool signedIn;
 final  bool foreground;

/// Create a copy of DeviceApprovalEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DeviceApprovalConditionsChangedEventCopyWith<DeviceApprovalConditionsChangedEvent> get copyWith => _$DeviceApprovalConditionsChangedEventCopyWithImpl<DeviceApprovalConditionsChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeviceApprovalConditionsChangedEvent&&(identical(other.signedIn, signedIn) || other.signedIn == signedIn)&&(identical(other.foreground, foreground) || other.foreground == foreground));
}


@override
int get hashCode => Object.hash(runtimeType,signedIn,foreground);

@override
String toString() {
  return 'DeviceApprovalEvent.conditionsChanged(signedIn: $signedIn, foreground: $foreground)';
}


}

/// @nodoc
abstract mixin class $DeviceApprovalConditionsChangedEventCopyWith<$Res> implements $DeviceApprovalEventCopyWith<$Res> {
  factory $DeviceApprovalConditionsChangedEventCopyWith(DeviceApprovalConditionsChangedEvent value, $Res Function(DeviceApprovalConditionsChangedEvent) _then) = _$DeviceApprovalConditionsChangedEventCopyWithImpl;
@useResult
$Res call({
 bool signedIn, bool foreground
});




}
/// @nodoc
class _$DeviceApprovalConditionsChangedEventCopyWithImpl<$Res>
    implements $DeviceApprovalConditionsChangedEventCopyWith<$Res> {
  _$DeviceApprovalConditionsChangedEventCopyWithImpl(this._self, this._then);

  final DeviceApprovalConditionsChangedEvent _self;
  final $Res Function(DeviceApprovalConditionsChangedEvent) _then;

/// Create a copy of DeviceApprovalEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? signedIn = null,Object? foreground = null,}) {
  return _then(DeviceApprovalConditionsChangedEvent(
signedIn: null == signedIn ? _self.signedIn : signedIn // ignore: cast_nullable_to_non_nullable
as bool,foreground: null == foreground ? _self.foreground : foreground // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class DeviceApprovalApprovedEvent implements DeviceApprovalEvent {
  const DeviceApprovalApprovedEvent(this.id);
  

 final  String id;

/// Create a copy of DeviceApprovalEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DeviceApprovalApprovedEventCopyWith<DeviceApprovalApprovedEvent> get copyWith => _$DeviceApprovalApprovedEventCopyWithImpl<DeviceApprovalApprovedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeviceApprovalApprovedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'DeviceApprovalEvent.approved(id: $id)';
}


}

/// @nodoc
abstract mixin class $DeviceApprovalApprovedEventCopyWith<$Res> implements $DeviceApprovalEventCopyWith<$Res> {
  factory $DeviceApprovalApprovedEventCopyWith(DeviceApprovalApprovedEvent value, $Res Function(DeviceApprovalApprovedEvent) _then) = _$DeviceApprovalApprovedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$DeviceApprovalApprovedEventCopyWithImpl<$Res>
    implements $DeviceApprovalApprovedEventCopyWith<$Res> {
  _$DeviceApprovalApprovedEventCopyWithImpl(this._self, this._then);

  final DeviceApprovalApprovedEvent _self;
  final $Res Function(DeviceApprovalApprovedEvent) _then;

/// Create a copy of DeviceApprovalEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(DeviceApprovalApprovedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class DeviceApprovalDeclinedEvent implements DeviceApprovalEvent {
  const DeviceApprovalDeclinedEvent(this.id);
  

 final  String id;

/// Create a copy of DeviceApprovalEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DeviceApprovalDeclinedEventCopyWith<DeviceApprovalDeclinedEvent> get copyWith => _$DeviceApprovalDeclinedEventCopyWithImpl<DeviceApprovalDeclinedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeviceApprovalDeclinedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'DeviceApprovalEvent.declined(id: $id)';
}


}

/// @nodoc
abstract mixin class $DeviceApprovalDeclinedEventCopyWith<$Res> implements $DeviceApprovalEventCopyWith<$Res> {
  factory $DeviceApprovalDeclinedEventCopyWith(DeviceApprovalDeclinedEvent value, $Res Function(DeviceApprovalDeclinedEvent) _then) = _$DeviceApprovalDeclinedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$DeviceApprovalDeclinedEventCopyWithImpl<$Res>
    implements $DeviceApprovalDeclinedEventCopyWith<$Res> {
  _$DeviceApprovalDeclinedEventCopyWithImpl(this._self, this._then);

  final DeviceApprovalDeclinedEvent _self;
  final $Res Function(DeviceApprovalDeclinedEvent) _then;

/// Create a copy of DeviceApprovalEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(DeviceApprovalDeclinedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$DeviceApprovalState {

/// The requests waiting, oldest first.
 List<ApprovalPrompt> get prompts;/// The request being answered, if one is.
 String? get answering;/// Why the last answer failed.
 ApplicationError? get error;
/// Create a copy of DeviceApprovalState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DeviceApprovalStateCopyWith<DeviceApprovalState> get copyWith => _$DeviceApprovalStateCopyWithImpl<DeviceApprovalState>(this as DeviceApprovalState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DeviceApprovalState&&const DeepCollectionEquality().equals(other.prompts, prompts)&&(identical(other.answering, answering) || other.answering == answering)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(prompts),answering,error);

@override
String toString() {
  return 'DeviceApprovalState(prompts: $prompts, answering: $answering, error: $error)';
}


}

/// @nodoc
abstract mixin class $DeviceApprovalStateCopyWith<$Res>  {
  factory $DeviceApprovalStateCopyWith(DeviceApprovalState value, $Res Function(DeviceApprovalState) _then) = _$DeviceApprovalStateCopyWithImpl;
@useResult
$Res call({
 List<ApprovalPrompt> prompts, String? answering, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$DeviceApprovalStateCopyWithImpl<$Res>
    implements $DeviceApprovalStateCopyWith<$Res> {
  _$DeviceApprovalStateCopyWithImpl(this._self, this._then);

  final DeviceApprovalState _self;
  final $Res Function(DeviceApprovalState) _then;

/// Create a copy of DeviceApprovalState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? prompts = null,Object? answering = freezed,Object? error = freezed,}) {
  return _then(_self.copyWith(
prompts: null == prompts ? _self.prompts : prompts // ignore: cast_nullable_to_non_nullable
as List<ApprovalPrompt>,answering: freezed == answering ? _self.answering : answering // ignore: cast_nullable_to_non_nullable
as String?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of DeviceApprovalState
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


/// Adds pattern-matching-related methods to [DeviceApprovalState].
extension DeviceApprovalStatePatterns on DeviceApprovalState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DeviceApprovalState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DeviceApprovalState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DeviceApprovalState value)  $default,){
final _that = this;
switch (_that) {
case _DeviceApprovalState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DeviceApprovalState value)?  $default,){
final _that = this;
switch (_that) {
case _DeviceApprovalState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<ApprovalPrompt> prompts,  String? answering,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DeviceApprovalState() when $default != null:
return $default(_that.prompts,_that.answering,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<ApprovalPrompt> prompts,  String? answering,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _DeviceApprovalState():
return $default(_that.prompts,_that.answering,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<ApprovalPrompt> prompts,  String? answering,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _DeviceApprovalState() when $default != null:
return $default(_that.prompts,_that.answering,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _DeviceApprovalState extends DeviceApprovalState {
  const _DeviceApprovalState({final  List<ApprovalPrompt> prompts = const <ApprovalPrompt>[], this.answering, this.error}): _prompts = prompts,super._();
  

/// The requests waiting, oldest first.
 final  List<ApprovalPrompt> _prompts;
/// The requests waiting, oldest first.
@override@JsonKey() List<ApprovalPrompt> get prompts {
  if (_prompts is EqualUnmodifiableListView) return _prompts;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_prompts);
}

/// The request being answered, if one is.
@override final  String? answering;
/// Why the last answer failed.
@override final  ApplicationError? error;

/// Create a copy of DeviceApprovalState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DeviceApprovalStateCopyWith<_DeviceApprovalState> get copyWith => __$DeviceApprovalStateCopyWithImpl<_DeviceApprovalState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DeviceApprovalState&&const DeepCollectionEquality().equals(other._prompts, _prompts)&&(identical(other.answering, answering) || other.answering == answering)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_prompts),answering,error);

@override
String toString() {
  return 'DeviceApprovalState(prompts: $prompts, answering: $answering, error: $error)';
}


}

/// @nodoc
abstract mixin class _$DeviceApprovalStateCopyWith<$Res> implements $DeviceApprovalStateCopyWith<$Res> {
  factory _$DeviceApprovalStateCopyWith(_DeviceApprovalState value, $Res Function(_DeviceApprovalState) _then) = __$DeviceApprovalStateCopyWithImpl;
@override @useResult
$Res call({
 List<ApprovalPrompt> prompts, String? answering, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$DeviceApprovalStateCopyWithImpl<$Res>
    implements _$DeviceApprovalStateCopyWith<$Res> {
  __$DeviceApprovalStateCopyWithImpl(this._self, this._then);

  final _DeviceApprovalState _self;
  final $Res Function(_DeviceApprovalState) _then;

/// Create a copy of DeviceApprovalState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? prompts = null,Object? answering = freezed,Object? error = freezed,}) {
  return _then(_DeviceApprovalState(
prompts: null == prompts ? _self._prompts : prompts // ignore: cast_nullable_to_non_nullable
as List<ApprovalPrompt>,answering: freezed == answering ? _self.answering : answering // ignore: cast_nullable_to_non_nullable
as String?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of DeviceApprovalState
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
