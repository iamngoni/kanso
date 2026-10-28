// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ai_settings_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AiFields {

 String get address; String get model; String get region;
/// Create a copy of AiFields
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiFieldsCopyWith<AiFields> get copyWith => _$AiFieldsCopyWithImpl<AiFields>(this as AiFields, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiFields&&(identical(other.address, address) || other.address == address)&&(identical(other.model, model) || other.model == model)&&(identical(other.region, region) || other.region == region));
}


@override
int get hashCode => Object.hash(runtimeType,address,model,region);

@override
String toString() {
  return 'AiFields(address: $address, model: $model, region: $region)';
}


}

/// @nodoc
abstract mixin class $AiFieldsCopyWith<$Res>  {
  factory $AiFieldsCopyWith(AiFields value, $Res Function(AiFields) _then) = _$AiFieldsCopyWithImpl;
@useResult
$Res call({
 String address, String model, String region
});




}
/// @nodoc
class _$AiFieldsCopyWithImpl<$Res>
    implements $AiFieldsCopyWith<$Res> {
  _$AiFieldsCopyWithImpl(this._self, this._then);

  final AiFields _self;
  final $Res Function(AiFields) _then;

/// Create a copy of AiFields
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? address = null,Object? model = null,Object? region = null,}) {
  return _then(_self.copyWith(
address: null == address ? _self.address : address // ignore: cast_nullable_to_non_nullable
as String,model: null == model ? _self.model : model // ignore: cast_nullable_to_non_nullable
as String,region: null == region ? _self.region : region // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [AiFields].
extension AiFieldsPatterns on AiFields {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AiFields value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AiFields() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AiFields value)  $default,){
final _that = this;
switch (_that) {
case _AiFields():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AiFields value)?  $default,){
final _that = this;
switch (_that) {
case _AiFields() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String address,  String model,  String region)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AiFields() when $default != null:
return $default(_that.address,_that.model,_that.region);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String address,  String model,  String region)  $default,) {final _that = this;
switch (_that) {
case _AiFields():
return $default(_that.address,_that.model,_that.region);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String address,  String model,  String region)?  $default,) {final _that = this;
switch (_that) {
case _AiFields() when $default != null:
return $default(_that.address,_that.model,_that.region);case _:
  return null;

}
}

}

/// @nodoc


class _AiFields implements AiFields {
  const _AiFields({this.address = '', this.model = '', this.region = ''});
  

@override@JsonKey() final  String address;
@override@JsonKey() final  String model;
@override@JsonKey() final  String region;

/// Create a copy of AiFields
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AiFieldsCopyWith<_AiFields> get copyWith => __$AiFieldsCopyWithImpl<_AiFields>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AiFields&&(identical(other.address, address) || other.address == address)&&(identical(other.model, model) || other.model == model)&&(identical(other.region, region) || other.region == region));
}


@override
int get hashCode => Object.hash(runtimeType,address,model,region);

@override
String toString() {
  return 'AiFields(address: $address, model: $model, region: $region)';
}


}

