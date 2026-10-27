// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'sync_settings_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SyncSettingsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncSettingsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SyncSettingsEvent()';
}


}

/// @nodoc
class $SyncSettingsEventCopyWith<$Res>  {
$SyncSettingsEventCopyWith(SyncSettingsEvent _, $Res Function(SyncSettingsEvent) __);
}


/// Adds pattern-matching-related methods to [SyncSettingsEvent].
extension SyncSettingsEventPatterns on SyncSettingsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SyncSettingsStartedEvent value)?  started,TResult Function( SyncSettingsEnabledChangedEvent value)?  enabledChanged,TResult Function( SyncSettingsIntervalChosenEvent value)?  intervalChosen,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SyncSettingsStartedEvent() when started != null:
return started(_that);case SyncSettingsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that);case SyncSettingsIntervalChosenEvent() when intervalChosen != null:
return intervalChosen(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SyncSettingsStartedEvent value)  started,required TResult Function( SyncSettingsEnabledChangedEvent value)  enabledChanged,required TResult Function( SyncSettingsIntervalChosenEvent value)  intervalChosen,}){
final _that = this;
switch (_that) {
case SyncSettingsStartedEvent():
return started(_that);case SyncSettingsEnabledChangedEvent():
return enabledChanged(_that);case SyncSettingsIntervalChosenEvent():
return intervalChosen(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SyncSettingsStartedEvent value)?  started,TResult? Function( SyncSettingsEnabledChangedEvent value)?  enabledChanged,TResult? Function( SyncSettingsIntervalChosenEvent value)?  intervalChosen,}){
final _that = this;
switch (_that) {
case SyncSettingsStartedEvent() when started != null:
return started(_that);case SyncSettingsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that);case SyncSettingsIntervalChosenEvent() when intervalChosen != null:
return intervalChosen(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( bool enabled)?  enabledChanged,TResult Function( int minutes)?  intervalChosen,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SyncSettingsStartedEvent() when started != null:
return started();case SyncSettingsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that.enabled);case SyncSettingsIntervalChosenEvent() when intervalChosen != null:
return intervalChosen(_that.minutes);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( bool enabled)  enabledChanged,required TResult Function( int minutes)  intervalChosen,}) {final _that = this;
switch (_that) {
case SyncSettingsStartedEvent():
return started();case SyncSettingsEnabledChangedEvent():
return enabledChanged(_that.enabled);case SyncSettingsIntervalChosenEvent():
return intervalChosen(_that.minutes);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( bool enabled)?  enabledChanged,TResult? Function( int minutes)?  intervalChosen,}) {final _that = this;
switch (_that) {
case SyncSettingsStartedEvent() when started != null:
return started();case SyncSettingsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that.enabled);case SyncSettingsIntervalChosenEvent() when intervalChosen != null:
return intervalChosen(_that.minutes);case _:
  return null;

}
}

}

/// @nodoc


class SyncSettingsStartedEvent implements SyncSettingsEvent {
  const SyncSettingsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncSettingsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SyncSettingsEvent.started()';
}


}




/// @nodoc


class SyncSettingsEnabledChangedEvent implements SyncSettingsEvent {
  const SyncSettingsEnabledChangedEvent({required this.enabled});
  

 final  bool enabled;

/// Create a copy of SyncSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SyncSettingsEnabledChangedEventCopyWith<SyncSettingsEnabledChangedEvent> get copyWith => _$SyncSettingsEnabledChangedEventCopyWithImpl<SyncSettingsEnabledChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncSettingsEnabledChangedEvent&&(identical(other.enabled, enabled) || other.enabled == enabled));
}


@override
int get hashCode => Object.hash(runtimeType,enabled);

@override
String toString() {
  return 'SyncSettingsEvent.enabledChanged(enabled: $enabled)';
}


}

