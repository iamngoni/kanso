// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'icon_themes_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$IconThemesEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IconThemesEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'IconThemesEvent()';
}


}

/// @nodoc
class $IconThemesEventCopyWith<$Res>  {
$IconThemesEventCopyWith(IconThemesEvent _, $Res Function(IconThemesEvent) __);
}


/// Adds pattern-matching-related methods to [IconThemesEvent].
extension IconThemesEventPatterns on IconThemesEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( IconThemesStartedEvent value)?  started,required TResult orElse(),}){
final _that = this;
switch (_that) {
case IconThemesStartedEvent() when started != null:
return started(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( IconThemesStartedEvent value)  started,}){
final _that = this;
switch (_that) {
case IconThemesStartedEvent():
return started(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( IconThemesStartedEvent value)?  started,}){
final _that = this;
switch (_that) {
case IconThemesStartedEvent() when started != null:
return started(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,required TResult orElse(),}) {final _that = this;
switch (_that) {
case IconThemesStartedEvent() when started != null:
return started();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,}) {final _that = this;
switch (_that) {
case IconThemesStartedEvent():
return started();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,}) {final _that = this;
switch (_that) {
case IconThemesStartedEvent() when started != null:
return started();case _:
  return null;

}
}

}

/// @nodoc


class IconThemesStartedEvent implements IconThemesEvent {
  const IconThemesStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IconThemesStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'IconThemesEvent.started()';
}


}




/// @nodoc
mixin _$IconThemesState {

 LoadStatus get status; List<IconThemeRow> get themes;
/// Create a copy of IconThemesState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$IconThemesStateCopyWith<IconThemesState> get copyWith => _$IconThemesStateCopyWithImpl<IconThemesState>(this as IconThemesState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IconThemesState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.themes, themes));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(themes));

@override
String toString() {
  return 'IconThemesState(status: $status, themes: $themes)';
}


}

/// @nodoc
abstract mixin class $IconThemesStateCopyWith<$Res>  {
  factory $IconThemesStateCopyWith(IconThemesState value, $Res Function(IconThemesState) _then) = _$IconThemesStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<IconThemeRow> themes
});




}
/// @nodoc
class _$IconThemesStateCopyWithImpl<$Res>
    implements $IconThemesStateCopyWith<$Res> {
  _$IconThemesStateCopyWithImpl(this._self, this._then);

  final IconThemesState _self;
  final $Res Function(IconThemesState) _then;

/// Create a copy of IconThemesState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? themes = null,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,themes: null == themes ? _self.themes : themes // ignore: cast_nullable_to_non_nullable
as List<IconThemeRow>,
  ));
}

}


/// Adds pattern-matching-related methods to [IconThemesState].
extension IconThemesStatePatterns on IconThemesState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _IconThemesState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _IconThemesState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _IconThemesState value)  $default,){
final _that = this;
switch (_that) {
case _IconThemesState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _IconThemesState value)?  $default,){
final _that = this;
switch (_that) {
case _IconThemesState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<IconThemeRow> themes)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _IconThemesState() when $default != null:
return $default(_that.status,_that.themes);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<IconThemeRow> themes)  $default,) {final _that = this;
switch (_that) {
case _IconThemesState():
return $default(_that.status,_that.themes);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<IconThemeRow> themes)?  $default,) {final _that = this;
switch (_that) {
case _IconThemesState() when $default != null:
return $default(_that.status,_that.themes);case _:
  return null;

}
}

}

/// @nodoc


class _IconThemesState implements IconThemesState {
  const _IconThemesState({this.status = LoadStatus.loading, final  List<IconThemeRow> themes = const <IconThemeRow>[]}): _themes = themes;
  

@override@JsonKey() final  LoadStatus status;
 final  List<IconThemeRow> _themes;
@override@JsonKey() List<IconThemeRow> get themes {
  if (_themes is EqualUnmodifiableListView) return _themes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_themes);
}


/// Create a copy of IconThemesState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$IconThemesStateCopyWith<_IconThemesState> get copyWith => __$IconThemesStateCopyWithImpl<_IconThemesState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _IconThemesState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._themes, _themes));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_themes));

@override
String toString() {
  return 'IconThemesState(status: $status, themes: $themes)';
}


}

/// @nodoc
abstract mixin class _$IconThemesStateCopyWith<$Res> implements $IconThemesStateCopyWith<$Res> {
  factory _$IconThemesStateCopyWith(_IconThemesState value, $Res Function(_IconThemesState) _then) = __$IconThemesStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<IconThemeRow> themes
});




}
/// @nodoc
class __$IconThemesStateCopyWithImpl<$Res>
    implements _$IconThemesStateCopyWith<$Res> {
  __$IconThemesStateCopyWithImpl(this._self, this._then);

  final _IconThemesState _self;
  final $Res Function(_IconThemesState) _then;

/// Create a copy of IconThemesState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? themes = null,}) {
  return _then(_IconThemesState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,themes: null == themes ? _self._themes : themes // ignore: cast_nullable_to_non_nullable
as List<IconThemeRow>,
  ));
}


}

// dart format on