/// @nodoc
abstract mixin class _$AiFieldsCopyWith<$Res> implements $AiFieldsCopyWith<$Res> {
  factory _$AiFieldsCopyWith(_AiFields value, $Res Function(_AiFields) _then) = __$AiFieldsCopyWithImpl;
@override @useResult
$Res call({
 String address, String model, String region
});




}
/// @nodoc
class __$AiFieldsCopyWithImpl<$Res>
    implements _$AiFieldsCopyWith<$Res> {
  __$AiFieldsCopyWithImpl(this._self, this._then);

  final _AiFields _self;
  final $Res Function(_AiFields) _then;

/// Create a copy of AiFields
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? address = null,Object? model = null,Object? region = null,}) {
  return _then(_AiFields(
address: null == address ? _self.address : address // ignore: cast_nullable_to_non_nullable
as String,model: null == model ? _self.model : model // ignore: cast_nullable_to_non_nullable
as String,region: null == region ? _self.region : region // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$AiSettingsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettingsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AiSettingsEvent()';
}


}

/// @nodoc
class $AiSettingsEventCopyWith<$Res>  {
$AiSettingsEventCopyWith(AiSettingsEvent _, $Res Function(AiSettingsEvent) __);
}


/// Adds pattern-matching-related methods to [AiSettingsEvent].
extension AiSettingsEventPatterns on AiSettingsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AiSettingsStartedEvent value)?  started,TResult Function( AiSettingsEnabledChangedEvent value)?  enabledChanged,TResult Function( AiSettingsProviderChosenEvent value)?  providerChosen,TResult Function( AiSettingsFieldsChangedEvent value)?  fieldsChanged,TResult Function( AiSettingsKeySavedEvent value)?  keySaved,TResult Function( AiSettingsKeyRemovedEvent value)?  keyRemoved,TResult Function( AiSettingsTestRequestedEvent value)?  testRequested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AiSettingsStartedEvent() when started != null:
return started(_that);case AiSettingsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that);case AiSettingsProviderChosenEvent() when providerChosen != null:
return providerChosen(_that);case AiSettingsFieldsChangedEvent() when fieldsChanged != null:
return fieldsChanged(_that);case AiSettingsKeySavedEvent() when keySaved != null:
return keySaved(_that);case AiSettingsKeyRemovedEvent() when keyRemoved != null:
return keyRemoved(_that);case AiSettingsTestRequestedEvent() when testRequested != null:
return testRequested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AiSettingsStartedEvent value)  started,required TResult Function( AiSettingsEnabledChangedEvent value)  enabledChanged,required TResult Function( AiSettingsProviderChosenEvent value)  providerChosen,required TResult Function( AiSettingsFieldsChangedEvent value)  fieldsChanged,required TResult Function( AiSettingsKeySavedEvent value)  keySaved,required TResult Function( AiSettingsKeyRemovedEvent value)  keyRemoved,required TResult Function( AiSettingsTestRequestedEvent value)  testRequested,}){
final _that = this;
switch (_that) {
case AiSettingsStartedEvent():
return started(_that);case AiSettingsEnabledChangedEvent():
return enabledChanged(_that);case AiSettingsProviderChosenEvent():
return providerChosen(_that);case AiSettingsFieldsChangedEvent():
return fieldsChanged(_that);case AiSettingsKeySavedEvent():
return keySaved(_that);case AiSettingsKeyRemovedEvent():
return keyRemoved(_that);case AiSettingsTestRequestedEvent():
return testRequested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AiSettingsStartedEvent value)?  started,TResult? Function( AiSettingsEnabledChangedEvent value)?  enabledChanged,TResult? Function( AiSettingsProviderChosenEvent value)?  providerChosen,TResult? Function( AiSettingsFieldsChangedEvent value)?  fieldsChanged,TResult? Function( AiSettingsKeySavedEvent value)?  keySaved,TResult? Function( AiSettingsKeyRemovedEvent value)?  keyRemoved,TResult? Function( AiSettingsTestRequestedEvent value)?  testRequested,}){
final _that = this;
switch (_that) {
case AiSettingsStartedEvent() when started != null:
return started(_that);case AiSettingsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that);case AiSettingsProviderChosenEvent() when providerChosen != null:
return providerChosen(_that);case AiSettingsFieldsChangedEvent() when fieldsChanged != null:
return fieldsChanged(_that);case AiSettingsKeySavedEvent() when keySaved != null:
return keySaved(_that);case AiSettingsKeyRemovedEvent() when keyRemoved != null:
return keyRemoved(_that);case AiSettingsTestRequestedEvent() when testRequested != null:
return testRequested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( bool enabled)?  enabledChanged,TResult Function( String provider)?  providerChosen,TResult Function( AiFields fields)?  fieldsChanged,TResult Function( AiKey key)?  keySaved,TResult Function()?  keyRemoved,TResult Function( AiFields fields)?  testRequested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AiSettingsStartedEvent() when started != null:
return started();case AiSettingsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that.enabled);case AiSettingsProviderChosenEvent() when providerChosen != null:
return providerChosen(_that.provider);case AiSettingsFieldsChangedEvent() when fieldsChanged != null:
return fieldsChanged(_that.fields);case AiSettingsKeySavedEvent() when keySaved != null:
return keySaved(_that.key);case AiSettingsKeyRemovedEvent() when keyRemoved != null:
return keyRemoved();case AiSettingsTestRequestedEvent() when testRequested != null:
return testRequested(_that.fields);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( bool enabled)  enabledChanged,required TResult Function( String provider)  providerChosen,required TResult Function( AiFields fields)  fieldsChanged,required TResult Function( AiKey key)  keySaved,required TResult Function()  keyRemoved,required TResult Function( AiFields fields)  testRequested,}) {final _that = this;
switch (_that) {
case AiSettingsStartedEvent():
return started();case AiSettingsEnabledChangedEvent():
return enabledChanged(_that.enabled);case AiSettingsProviderChosenEvent():
return providerChosen(_that.provider);case AiSettingsFieldsChangedEvent():
return fieldsChanged(_that.fields);case AiSettingsKeySavedEvent():
return keySaved(_that.key);case AiSettingsKeyRemovedEvent():
return keyRemoved();case AiSettingsTestRequestedEvent():
return testRequested(_that.fields);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( bool enabled)?  enabledChanged,TResult? Function( String provider)?  providerChosen,TResult? Function( AiFields fields)?  fieldsChanged,TResult? Function( AiKey key)?  keySaved,TResult? Function()?  keyRemoved,TResult? Function( AiFields fields)?  testRequested,}) {final _that = this;
switch (_that) {
case AiSettingsStartedEvent() when started != null:
return started();case AiSettingsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that.enabled);case AiSettingsProviderChosenEvent() when providerChosen != null:
return providerChosen(_that.provider);case AiSettingsFieldsChangedEvent() when fieldsChanged != null:
return fieldsChanged(_that.fields);case AiSettingsKeySavedEvent() when keySaved != null:
return keySaved(_that.key);case AiSettingsKeyRemovedEvent() when keyRemoved != null:
return keyRemoved();case AiSettingsTestRequestedEvent() when testRequested != null:
return testRequested(_that.fields);case _:
  return null;

}
}

}