/// @nodoc
abstract mixin class $SyncSettingsEnabledChangedEventCopyWith<$Res> implements $SyncSettingsEventCopyWith<$Res> {
  factory $SyncSettingsEnabledChangedEventCopyWith(SyncSettingsEnabledChangedEvent value, $Res Function(SyncSettingsEnabledChangedEvent) _then) = _$SyncSettingsEnabledChangedEventCopyWithImpl;
@useResult
$Res call({
 bool enabled
});




}
/// @nodoc
class _$SyncSettingsEnabledChangedEventCopyWithImpl<$Res>
    implements $SyncSettingsEnabledChangedEventCopyWith<$Res> {
  _$SyncSettingsEnabledChangedEventCopyWithImpl(this._self, this._then);

  final SyncSettingsEnabledChangedEvent _self;
  final $Res Function(SyncSettingsEnabledChangedEvent) _then;

/// Create a copy of SyncSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? enabled = null,}) {
  return _then(SyncSettingsEnabledChangedEvent(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class SyncSettingsIntervalChosenEvent implements SyncSettingsEvent {
  const SyncSettingsIntervalChosenEvent(this.minutes);
  

 final  int minutes;

/// Create a copy of SyncSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SyncSettingsIntervalChosenEventCopyWith<SyncSettingsIntervalChosenEvent> get copyWith => _$SyncSettingsIntervalChosenEventCopyWithImpl<SyncSettingsIntervalChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncSettingsIntervalChosenEvent&&(identical(other.minutes, minutes) || other.minutes == minutes));
}


@override
int get hashCode => Object.hash(runtimeType,minutes);

@override
String toString() {
  return 'SyncSettingsEvent.intervalChosen(minutes: $minutes)';
}


}

/// @nodoc
abstract mixin class $SyncSettingsIntervalChosenEventCopyWith<$Res> implements $SyncSettingsEventCopyWith<$Res> {
  factory $SyncSettingsIntervalChosenEventCopyWith(SyncSettingsIntervalChosenEvent value, $Res Function(SyncSettingsIntervalChosenEvent) _then) = _$SyncSettingsIntervalChosenEventCopyWithImpl;
@useResult
$Res call({
 int minutes
});




}
/// @nodoc
class _$SyncSettingsIntervalChosenEventCopyWithImpl<$Res>
    implements $SyncSettingsIntervalChosenEventCopyWith<$Res> {
  _$SyncSettingsIntervalChosenEventCopyWithImpl(this._self, this._then);

  final SyncSettingsIntervalChosenEvent _self;
  final $Res Function(SyncSettingsIntervalChosenEvent) _then;

/// Create a copy of SyncSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? minutes = null,}) {
  return _then(SyncSettingsIntervalChosenEvent(
null == minutes ? _self.minutes : minutes // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc
mixin _$SyncSettingsState {

 LoadStatus get status;/// The settings as shown; null until they are read.
 SyncSettingsRow? get settings;/// How often sync may run, in the desktop's words.
 List<SyncIntervalOption> get options;/// Why the settings could not be read or the last change saved.
 ApplicationError? get error;
/// Create a copy of SyncSettingsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SyncSettingsStateCopyWith<SyncSettingsState> get copyWith => _$SyncSettingsStateCopyWithImpl<SyncSettingsState>(this as SyncSettingsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncSettingsState&&(identical(other.status, status) || other.status == status)&&(identical(other.settings, settings) || other.settings == settings)&&const DeepCollectionEquality().equals(other.options, options)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,settings,const DeepCollectionEquality().hash(options),error);

@override
String toString() {
  return 'SyncSettingsState(status: $status, settings: $settings, options: $options, error: $error)';
}


}

/// @nodoc
abstract mixin class $SyncSettingsStateCopyWith<$Res>  {
  factory $SyncSettingsStateCopyWith(SyncSettingsState value, $Res Function(SyncSettingsState) _then) = _$SyncSettingsStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, SyncSettingsRow? settings, List<SyncIntervalOption> options, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$SyncSettingsStateCopyWithImpl<$Res>
    implements $SyncSettingsStateCopyWith<$Res> {
  _$SyncSettingsStateCopyWithImpl(this._self, this._then);

  final SyncSettingsState _self;
  final $Res Function(SyncSettingsState) _then;

/// Create a copy of SyncSettingsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? settings = freezed,Object? options = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,settings: freezed == settings ? _self.settings : settings // ignore: cast_nullable_to_non_nullable
as SyncSettingsRow?,options: null == options ? _self.options : options // ignore: cast_nullable_to_non_nullable
as List<SyncIntervalOption>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of SyncSettingsState
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


/// Adds pattern-matching-related methods to [SyncSettingsState].
extension SyncSettingsStatePatterns on SyncSettingsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SyncSettingsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SyncSettingsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SyncSettingsState value)  $default,){
final _that = this;
switch (_that) {
case _SyncSettingsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SyncSettingsState value)?  $default,){
final _that = this;
switch (_that) {
case _SyncSettingsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  SyncSettingsRow? settings,  List<SyncIntervalOption> options,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SyncSettingsState() when $default != null:
return $default(_that.status,_that.settings,_that.options,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  SyncSettingsRow? settings,  List<SyncIntervalOption> options,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _SyncSettingsState():
return $default(_that.status,_that.settings,_that.options,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  SyncSettingsRow? settings,  List<SyncIntervalOption> options,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _SyncSettingsState() when $default != null:
return $default(_that.status,_that.settings,_that.options,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _SyncSettingsState extends SyncSettingsState {
  const _SyncSettingsState({this.status = LoadStatus.loading, this.settings, final  List<SyncIntervalOption> options = const <SyncIntervalOption>[], this.error}): _options = options,super._();
  

@override@JsonKey() final  LoadStatus status;
/// The settings as shown; null until they are read.
@override final  SyncSettingsRow? settings;
/// How often sync may run, in the desktop's words.
 final  List<SyncIntervalOption> _options;
/// How often sync may run, in the desktop's words.
@override@JsonKey() List<SyncIntervalOption> get options {
  if (_options is EqualUnmodifiableListView) return _options;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_options);
}

/// Why the settings could not be read or the last change saved.
@override final  ApplicationError? error;

/// Create a copy of SyncSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SyncSettingsStateCopyWith<_SyncSettingsState> get copyWith => __$SyncSettingsStateCopyWithImpl<_SyncSettingsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SyncSettingsState&&(identical(other.status, status) || other.status == status)&&(identical(other.settings, settings) || other.settings == settings)&&const DeepCollectionEquality().equals(other._options, _options)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,settings,const DeepCollectionEquality().hash(_options),error);

@override
String toString() {
  return 'SyncSettingsState(status: $status, settings: $settings, options: $options, error: $error)';
}


}

/// @nodoc
abstract mixin class _$SyncSettingsStateCopyWith<$Res> implements $SyncSettingsStateCopyWith<$Res> {
  factory _$SyncSettingsStateCopyWith(_SyncSettingsState value, $Res Function(_SyncSettingsState) _then) = __$SyncSettingsStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, SyncSettingsRow? settings, List<SyncIntervalOption> options, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$SyncSettingsStateCopyWithImpl<$Res>
    implements _$SyncSettingsStateCopyWith<$Res> {
  __$SyncSettingsStateCopyWithImpl(this._self, this._then);

  final _SyncSettingsState _self;
  final $Res Function(_SyncSettingsState) _then;

/// Create a copy of SyncSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? settings = freezed,Object? options = null,Object? error = freezed,}) {
  return _then(_SyncSettingsState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,settings: freezed == settings ? _self.settings : settings // ignore: cast_nullable_to_non_nullable
as SyncSettingsRow?,options: null == options ? _self._options : options // ignore: cast_nullable_to_non_nullable
as List<SyncIntervalOption>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of SyncSettingsState
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
