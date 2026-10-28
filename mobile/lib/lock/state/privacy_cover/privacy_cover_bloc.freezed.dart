// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'privacy_cover_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PrivacyCoverEvent {

 bool get secure;
/// Create a copy of PrivacyCoverEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PrivacyCoverEventCopyWith<PrivacyCoverEvent> get copyWith => _$PrivacyCoverEventCopyWithImpl<PrivacyCoverEvent>(this as PrivacyCoverEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PrivacyCoverEvent&&(identical(other.secure, secure) || other.secure == secure));
}


@override
int get hashCode => Object.hash(runtimeType,secure);

@override
String toString() {
  return 'PrivacyCoverEvent(secure: $secure)';
}


}

/// @nodoc
abstract mixin class $PrivacyCoverEventCopyWith<$Res>  {
  factory $PrivacyCoverEventCopyWith(PrivacyCoverEvent value, $Res Function(PrivacyCoverEvent) _then) = _$PrivacyCoverEventCopyWithImpl;
@useResult
$Res call({
 bool secure
});




}
/// @nodoc
class _$PrivacyCoverEventCopyWithImpl<$Res>
    implements $PrivacyCoverEventCopyWith<$Res> {
  _$PrivacyCoverEventCopyWithImpl(this._self, this._then);

  final PrivacyCoverEvent _self;
  final $Res Function(PrivacyCoverEvent) _then;

/// Create a copy of PrivacyCoverEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? secure = null,}) {
  return _then(_self.copyWith(
secure: null == secure ? _self.secure : secure // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [PrivacyCoverEvent].
extension PrivacyCoverEventPatterns on PrivacyCoverEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PrivacyCoverChangedEvent value)?  changed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PrivacyCoverChangedEvent() when changed != null:
return changed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PrivacyCoverChangedEvent value)  changed,}){
final _that = this;
switch (_that) {
case PrivacyCoverChangedEvent():
return changed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PrivacyCoverChangedEvent value)?  changed,}){
final _that = this;
switch (_that) {
case PrivacyCoverChangedEvent() when changed != null:
return changed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( bool secure)?  changed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PrivacyCoverChangedEvent() when changed != null:
return changed(_that.secure);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( bool secure)  changed,}) {final _that = this;
switch (_that) {
case PrivacyCoverChangedEvent():
return changed(_that.secure);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( bool secure)?  changed,}) {final _that = this;
switch (_that) {
case PrivacyCoverChangedEvent() when changed != null:
return changed(_that.secure);case _:
  return null;

}
}

}

/// @nodoc


class PrivacyCoverChangedEvent implements PrivacyCoverEvent {
  const PrivacyCoverChangedEvent({required this.secure});
  

@override final  bool secure;

/// Create a copy of PrivacyCoverEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PrivacyCoverChangedEventCopyWith<PrivacyCoverChangedEvent> get copyWith => _$PrivacyCoverChangedEventCopyWithImpl<PrivacyCoverChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PrivacyCoverChangedEvent&&(identical(other.secure, secure) || other.secure == secure));
}


@override
int get hashCode => Object.hash(runtimeType,secure);

@override
String toString() {
  return 'PrivacyCoverEvent.changed(secure: $secure)';
}


}