/// @nodoc


class AiSettingsStartedEvent implements AiSettingsEvent {
  const AiSettingsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettingsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AiSettingsEvent.started()';
}


}




/// @nodoc


class AiSettingsEnabledChangedEvent implements AiSettingsEvent {
  const AiSettingsEnabledChangedEvent({required this.enabled});
  

 final  bool enabled;

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiSettingsEnabledChangedEventCopyWith<AiSettingsEnabledChangedEvent> get copyWith => _$AiSettingsEnabledChangedEventCopyWithImpl<AiSettingsEnabledChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettingsEnabledChangedEvent&&(identical(other.enabled, enabled) || other.enabled == enabled));
}


@override
int get hashCode => Object.hash(runtimeType,enabled);

@override
String toString() {
  return 'AiSettingsEvent.enabledChanged(enabled: $enabled)';
}


}

/// @nodoc
abstract mixin class $AiSettingsEnabledChangedEventCopyWith<$Res> implements $AiSettingsEventCopyWith<$Res> {
  factory $AiSettingsEnabledChangedEventCopyWith(AiSettingsEnabledChangedEvent value, $Res Function(AiSettingsEnabledChangedEvent) _then) = _$AiSettingsEnabledChangedEventCopyWithImpl;
@useResult
$Res call({
 bool enabled
});




}
/// @nodoc
class _$AiSettingsEnabledChangedEventCopyWithImpl<$Res>
    implements $AiSettingsEnabledChangedEventCopyWith<$Res> {
  _$AiSettingsEnabledChangedEventCopyWithImpl(this._self, this._then);

  final AiSettingsEnabledChangedEvent _self;
  final $Res Function(AiSettingsEnabledChangedEvent) _then;

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? enabled = null,}) {
  return _then(AiSettingsEnabledChangedEvent(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class AiSettingsProviderChosenEvent implements AiSettingsEvent {
  const AiSettingsProviderChosenEvent(this.provider);
  

 final  String provider;

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiSettingsProviderChosenEventCopyWith<AiSettingsProviderChosenEvent> get copyWith => _$AiSettingsProviderChosenEventCopyWithImpl<AiSettingsProviderChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettingsProviderChosenEvent&&(identical(other.provider, provider) || other.provider == provider));
}


@override
int get hashCode => Object.hash(runtimeType,provider);

@override
String toString() {
  return 'AiSettingsEvent.providerChosen(provider: $provider)';
}


}

/// @nodoc
abstract mixin class $AiSettingsProviderChosenEventCopyWith<$Res> implements $AiSettingsEventCopyWith<$Res> {
  factory $AiSettingsProviderChosenEventCopyWith(AiSettingsProviderChosenEvent value, $Res Function(AiSettingsProviderChosenEvent) _then) = _$AiSettingsProviderChosenEventCopyWithImpl;
@useResult
$Res call({
 String provider
});




}
/// @nodoc
class _$AiSettingsProviderChosenEventCopyWithImpl<$Res>
    implements $AiSettingsProviderChosenEventCopyWith<$Res> {
  _$AiSettingsProviderChosenEventCopyWithImpl(this._self, this._then);

  final AiSettingsProviderChosenEvent _self;
  final $Res Function(AiSettingsProviderChosenEvent) _then;

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? provider = null,}) {
  return _then(AiSettingsProviderChosenEvent(
null == provider ? _self.provider : provider // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AiSettingsFieldsChangedEvent implements AiSettingsEvent {
  const AiSettingsFieldsChangedEvent(this.fields);
  

 final  AiFields fields;

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiSettingsFieldsChangedEventCopyWith<AiSettingsFieldsChangedEvent> get copyWith => _$AiSettingsFieldsChangedEventCopyWithImpl<AiSettingsFieldsChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettingsFieldsChangedEvent&&(identical(other.fields, fields) || other.fields == fields));
}


@override
int get hashCode => Object.hash(runtimeType,fields);

@override
String toString() {
  return 'AiSettingsEvent.fieldsChanged(fields: $fields)';
}


}

