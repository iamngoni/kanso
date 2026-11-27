// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'library_pane_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$LibraryPaneEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryPaneEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LibraryPaneEvent()';
}


}

/// @nodoc
class $LibraryPaneEventCopyWith<$Res>  {
$LibraryPaneEventCopyWith(LibraryPaneEvent _, $Res Function(LibraryPaneEvent) __);
}


/// Adds pattern-matching-related methods to [LibraryPaneEvent].
extension LibraryPaneEventPatterns on LibraryPaneEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LibraryPaneToggledEvent value)?  toggled,TResult Function( LibraryPanesToggledEvent value)?  panesToggled,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LibraryPaneToggledEvent() when toggled != null:
return toggled(_that);case LibraryPanesToggledEvent() when panesToggled != null:
return panesToggled(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LibraryPaneToggledEvent value)  toggled,required TResult Function( LibraryPanesToggledEvent value)  panesToggled,}){
final _that = this;
switch (_that) {
case LibraryPaneToggledEvent():
return toggled(_that);case LibraryPanesToggledEvent():
return panesToggled(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LibraryPaneToggledEvent value)?  toggled,TResult? Function( LibraryPanesToggledEvent value)?  panesToggled,}){
final _that = this;
switch (_that) {
case LibraryPaneToggledEvent() when toggled != null:
return toggled(_that);case LibraryPanesToggledEvent() when panesToggled != null:
return panesToggled(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  toggled,TResult Function()?  panesToggled,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LibraryPaneToggledEvent() when toggled != null:
return toggled();case LibraryPanesToggledEvent() when panesToggled != null:
return panesToggled();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  toggled,required TResult Function()  panesToggled,}) {final _that = this;
switch (_that) {
case LibraryPaneToggledEvent():
return toggled();case LibraryPanesToggledEvent():
return panesToggled();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  toggled,TResult? Function()?  panesToggled,}) {final _that = this;
switch (_that) {
case LibraryPaneToggledEvent() when toggled != null:
return toggled();case LibraryPanesToggledEvent() when panesToggled != null:
return panesToggled();case _:
  return null;

}
}

}

/// @nodoc


class LibraryPaneToggledEvent implements LibraryPaneEvent {
  const LibraryPaneToggledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryPaneToggledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LibraryPaneEvent.toggled()';
}


}




/// @nodoc


class LibraryPanesToggledEvent implements LibraryPaneEvent {
  const LibraryPanesToggledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryPanesToggledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LibraryPaneEvent.panesToggled()';
}


}




/// @nodoc
mixin _$LibraryPaneState {

/// Whether the pane is shown beside the list.
 bool get visible;/// Whether the library and the list are both put away while a note is
/// open, so the note has the whole width of a tablet to write in.
 bool get panesHidden;
/// Create a copy of LibraryPaneState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LibraryPaneStateCopyWith<LibraryPaneState> get copyWith => _$LibraryPaneStateCopyWithImpl<LibraryPaneState>(this as LibraryPaneState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryPaneState&&(identical(other.visible, visible) || other.visible == visible)&&(identical(other.panesHidden, panesHidden) || other.panesHidden == panesHidden));
}


@override
int get hashCode => Object.hash(runtimeType,visible,panesHidden);

@override
String toString() {
  return 'LibraryPaneState(visible: $visible, panesHidden: $panesHidden)';
}


}

/// @nodoc
abstract mixin class $LibraryPaneStateCopyWith<$Res>  {
  factory $LibraryPaneStateCopyWith(LibraryPaneState value, $Res Function(LibraryPaneState) _then) = _$LibraryPaneStateCopyWithImpl;
@useResult
$Res call({
 bool visible, bool panesHidden
});




}
/// @nodoc
class _$LibraryPaneStateCopyWithImpl<$Res>
    implements $LibraryPaneStateCopyWith<$Res> {
  _$LibraryPaneStateCopyWithImpl(this._self, this._then);

  final LibraryPaneState _self;
  final $Res Function(LibraryPaneState) _then;

/// Create a copy of LibraryPaneState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? visible = null,Object? panesHidden = null,}) {
  return _then(_self.copyWith(
visible: null == visible ? _self.visible : visible // ignore: cast_nullable_to_non_nullable
as bool,panesHidden: null == panesHidden ? _self.panesHidden : panesHidden // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [LibraryPaneState].
extension LibraryPaneStatePatterns on LibraryPaneState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _LibraryPaneState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _LibraryPaneState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _LibraryPaneState value)  $default,){
final _that = this;
switch (_that) {
case _LibraryPaneState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _LibraryPaneState value)?  $default,){
final _that = this;
switch (_that) {
case _LibraryPaneState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool visible,  bool panesHidden)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _LibraryPaneState() when $default != null:
return $default(_that.visible,_that.panesHidden);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool visible,  bool panesHidden)  $default,) {final _that = this;
switch (_that) {
case _LibraryPaneState():
return $default(_that.visible,_that.panesHidden);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool visible,  bool panesHidden)?  $default,) {final _that = this;
switch (_that) {
case _LibraryPaneState() when $default != null:
return $default(_that.visible,_that.panesHidden);case _:
  return null;

}
}

}

/// @nodoc


class _LibraryPaneState implements LibraryPaneState {
  const _LibraryPaneState({this.visible = true, this.panesHidden = false});
  

/// Whether the pane is shown beside the list.
@override@JsonKey() final  bool visible;
/// Whether the library and the list are both put away while a note is
/// open, so the note has the whole width of a tablet to write in.
@override@JsonKey() final  bool panesHidden;

/// Create a copy of LibraryPaneState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$LibraryPaneStateCopyWith<_LibraryPaneState> get copyWith => __$LibraryPaneStateCopyWithImpl<_LibraryPaneState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _LibraryPaneState&&(identical(other.visible, visible) || other.visible == visible)&&(identical(other.panesHidden, panesHidden) || other.panesHidden == panesHidden));
}


@override
int get hashCode => Object.hash(runtimeType,visible,panesHidden);

@override
String toString() {
  return 'LibraryPaneState(visible: $visible, panesHidden: $panesHidden)';
}


}

/// @nodoc
abstract mixin class _$LibraryPaneStateCopyWith<$Res> implements $LibraryPaneStateCopyWith<$Res> {
  factory _$LibraryPaneStateCopyWith(_LibraryPaneState value, $Res Function(_LibraryPaneState) _then) = __$LibraryPaneStateCopyWithImpl;
@override @useResult
$Res call({
 bool visible, bool panesHidden
});




}
/// @nodoc
class __$LibraryPaneStateCopyWithImpl<$Res>
    implements _$LibraryPaneStateCopyWith<$Res> {
  __$LibraryPaneStateCopyWithImpl(this._self, this._then);

  final _LibraryPaneState _self;
  final $Res Function(_LibraryPaneState) _then;

/// Create a copy of LibraryPaneState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? visible = null,Object? panesHidden = null,}) {
  return _then(_LibraryPaneState(
visible: null == visible ? _self.visible : visible // ignore: cast_nullable_to_non_nullable
as bool,panesHidden: null == panesHidden ? _self.panesHidden : panesHidden // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
