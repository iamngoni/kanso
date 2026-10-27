// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'widget_colors_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$WidgetColorsEvent {

 KansoColors get light; KansoColors get dark;
/// Create a copy of WidgetColorsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WidgetColorsEventCopyWith<WidgetColorsEvent> get copyWith => _$WidgetColorsEventCopyWithImpl<WidgetColorsEvent>(this as WidgetColorsEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WidgetColorsEvent&&(identical(other.light, light) || other.light == light)&&(identical(other.dark, dark) || other.dark == dark));
}


@override
int get hashCode => Object.hash(runtimeType,light,dark);

@override
String toString() {
  return 'WidgetColorsEvent(light: $light, dark: $dark)';
}


}

/// @nodoc
abstract mixin class $WidgetColorsEventCopyWith<$Res>  {
  factory $WidgetColorsEventCopyWith(WidgetColorsEvent value, $Res Function(WidgetColorsEvent) _then) = _$WidgetColorsEventCopyWithImpl;
@useResult
$Res call({
 KansoColors light, KansoColors dark
});




}
/// @nodoc
class _$WidgetColorsEventCopyWithImpl<$Res>
    implements $WidgetColorsEventCopyWith<$Res> {
  _$WidgetColorsEventCopyWithImpl(this._self, this._then);

  final WidgetColorsEvent _self;
  final $Res Function(WidgetColorsEvent) _then;

/// Create a copy of WidgetColorsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? light = null,Object? dark = null,}) {
  return _then(_self.copyWith(
light: null == light ? _self.light : light // ignore: cast_nullable_to_non_nullable
as KansoColors,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as KansoColors,
  ));
}

}


/// Adds pattern-matching-related methods to [WidgetColorsEvent].
extension WidgetColorsEventPatterns on WidgetColorsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( WidgetColorsChangedEvent value)?  changed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case WidgetColorsChangedEvent() when changed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( WidgetColorsChangedEvent value)  changed,}){
final _that = this;
switch (_that) {
case WidgetColorsChangedEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( WidgetColorsChangedEvent value)?  changed,}){
final _that = this;
switch (_that) {
case WidgetColorsChangedEvent() when changed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( KansoColors light,  KansoColors dark)?  changed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case WidgetColorsChangedEvent() when changed != null:
return changed(_that.light,_that.dark);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( KansoColors light,  KansoColors dark)  changed,}) {final _that = this;
switch (_that) {
case WidgetColorsChangedEvent():
return changed(_that.light,_that.dark);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( KansoColors light,  KansoColors dark)?  changed,}) {final _that = this;
switch (_that) {
case WidgetColorsChangedEvent() when changed != null:
return changed(_that.light,_that.dark);case _:
  return null;

}
}

}

/// @nodoc


class WidgetColorsChangedEvent implements WidgetColorsEvent {
  const WidgetColorsChangedEvent({required this.light, required this.dark});
  

@override final  KansoColors light;
@override final  KansoColors dark;

/// Create a copy of WidgetColorsEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WidgetColorsChangedEventCopyWith<WidgetColorsChangedEvent> get copyWith => _$WidgetColorsChangedEventCopyWithImpl<WidgetColorsChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WidgetColorsChangedEvent&&(identical(other.light, light) || other.light == light)&&(identical(other.dark, dark) || other.dark == dark));
}


@override
int get hashCode => Object.hash(runtimeType,light,dark);

@override
String toString() {
  return 'WidgetColorsEvent.changed(light: $light, dark: $dark)';
}


}

/// @nodoc
abstract mixin class $WidgetColorsChangedEventCopyWith<$Res> implements $WidgetColorsEventCopyWith<$Res> {
  factory $WidgetColorsChangedEventCopyWith(WidgetColorsChangedEvent value, $Res Function(WidgetColorsChangedEvent) _then) = _$WidgetColorsChangedEventCopyWithImpl;
@override @useResult
$Res call({
 KansoColors light, KansoColors dark
});




}
/// @nodoc
class _$WidgetColorsChangedEventCopyWithImpl<$Res>
    implements $WidgetColorsChangedEventCopyWith<$Res> {
  _$WidgetColorsChangedEventCopyWithImpl(this._self, this._then);

  final WidgetColorsChangedEvent _self;
  final $Res Function(WidgetColorsChangedEvent) _then;

/// Create a copy of WidgetColorsEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? light = null,Object? dark = null,}) {
  return _then(WidgetColorsChangedEvent(
light: null == light ? _self.light : light // ignore: cast_nullable_to_non_nullable
as KansoColors,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as KansoColors,
  ));
}


}