/// @nodoc
abstract mixin class $AiSettingsFieldsChangedEventCopyWith<$Res> implements $AiSettingsEventCopyWith<$Res> {
  factory $AiSettingsFieldsChangedEventCopyWith(AiSettingsFieldsChangedEvent value, $Res Function(AiSettingsFieldsChangedEvent) _then) = _$AiSettingsFieldsChangedEventCopyWithImpl;
@useResult
$Res call({
 AiFields fields
});


$AiFieldsCopyWith<$Res> get fields;

}
/// @nodoc
class _$AiSettingsFieldsChangedEventCopyWithImpl<$Res>
    implements $AiSettingsFieldsChangedEventCopyWith<$Res> {
  _$AiSettingsFieldsChangedEventCopyWithImpl(this._self, this._then);

  final AiSettingsFieldsChangedEvent _self;
  final $Res Function(AiSettingsFieldsChangedEvent) _then;

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? fields = null,}) {
  return _then(AiSettingsFieldsChangedEvent(
null == fields ? _self.fields : fields // ignore: cast_nullable_to_non_nullable
as AiFields,
  ));
}

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AiFieldsCopyWith<$Res> get fields {
  
  return $AiFieldsCopyWith<$Res>(_self.fields, (value) {
    return _then(_self.copyWith(fields: value));
  });
}
}

