// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'discovery_settings_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DiscoverySettingsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DiscoverySettingsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DiscoverySettingsEvent()';
}


}

/// @nodoc
class $DiscoverySettingsEventCopyWith<$Res>  {
$DiscoverySettingsEventCopyWith(DiscoverySettingsEvent _, $Res Function(DiscoverySettingsEvent) __);
}


/// Adds pattern-matching-related methods to [DiscoverySettingsEvent].
extension DiscoverySettingsEventPatterns on DiscoverySettingsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DiscoverySettingsStartedEvent value)?  started,TResult Function( RelatedSettingChangedEvent value)?  relatedChanged,TResult Function( RelatedRetryRequestedEvent value)?  relatedRetryRequested,TResult Function( OcrSettingChangedEvent value)?  ocrChanged,TResult Function( RelatedStatusReceivedEvent value)?  relatedStatusReceived,TResult Function( OcrCountReceivedEvent value)?  ocrCountReceived,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DiscoverySettingsStartedEvent() when started != null:
return started(_that);case RelatedSettingChangedEvent() when relatedChanged != null:
return relatedChanged(_that);case RelatedRetryRequestedEvent() when relatedRetryRequested != null:
return relatedRetryRequested(_that);case OcrSettingChangedEvent() when ocrChanged != null:
return ocrChanged(_that);case RelatedStatusReceivedEvent() when relatedStatusReceived != null:
return relatedStatusReceived(_that);case OcrCountReceivedEvent() when ocrCountReceived != null:
return ocrCountReceived(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DiscoverySettingsStartedEvent value)  started,required TResult Function( RelatedSettingChangedEvent value)  relatedChanged,required TResult Function( RelatedRetryRequestedEvent value)  relatedRetryRequested,required TResult Function( OcrSettingChangedEvent value)  ocrChanged,required TResult Function( RelatedStatusReceivedEvent value)  relatedStatusReceived,required TResult Function( OcrCountReceivedEvent value)  ocrCountReceived,}){
final _that = this;
switch (_that) {
case DiscoverySettingsStartedEvent():
return started(_that);case RelatedSettingChangedEvent():
return relatedChanged(_that);case RelatedRetryRequestedEvent():
return relatedRetryRequested(_that);case OcrSettingChangedEvent():
return ocrChanged(_that);case RelatedStatusReceivedEvent():
return relatedStatusReceived(_that);case OcrCountReceivedEvent():
return ocrCountReceived(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DiscoverySettingsStartedEvent value)?  started,TResult? Function( RelatedSettingChangedEvent value)?  relatedChanged,TResult? Function( RelatedRetryRequestedEvent value)?  relatedRetryRequested,TResult? Function( OcrSettingChangedEvent value)?  ocrChanged,TResult? Function( RelatedStatusReceivedEvent value)?  relatedStatusReceived,TResult? Function( OcrCountReceivedEvent value)?  ocrCountReceived,}){
final _that = this;
switch (_that) {
case DiscoverySettingsStartedEvent() when started != null:
return started(_that);case RelatedSettingChangedEvent() when relatedChanged != null:
return relatedChanged(_that);case RelatedRetryRequestedEvent() when relatedRetryRequested != null:
return relatedRetryRequested(_that);case OcrSettingChangedEvent() when ocrChanged != null:
return ocrChanged(_that);case RelatedStatusReceivedEvent() when relatedStatusReceived != null:
return relatedStatusReceived(_that);case OcrCountReceivedEvent() when ocrCountReceived != null:
return ocrCountReceived(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( bool enabled)?  relatedChanged,TResult Function()?  relatedRetryRequested,TResult Function( bool enabled)?  ocrChanged,TResult Function( RelatedState status)?  relatedStatusReceived,TResult Function( int count)?  ocrCountReceived,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DiscoverySettingsStartedEvent() when started != null:
return started();case RelatedSettingChangedEvent() when relatedChanged != null:
return relatedChanged(_that.enabled);case RelatedRetryRequestedEvent() when relatedRetryRequested != null:
return relatedRetryRequested();case OcrSettingChangedEvent() when ocrChanged != null:
return ocrChanged(_that.enabled);case RelatedStatusReceivedEvent() when relatedStatusReceived != null:
return relatedStatusReceived(_that.status);case OcrCountReceivedEvent() when ocrCountReceived != null:
return ocrCountReceived(_that.count);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( bool enabled)  relatedChanged,required TResult Function()  relatedRetryRequested,required TResult Function( bool enabled)  ocrChanged,required TResult Function( RelatedState status)  relatedStatusReceived,required TResult Function( int count)  ocrCountReceived,}) {final _that = this;
switch (_that) {
case DiscoverySettingsStartedEvent():
return started();case RelatedSettingChangedEvent():
return relatedChanged(_that.enabled);case RelatedRetryRequestedEvent():
return relatedRetryRequested();case OcrSettingChangedEvent():
return ocrChanged(_that.enabled);case RelatedStatusReceivedEvent():
return relatedStatusReceived(_that.status);case OcrCountReceivedEvent():
return ocrCountReceived(_that.count);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( bool enabled)?  relatedChanged,TResult? Function()?  relatedRetryRequested,TResult? Function( bool enabled)?  ocrChanged,TResult? Function( RelatedState status)?  relatedStatusReceived,TResult? Function( int count)?  ocrCountReceived,}) {final _that = this;
switch (_that) {
case DiscoverySettingsStartedEvent() when started != null:
return started();case RelatedSettingChangedEvent() when relatedChanged != null:
return relatedChanged(_that.enabled);case RelatedRetryRequestedEvent() when relatedRetryRequested != null:
return relatedRetryRequested();case OcrSettingChangedEvent() when ocrChanged != null:
return ocrChanged(_that.enabled);case RelatedStatusReceivedEvent() when relatedStatusReceived != null:
return relatedStatusReceived(_that.status);case OcrCountReceivedEvent() when ocrCountReceived != null:
return ocrCountReceived(_that.count);case _:
  return null;

}
}

}