/// @nodoc
abstract mixin class $PrivacyCoverChangedEventCopyWith<$Res> implements $PrivacyCoverEventCopyWith<$Res> {
  factory $PrivacyCoverChangedEventCopyWith(PrivacyCoverChangedEvent value, $Res Function(PrivacyCoverChangedEvent) _then) = _$PrivacyCoverChangedEventCopyWithImpl;
@override @useResult
$Res call({
 bool secure
});




}
/// @nodoc
class _$PrivacyCoverChangedEventCopyWithImpl<$Res>
    implements $PrivacyCoverChangedEventCopyWith<$Res> {
  _$PrivacyCoverChangedEventCopyWithImpl(this._self, this._then);

  final PrivacyCoverChangedEvent _self;
  final $Res Function(PrivacyCoverChangedEvent) _then;

/// Create a copy of PrivacyCoverEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? secure = null,}) {
  return _then(PrivacyCoverChangedEvent(
secure: null == secure ? _self.secure : secure // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc
mixin _$PrivacyCoverState {

/// Whether the cover is on.
 bool get secure;
/// Create a copy of PrivacyCoverState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PrivacyCoverStateCopyWith<PrivacyCoverState> get copyWith => _$PrivacyCoverStateCopyWithImpl<PrivacyCoverState>(this as PrivacyCoverState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PrivacyCoverState&&(identical(other.secure, secure) || other.secure == secure));
}


@override
int get hashCode => Object.hash(runtimeType,secure);

@override
String toString() {
  return 'PrivacyCoverState(secure: $secure)';
}


}

/// @nodoc
abstract mixin class $PrivacyCoverStateCopyWith<$Res>  {
  factory $PrivacyCoverStateCopyWith(PrivacyCoverState value, $Res Function(PrivacyCoverState) _then) = _$PrivacyCoverStateCopyWithImpl;
@useResult
$Res call({
 bool secure
});




}
/// @nodoc
class _$PrivacyCoverStateCopyWithImpl<$Res>
    implements $PrivacyCoverStateCopyWith<$Res> {
  _$PrivacyCoverStateCopyWithImpl(this._self, this._then);

  final PrivacyCoverState _self;
  final $Res Function(PrivacyCoverState) _then;

/// Create a copy of PrivacyCoverState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? secure = null,}) {
  return _then(_self.copyWith(
secure: null == secure ? _self.secure : secure // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [PrivacyCoverState].
extension PrivacyCoverStatePatterns on PrivacyCoverState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PrivacyCoverState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PrivacyCoverState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PrivacyCoverState value)  $default,){
final _that = this;
switch (_that) {
case _PrivacyCoverState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PrivacyCoverState value)?  $default,){
final _that = this;
switch (_that) {
case _PrivacyCoverState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool secure)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PrivacyCoverState() when $default != null:
return $default(_that.secure);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool secure)  $default,) {final _that = this;
switch (_that) {
case _PrivacyCoverState():
return $default(_that.secure);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool secure)?  $default,) {final _that = this;
switch (_that) {
case _PrivacyCoverState() when $default != null:
return $default(_that.secure);case _:
  return null;

}
}

}

/// @nodoc


class _PrivacyCoverState implements PrivacyCoverState {
  const _PrivacyCoverState({this.secure = false});
  

/// Whether the cover is on.
@override@JsonKey() final  bool secure;

/// Create a copy of PrivacyCoverState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PrivacyCoverStateCopyWith<_PrivacyCoverState> get copyWith => __$PrivacyCoverStateCopyWithImpl<_PrivacyCoverState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PrivacyCoverState&&(identical(other.secure, secure) || other.secure == secure));
}


@override
int get hashCode => Object.hash(runtimeType,secure);

@override
String toString() {
  return 'PrivacyCoverState(secure: $secure)';
}


}

/// @nodoc
abstract mixin class _$PrivacyCoverStateCopyWith<$Res> implements $PrivacyCoverStateCopyWith<$Res> {
  factory _$PrivacyCoverStateCopyWith(_PrivacyCoverState value, $Res Function(_PrivacyCoverState) _then) = __$PrivacyCoverStateCopyWithImpl;
@override @useResult
$Res call({
 bool secure
});




}
/// @nodoc
class __$PrivacyCoverStateCopyWithImpl<$Res>
    implements _$PrivacyCoverStateCopyWith<$Res> {
  __$PrivacyCoverStateCopyWithImpl(this._self, this._then);

  final _PrivacyCoverState _self;
  final $Res Function(_PrivacyCoverState) _then;

/// Create a copy of PrivacyCoverState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? secure = null,}) {
  return _then(_PrivacyCoverState(
secure: null == secure ? _self.secure : secure // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