/// @nodoc


class AiSettingsKeySavedEvent implements AiSettingsEvent {
  const AiSettingsKeySavedEvent(this.key);
  

 final  AiKey key;

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiSettingsKeySavedEventCopyWith<AiSettingsKeySavedEvent> get copyWith => _$AiSettingsKeySavedEventCopyWithImpl<AiSettingsKeySavedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettingsKeySavedEvent&&(identical(other.key, key) || other.key == key));
}


@override
int get hashCode => Object.hash(runtimeType,key);

@override
String toString() {
  return 'AiSettingsEvent.keySaved(key: $key)';
}


}

/// @nodoc
abstract mixin class $AiSettingsKeySavedEventCopyWith<$Res> implements $AiSettingsEventCopyWith<$Res> {
  factory $AiSettingsKeySavedEventCopyWith(AiSettingsKeySavedEvent value, $Res Function(AiSettingsKeySavedEvent) _then) = _$AiSettingsKeySavedEventCopyWithImpl;
@useResult
$Res call({
 AiKey key
});


$AiKeyCopyWith<$Res> get key;

}
/// @nodoc
class _$AiSettingsKeySavedEventCopyWithImpl<$Res>
    implements $AiSettingsKeySavedEventCopyWith<$Res> {
  _$AiSettingsKeySavedEventCopyWithImpl(this._self, this._then);

  final AiSettingsKeySavedEvent _self;
  final $Res Function(AiSettingsKeySavedEvent) _then;

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = null,}) {
  return _then(AiSettingsKeySavedEvent(
null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as AiKey,
  ));
}

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AiKeyCopyWith<$Res> get key {
  
  return $AiKeyCopyWith<$Res>(_self.key, (value) {
    return _then(_self.copyWith(key: value));
  });
}
}

/// @nodoc


class AiSettingsKeyRemovedEvent implements AiSettingsEvent {
  const AiSettingsKeyRemovedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettingsKeyRemovedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AiSettingsEvent.keyRemoved()';
}


}




/// @nodoc


class AiSettingsTestRequestedEvent implements AiSettingsEvent {
  const AiSettingsTestRequestedEvent(this.fields);
  

 final  AiFields fields;

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiSettingsTestRequestedEventCopyWith<AiSettingsTestRequestedEvent> get copyWith => _$AiSettingsTestRequestedEventCopyWithImpl<AiSettingsTestRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettingsTestRequestedEvent&&(identical(other.fields, fields) || other.fields == fields));
}


@override
int get hashCode => Object.hash(runtimeType,fields);

@override
String toString() {
  return 'AiSettingsEvent.testRequested(fields: $fields)';
}


}

/// @nodoc
abstract mixin class $AiSettingsTestRequestedEventCopyWith<$Res> implements $AiSettingsEventCopyWith<$Res> {
  factory $AiSettingsTestRequestedEventCopyWith(AiSettingsTestRequestedEvent value, $Res Function(AiSettingsTestRequestedEvent) _then) = _$AiSettingsTestRequestedEventCopyWithImpl;
@useResult
$Res call({
 AiFields fields
});


$AiFieldsCopyWith<$Res> get fields;

}
/// @nodoc
class _$AiSettingsTestRequestedEventCopyWithImpl<$Res>
    implements $AiSettingsTestRequestedEventCopyWith<$Res> {
  _$AiSettingsTestRequestedEventCopyWithImpl(this._self, this._then);

  final AiSettingsTestRequestedEvent _self;
  final $Res Function(AiSettingsTestRequestedEvent) _then;

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? fields = null,}) {
  return _then(AiSettingsTestRequestedEvent(
null == fields ? _self.fields : fields // ignore: cast_nullable_to_non_nullable
as AiFields,
  ));
}

/// Create a copy of AiSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AiFieldsCopyWith<$Res> get fields {
  
  return $AiFieldsCopyWith<$Res>(_self.fields, (value) {
    return _then(_self.copyWith(fields: value));
  });
}
}