/// @nodoc


class DiscoverySettingsStartedEvent implements DiscoverySettingsEvent {
  const DiscoverySettingsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DiscoverySettingsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DiscoverySettingsEvent.started()';
}


}




/// @nodoc


class RelatedSettingChangedEvent implements DiscoverySettingsEvent {
  const RelatedSettingChangedEvent({required this.enabled});
  

 final  bool enabled;

/// Create a copy of DiscoverySettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RelatedSettingChangedEventCopyWith<RelatedSettingChangedEvent> get copyWith => _$RelatedSettingChangedEventCopyWithImpl<RelatedSettingChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RelatedSettingChangedEvent&&(identical(other.enabled, enabled) || other.enabled == enabled));
}


@override
int get hashCode => Object.hash(runtimeType,enabled);

@override
String toString() {
  return 'DiscoverySettingsEvent.relatedChanged(enabled: $enabled)';
}


}

/// @nodoc
abstract mixin class $RelatedSettingChangedEventCopyWith<$Res> implements $DiscoverySettingsEventCopyWith<$Res> {
  factory $RelatedSettingChangedEventCopyWith(RelatedSettingChangedEvent value, $Res Function(RelatedSettingChangedEvent) _then) = _$RelatedSettingChangedEventCopyWithImpl;
@useResult
$Res call({
 bool enabled
});




}
/// @nodoc
class _$RelatedSettingChangedEventCopyWithImpl<$Res>
    implements $RelatedSettingChangedEventCopyWith<$Res> {
  _$RelatedSettingChangedEventCopyWithImpl(this._self, this._then);

  final RelatedSettingChangedEvent _self;
  final $Res Function(RelatedSettingChangedEvent) _then;

/// Create a copy of DiscoverySettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? enabled = null,}) {
  return _then(RelatedSettingChangedEvent(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class RelatedRetryRequestedEvent implements DiscoverySettingsEvent {
  const RelatedRetryRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RelatedRetryRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DiscoverySettingsEvent.relatedRetryRequested()';
}


}




/// @nodoc


class OcrSettingChangedEvent implements DiscoverySettingsEvent {
  const OcrSettingChangedEvent({required this.enabled});
  

 final  bool enabled;

/// Create a copy of DiscoverySettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OcrSettingChangedEventCopyWith<OcrSettingChangedEvent> get copyWith => _$OcrSettingChangedEventCopyWithImpl<OcrSettingChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OcrSettingChangedEvent&&(identical(other.enabled, enabled) || other.enabled == enabled));
}


@override
int get hashCode => Object.hash(runtimeType,enabled);

@override
String toString() {
  return 'DiscoverySettingsEvent.ocrChanged(enabled: $enabled)';
}


}

/// @nodoc
abstract mixin class $OcrSettingChangedEventCopyWith<$Res> implements $DiscoverySettingsEventCopyWith<$Res> {
  factory $OcrSettingChangedEventCopyWith(OcrSettingChangedEvent value, $Res Function(OcrSettingChangedEvent) _then) = _$OcrSettingChangedEventCopyWithImpl;
@useResult
$Res call({
 bool enabled
});




}
/// @nodoc
class _$OcrSettingChangedEventCopyWithImpl<$Res>
    implements $OcrSettingChangedEventCopyWith<$Res> {
  _$OcrSettingChangedEventCopyWithImpl(this._self, this._then);

  final OcrSettingChangedEvent _self;
  final $Res Function(OcrSettingChangedEvent) _then;

/// Create a copy of DiscoverySettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? enabled = null,}) {
  return _then(OcrSettingChangedEvent(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class RelatedStatusReceivedEvent implements DiscoverySettingsEvent {
  const RelatedStatusReceivedEvent(this.status);
  

 final  RelatedState status;

/// Create a copy of DiscoverySettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RelatedStatusReceivedEventCopyWith<RelatedStatusReceivedEvent> get copyWith => _$RelatedStatusReceivedEventCopyWithImpl<RelatedStatusReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RelatedStatusReceivedEvent&&(identical(other.status, status) || other.status == status));
}


@override
int get hashCode => Object.hash(runtimeType,status);

@override
String toString() {
  return 'DiscoverySettingsEvent.relatedStatusReceived(status: $status)';
}


}

/// @nodoc
abstract mixin class $RelatedStatusReceivedEventCopyWith<$Res> implements $DiscoverySettingsEventCopyWith<$Res> {
  factory $RelatedStatusReceivedEventCopyWith(RelatedStatusReceivedEvent value, $Res Function(RelatedStatusReceivedEvent) _then) = _$RelatedStatusReceivedEventCopyWithImpl;
@useResult
$Res call({
 RelatedState status
});




}
/// @nodoc
class _$RelatedStatusReceivedEventCopyWithImpl<$Res>
    implements $RelatedStatusReceivedEventCopyWith<$Res> {
  _$RelatedStatusReceivedEventCopyWithImpl(this._self, this._then);

  final RelatedStatusReceivedEvent _self;
  final $Res Function(RelatedStatusReceivedEvent) _then;

/// Create a copy of DiscoverySettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? status = null,}) {
  return _then(RelatedStatusReceivedEvent(
null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as RelatedState,
  ));
}


}

/// @nodoc


class OcrCountReceivedEvent implements DiscoverySettingsEvent {
  const OcrCountReceivedEvent(this.count);
  

 final  int count;

/// Create a copy of DiscoverySettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OcrCountReceivedEventCopyWith<OcrCountReceivedEvent> get copyWith => _$OcrCountReceivedEventCopyWithImpl<OcrCountReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OcrCountReceivedEvent&&(identical(other.count, count) || other.count == count));
}


@override
int get hashCode => Object.hash(runtimeType,count);

@override
String toString() {
  return 'DiscoverySettingsEvent.ocrCountReceived(count: $count)';
}


}

/// @nodoc
abstract mixin class $OcrCountReceivedEventCopyWith<$Res> implements $DiscoverySettingsEventCopyWith<$Res> {
  factory $OcrCountReceivedEventCopyWith(OcrCountReceivedEvent value, $Res Function(OcrCountReceivedEvent) _then) = _$OcrCountReceivedEventCopyWithImpl;
@useResult
$Res call({
 int count
});




}
/// @nodoc
class _$OcrCountReceivedEventCopyWithImpl<$Res>
    implements $OcrCountReceivedEventCopyWith<$Res> {
  _$OcrCountReceivedEventCopyWithImpl(this._self, this._then);

  final OcrCountReceivedEvent _self;
  final $Res Function(OcrCountReceivedEvent) _then;

/// Create a copy of DiscoverySettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? count = null,}) {
  return _then(OcrCountReceivedEvent(
null == count ? _self.count : count // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc
mixin _$DiscoverySettingsState {

 bool get relatedEnabled; RelatedState? get related; bool get relatedBusy; bool get ocrEnabled; int get ocrWaiting;
/// Create a copy of DiscoverySettingsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DiscoverySettingsStateCopyWith<DiscoverySettingsState> get copyWith => _$DiscoverySettingsStateCopyWithImpl<DiscoverySettingsState>(this as DiscoverySettingsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DiscoverySettingsState&&(identical(other.relatedEnabled, relatedEnabled) || other.relatedEnabled == relatedEnabled)&&(identical(other.related, related) || other.related == related)&&(identical(other.relatedBusy, relatedBusy) || other.relatedBusy == relatedBusy)&&(identical(other.ocrEnabled, ocrEnabled) || other.ocrEnabled == ocrEnabled)&&(identical(other.ocrWaiting, ocrWaiting) || other.ocrWaiting == ocrWaiting));
}


@override
int get hashCode => Object.hash(runtimeType,relatedEnabled,related,relatedBusy,ocrEnabled,ocrWaiting);

@override
String toString() {
  return 'DiscoverySettingsState(relatedEnabled: $relatedEnabled, related: $related, relatedBusy: $relatedBusy, ocrEnabled: $ocrEnabled, ocrWaiting: $ocrWaiting)';
}


}

/// @nodoc
abstract mixin class $DiscoverySettingsStateCopyWith<$Res>  {
  factory $DiscoverySettingsStateCopyWith(DiscoverySettingsState value, $Res Function(DiscoverySettingsState) _then) = _$DiscoverySettingsStateCopyWithImpl;
@useResult
$Res call({
 bool relatedEnabled, RelatedState? related, bool relatedBusy, bool ocrEnabled, int ocrWaiting
});




}
/// @nodoc
class _$DiscoverySettingsStateCopyWithImpl<$Res>
    implements $DiscoverySettingsStateCopyWith<$Res> {
  _$DiscoverySettingsStateCopyWithImpl(this._self, this._then);

  final DiscoverySettingsState _self;
  final $Res Function(DiscoverySettingsState) _then;

/// Create a copy of DiscoverySettingsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? relatedEnabled = null,Object? related = freezed,Object? relatedBusy = null,Object? ocrEnabled = null,Object? ocrWaiting = null,}) {
  return _then(_self.copyWith(
relatedEnabled: null == relatedEnabled ? _self.relatedEnabled : relatedEnabled // ignore: cast_nullable_to_non_nullable
as bool,related: freezed == related ? _self.related : related // ignore: cast_nullable_to_non_nullable
as RelatedState?,relatedBusy: null == relatedBusy ? _self.relatedBusy : relatedBusy // ignore: cast_nullable_to_non_nullable
as bool,ocrEnabled: null == ocrEnabled ? _self.ocrEnabled : ocrEnabled // ignore: cast_nullable_to_non_nullable
as bool,ocrWaiting: null == ocrWaiting ? _self.ocrWaiting : ocrWaiting // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [DiscoverySettingsState].
extension DiscoverySettingsStatePatterns on DiscoverySettingsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DiscoverySettingsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DiscoverySettingsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DiscoverySettingsState value)  $default,){
final _that = this;
switch (_that) {
case _DiscoverySettingsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DiscoverySettingsState value)?  $default,){
final _that = this;
switch (_that) {
case _DiscoverySettingsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool relatedEnabled,  RelatedState? related,  bool relatedBusy,  bool ocrEnabled,  int ocrWaiting)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DiscoverySettingsState() when $default != null:
return $default(_that.relatedEnabled,_that.related,_that.relatedBusy,_that.ocrEnabled,_that.ocrWaiting);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool relatedEnabled,  RelatedState? related,  bool relatedBusy,  bool ocrEnabled,  int ocrWaiting)  $default,) {final _that = this;
switch (_that) {
case _DiscoverySettingsState():
return $default(_that.relatedEnabled,_that.related,_that.relatedBusy,_that.ocrEnabled,_that.ocrWaiting);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool relatedEnabled,  RelatedState? related,  bool relatedBusy,  bool ocrEnabled,  int ocrWaiting)?  $default,) {final _that = this;
switch (_that) {
case _DiscoverySettingsState() when $default != null:
return $default(_that.relatedEnabled,_that.related,_that.relatedBusy,_that.ocrEnabled,_that.ocrWaiting);case _:
  return null;

}
}

}

/// @nodoc


class _DiscoverySettingsState implements DiscoverySettingsState {
  const _DiscoverySettingsState({this.relatedEnabled = false, this.related, this.relatedBusy = false, this.ocrEnabled = false, this.ocrWaiting = 0});
  

@override@JsonKey() final  bool relatedEnabled;
@override final  RelatedState? related;
@override@JsonKey() final  bool relatedBusy;
@override@JsonKey() final  bool ocrEnabled;
@override@JsonKey() final  int ocrWaiting;

/// Create a copy of DiscoverySettingsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DiscoverySettingsStateCopyWith<_DiscoverySettingsState> get copyWith => __$DiscoverySettingsStateCopyWithImpl<_DiscoverySettingsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DiscoverySettingsState&&(identical(other.relatedEnabled, relatedEnabled) || other.relatedEnabled == relatedEnabled)&&(identical(other.related, related) || other.related == related)&&(identical(other.relatedBusy, relatedBusy) || other.relatedBusy == relatedBusy)&&(identical(other.ocrEnabled, ocrEnabled) || other.ocrEnabled == ocrEnabled)&&(identical(other.ocrWaiting, ocrWaiting) || other.ocrWaiting == ocrWaiting));
}


@override
int get hashCode => Object.hash(runtimeType,relatedEnabled,related,relatedBusy,ocrEnabled,ocrWaiting);

@override
String toString() {
  return 'DiscoverySettingsState(relatedEnabled: $relatedEnabled, related: $related, relatedBusy: $relatedBusy, ocrEnabled: $ocrEnabled, ocrWaiting: $ocrWaiting)';
}


}

/// @nodoc
abstract mixin class _$DiscoverySettingsStateCopyWith<$Res> implements $DiscoverySettingsStateCopyWith<$Res> {
  factory _$DiscoverySettingsStateCopyWith(_DiscoverySettingsState value, $Res Function(_DiscoverySettingsState) _then) = __$DiscoverySettingsStateCopyWithImpl;
@override @useResult
$Res call({
 bool relatedEnabled, RelatedState? related, bool relatedBusy, bool ocrEnabled, int ocrWaiting
});




}
/// @nodoc
class __$DiscoverySettingsStateCopyWithImpl<$Res>
    implements _$DiscoverySettingsStateCopyWith<$Res> {
  __$DiscoverySettingsStateCopyWithImpl(this._self, this._then);

  final _DiscoverySettingsState _self;
  final $Res Function(_DiscoverySettingsState) _then;

/// Create a copy of DiscoverySettingsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? relatedEnabled = null,Object? related = freezed,Object? relatedBusy = null,Object? ocrEnabled = null,Object? ocrWaiting = null,}) {
  return _then(_DiscoverySettingsState(
relatedEnabled: null == relatedEnabled ? _self.relatedEnabled : relatedEnabled // ignore: cast_nullable_to_non_nullable
as bool,related: freezed == related ? _self.related : related // ignore: cast_nullable_to_non_nullable
as RelatedState?,relatedBusy: null == relatedBusy ? _self.relatedBusy : relatedBusy // ignore: cast_nullable_to_non_nullable
as bool,ocrEnabled: null == ocrEnabled ? _self.ocrEnabled : ocrEnabled // ignore: cast_nullable_to_non_nullable
as bool,ocrWaiting: null == ocrWaiting ? _self.ocrWaiting : ocrWaiting // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