/// @nodoc
mixin _$WidgetColorsState {

/// How many times the colours were written since the app started.
 int get publishes;/// Why the last attempt failed; null once one works.
 ApplicationError? get error;
/// Create a copy of WidgetColorsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$WidgetColorsStateCopyWith<WidgetColorsState> get copyWith => _$WidgetColorsStateCopyWithImpl<WidgetColorsState>(this as WidgetColorsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is WidgetColorsState&&(identical(other.publishes, publishes) || other.publishes == publishes)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,publishes,error);

@override
String toString() {
  return 'WidgetColorsState(publishes: $publishes, error: $error)';
}


}

/// @nodoc
abstract mixin class $WidgetColorsStateCopyWith<$Res>  {
  factory $WidgetColorsStateCopyWith(WidgetColorsState value, $Res Function(WidgetColorsState) _then) = _$WidgetColorsStateCopyWithImpl;
@useResult
$Res call({
 int publishes, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$WidgetColorsStateCopyWithImpl<$Res>
    implements $WidgetColorsStateCopyWith<$Res> {
  _$WidgetColorsStateCopyWithImpl(this._self, this._then);

  final WidgetColorsState _self;
  final $Res Function(WidgetColorsState) _then;

/// Create a copy of WidgetColorsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? publishes = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
publishes: null == publishes ? _self.publishes : publishes // ignore: cast_nullable_to_non_nullable
as int,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of WidgetColorsState
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


/// Adds pattern-matching-related methods to [WidgetColorsState].
extension WidgetColorsStatePatterns on WidgetColorsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _WidgetColorsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _WidgetColorsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _WidgetColorsState value)  $default,){
final _that = this;
switch (_that) {
case _WidgetColorsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _WidgetColorsState value)?  $default,){
final _that = this;
switch (_that) {
case _WidgetColorsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int publishes,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _WidgetColorsState() when $default != null:
return $default(_that.publishes,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int publishes,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _WidgetColorsState():
return $default(_that.publishes,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int publishes,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _WidgetColorsState() when $default != null:
return $default(_that.publishes,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _WidgetColorsState implements WidgetColorsState {
  const _WidgetColorsState({this.publishes = 0, this.error});
  

/// How many times the colours were written since the app started.
@override@JsonKey() final  int publishes;
/// Why the last attempt failed; null once one works.
@override final  ApplicationError? error;

/// Create a copy of WidgetColorsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$WidgetColorsStateCopyWith<_WidgetColorsState> get copyWith => __$WidgetColorsStateCopyWithImpl<_WidgetColorsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _WidgetColorsState&&(identical(other.publishes, publishes) || other.publishes == publishes)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,publishes,error);

@override
String toString() {
  return 'WidgetColorsState(publishes: $publishes, error: $error)';
}


}

/// @nodoc
abstract mixin class _$WidgetColorsStateCopyWith<$Res> implements $WidgetColorsStateCopyWith<$Res> {
  factory _$WidgetColorsStateCopyWith(_WidgetColorsState value, $Res Function(_WidgetColorsState) _then) = __$WidgetColorsStateCopyWithImpl;
@override @useResult
$Res call({
 int publishes, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$WidgetColorsStateCopyWithImpl<$Res>
    implements _$WidgetColorsStateCopyWith<$Res> {
  __$WidgetColorsStateCopyWithImpl(this._self, this._then);

  final _WidgetColorsState _self;
  final $Res Function(_WidgetColorsState) _then;

/// Create a copy of WidgetColorsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? publishes = null,Object? error = freezed,}) {
  return _then(_WidgetColorsState(
publishes: null == publishes ? _self.publishes : publishes // ignore: cast_nullable_to_non_nullable
as int,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of WidgetColorsState
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