/// @nodoc
mixin _$AiSettingsState {

 LoadStatus get status; AiSettings get settings;/// The places answers can come from.
 List<AiProviderInfo> get providers; AiTestStatus get test;/// What the model said, or why the test failed, once it has run.
 String get testMessage;/// Whether a key is being saved or removed.
 bool get keyBusy;/// Why the last change could not be saved.
 ApplicationError? get error;
/// Create a copy of AiSettingsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiSettingsStateCopyWith<AiSettingsState> get copyWith => _$AiSettingsStateCopyWithImpl<AiSettingsState>(this as AiSettingsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettingsState&&(identical(other.status, status) || other.status == status)&&(identical(other.settings, settings) || other.settings == settings)&&const DeepCollectionEquality().equals(other.providers, providers)&&(identical(other.test, test) || other.test == test)&&(identical(other.testMessage, testMessage) || other.testMessage == testMessage)&&(identical(other.keyBusy, keyBusy) || other.keyBusy == keyBusy)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,settings,const DeepCollectionEquality().hash(providers),test,testMessage,keyBusy,error);

@override
String toString() {
  return 'AiSettingsState(status: $status, settings: $settings, providers: $providers, test: $test, testMessage: $testMessage, keyBusy: $keyBusy, error: $error)';
}


}

/// @nodoc
abstract mixin class $AiSettingsStateCopyWith<$Res>  {
  factory $AiSettingsStateCopyWith(AiSettingsState value, $Res Function(AiSettingsState) _then) = _$AiSettingsStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, AiSettings settings, List<AiProviderInfo> providers, AiTestStatus test, String testMessage, bool keyBusy, ApplicationError? error
});


$AiSettingsCopyWith<$Res> get settings;$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$AiSettingsStateCopyWithImpl<$Res>
    implements $AiSettingsStateCopyWith<$Res> {
  _$AiSettingsStateCopyWithImpl(this._self, this._then);

  final AiSettingsState _self;
  final $Res Function(AiSettingsState) _then;

/// Create a copy of AiSettingsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? settings = null,Object? providers = null,Object? test = null,Object? testMessage = null,Object? keyBusy = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,settings: null == settings ? _self.settings : settings // ignore: cast_nullable_to_non_nullable
as AiSettings,providers: null == providers ? _self.providers : providers // ignore: cast_nullable_to_non_nullable
as List<AiProviderInfo>,test: null == test ? _self.test : test // ignore: cast_nullable_to_non_nullable
as AiTestStatus,testMessage: null == testMessage ? _self.testMessage : testMessage // ignore: cast_nullable_to_non_nullable
as String,keyBusy: null == keyBusy ? _self.keyBusy : keyBusy // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of AiSettingsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AiSettingsCopyWith<$Res> get settings {
  
  return $AiSettingsCopyWith<$Res>(_self.settings, (value) {
    return _then(_self.copyWith(settings: value));
  });
}/// Create a copy of AiSettingsState
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


/// Adds pattern-matching-related methods to [AiSettingsState].
extension AiSettingsStatePatterns on AiSettingsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AiSettingsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AiSettingsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AiSettingsState value)  $default,){
final _that = this;
switch (_that) {
case _AiSettingsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AiSettingsState value)?  $default,){
final _that = this;
switch (_that) {
case _AiSettingsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  AiSettings settings,  List<AiProviderInfo> providers,  AiTestStatus test,  String testMessage,  bool keyBusy,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AiSettingsState() when $default != null:
return $default(_that.status,_that.settings,_that.providers,_that.test,_that.testMessage,_that.keyBusy,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  AiSettings settings,  List<AiProviderInfo> providers,  AiTestStatus test,  String testMessage,  bool keyBusy,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _AiSettingsState():
return $default(_that.status,_that.settings,_that.providers,_that.test,_that.testMessage,_that.keyBusy,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  AiSettings settings,  List<AiProviderInfo> providers,  AiTestStatus test,  String testMessage,  bool keyBusy,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _AiSettingsState() when $default != null:
return $default(_that.status,_that.settings,_that.providers,_that.test,_that.testMessage,_that.keyBusy,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _AiSettingsState extends AiSettingsState {
  const _AiSettingsState({this.status = LoadStatus.loading, this.settings = const AiSettings(), final  List<AiProviderInfo> providers = const <AiProviderInfo>[], this.test = AiTestStatus.idle, this.testMessage = '', this.keyBusy = false, this.error}): _providers = providers,super._();
  

@override@JsonKey() final  LoadStatus status;
@override@JsonKey() final  AiSettings settings;
/// The places answers can come from.
 final  List<AiProviderInfo> _providers;
/// The places answers can come from.
@override@JsonKey() List<AiProviderInfo> get providers {
  if (_providers is EqualUnmodifiableListView) return _providers;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_providers);
}

@override@JsonKey() final  AiTestStatus test;
/// What the model said, or why the test failed, once it has run.
@override@JsonKey() final  String testMessage;
/// Whether a key is being saved or removed.
@override@JsonKey() final  bool keyBusy;
/// Why the last change could not be saved.
@override final  ApplicationError? error;

/// Create a copy of AiSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AiSettingsStateCopyWith<_AiSettingsState> get copyWith => __$AiSettingsStateCopyWithImpl<_AiSettingsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AiSettingsState&&(identical(other.status, status) || other.status == status)&&(identical(other.settings, settings) || other.settings == settings)&&const DeepCollectionEquality().equals(other._providers, _providers)&&(identical(other.test, test) || other.test == test)&&(identical(other.testMessage, testMessage) || other.testMessage == testMessage)&&(identical(other.keyBusy, keyBusy) || other.keyBusy == keyBusy)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,settings,const DeepCollectionEquality().hash(_providers),test,testMessage,keyBusy,error);

@override
String toString() {
  return 'AiSettingsState(status: $status, settings: $settings, providers: $providers, test: $test, testMessage: $testMessage, keyBusy: $keyBusy, error: $error)';
}


}

/// @nodoc
abstract mixin class _$AiSettingsStateCopyWith<$Res> implements $AiSettingsStateCopyWith<$Res> {
  factory _$AiSettingsStateCopyWith(_AiSettingsState value, $Res Function(_AiSettingsState) _then) = __$AiSettingsStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, AiSettings settings, List<AiProviderInfo> providers, AiTestStatus test, String testMessage, bool keyBusy, ApplicationError? error
});


@override $AiSettingsCopyWith<$Res> get settings;@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$AiSettingsStateCopyWithImpl<$Res>
    implements _$AiSettingsStateCopyWith<$Res> {
  __$AiSettingsStateCopyWithImpl(this._self, this._then);

  final _AiSettingsState _self;
  final $Res Function(_AiSettingsState) _then;

/// Create a copy of AiSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? settings = null,Object? providers = null,Object? test = null,Object? testMessage = null,Object? keyBusy = null,Object? error = freezed,}) {
  return _then(_AiSettingsState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,settings: null == settings ? _self.settings : settings // ignore: cast_nullable_to_non_nullable
as AiSettings,providers: null == providers ? _self._providers : providers // ignore: cast_nullable_to_non_nullable
as List<AiProviderInfo>,test: null == test ? _self.test : test // ignore: cast_nullable_to_non_nullable
as AiTestStatus,testMessage: null == testMessage ? _self.testMessage : testMessage // ignore: cast_nullable_to_non_nullable
as String,keyBusy: null == keyBusy ? _self.keyBusy : keyBusy // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of AiSettingsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AiSettingsCopyWith<$Res> get settings {
  
  return $AiSettingsCopyWith<$Res>(_self.settings, (value) {
    return _then(_self.copyWith(settings: value));
  });
}/// Create a copy of AiSettingsState
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
