// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'default_notebook_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DefaultNotebookEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DefaultNotebookEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DefaultNotebookEvent()';
}


}

/// @nodoc
class $DefaultNotebookEventCopyWith<$Res>  {
$DefaultNotebookEventCopyWith(DefaultNotebookEvent _, $Res Function(DefaultNotebookEvent) __);
}


/// Adds pattern-matching-related methods to [DefaultNotebookEvent].
extension DefaultNotebookEventPatterns on DefaultNotebookEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DefaultNotebookStartedEvent value)?  started,TResult Function( DefaultNotebookChosenEvent value)?  chosen,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DefaultNotebookStartedEvent() when started != null:
return started(_that);case DefaultNotebookChosenEvent() when chosen != null:
return chosen(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DefaultNotebookStartedEvent value)  started,required TResult Function( DefaultNotebookChosenEvent value)  chosen,}){
final _that = this;
switch (_that) {
case DefaultNotebookStartedEvent():
return started(_that);case DefaultNotebookChosenEvent():
return chosen(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DefaultNotebookStartedEvent value)?  started,TResult? Function( DefaultNotebookChosenEvent value)?  chosen,}){
final _that = this;
switch (_that) {
case DefaultNotebookStartedEvent() when started != null:
return started(_that);case DefaultNotebookChosenEvent() when chosen != null:
return chosen(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( String? notebookId)?  chosen,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DefaultNotebookStartedEvent() when started != null:
return started();case DefaultNotebookChosenEvent() when chosen != null:
return chosen(_that.notebookId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( String? notebookId)  chosen,}) {final _that = this;
switch (_that) {
case DefaultNotebookStartedEvent():
return started();case DefaultNotebookChosenEvent():
return chosen(_that.notebookId);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( String? notebookId)?  chosen,}) {final _that = this;
switch (_that) {
case DefaultNotebookStartedEvent() when started != null:
return started();case DefaultNotebookChosenEvent() when chosen != null:
return chosen(_that.notebookId);case _:
  return null;

}
}

}

/// @nodoc


class DefaultNotebookStartedEvent implements DefaultNotebookEvent {
  const DefaultNotebookStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DefaultNotebookStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DefaultNotebookEvent.started()';
}


}




/// @nodoc


class DefaultNotebookChosenEvent implements DefaultNotebookEvent {
  const DefaultNotebookChosenEvent(this.notebookId);
  

 final  String? notebookId;

/// Create a copy of DefaultNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DefaultNotebookChosenEventCopyWith<DefaultNotebookChosenEvent> get copyWith => _$DefaultNotebookChosenEventCopyWithImpl<DefaultNotebookChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DefaultNotebookChosenEvent&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'DefaultNotebookEvent.chosen(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $DefaultNotebookChosenEventCopyWith<$Res> implements $DefaultNotebookEventCopyWith<$Res> {
  factory $DefaultNotebookChosenEventCopyWith(DefaultNotebookChosenEvent value, $Res Function(DefaultNotebookChosenEvent) _then) = _$DefaultNotebookChosenEventCopyWithImpl;
@useResult
$Res call({
 String? notebookId
});




}
/// @nodoc
class _$DefaultNotebookChosenEventCopyWithImpl<$Res>
    implements $DefaultNotebookChosenEventCopyWith<$Res> {
  _$DefaultNotebookChosenEventCopyWithImpl(this._self, this._then);

  final DefaultNotebookChosenEvent _self;
  final $Res Function(DefaultNotebookChosenEvent) _then;

/// Create a copy of DefaultNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notebookId = freezed,}) {
  return _then(DefaultNotebookChosenEvent(
freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc
mixin _$DefaultNotebookState {

/// The notebook chosen; null is the first notebook.
 String? get notebookId;
/// Create a copy of DefaultNotebookState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DefaultNotebookStateCopyWith<DefaultNotebookState> get copyWith => _$DefaultNotebookStateCopyWithImpl<DefaultNotebookState>(this as DefaultNotebookState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DefaultNotebookState&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'DefaultNotebookState(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $DefaultNotebookStateCopyWith<$Res>  {
  factory $DefaultNotebookStateCopyWith(DefaultNotebookState value, $Res Function(DefaultNotebookState) _then) = _$DefaultNotebookStateCopyWithImpl;
@useResult
$Res call({
 String? notebookId
});




}
/// @nodoc
class _$DefaultNotebookStateCopyWithImpl<$Res>
    implements $DefaultNotebookStateCopyWith<$Res> {
  _$DefaultNotebookStateCopyWithImpl(this._self, this._then);

  final DefaultNotebookState _self;
  final $Res Function(DefaultNotebookState) _then;

/// Create a copy of DefaultNotebookState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? notebookId = freezed,}) {
  return _then(_self.copyWith(
notebookId: freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [DefaultNotebookState].
extension DefaultNotebookStatePatterns on DefaultNotebookState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DefaultNotebookState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DefaultNotebookState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DefaultNotebookState value)  $default,){
final _that = this;
switch (_that) {
case _DefaultNotebookState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DefaultNotebookState value)?  $default,){
final _that = this;
switch (_that) {
case _DefaultNotebookState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String? notebookId)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DefaultNotebookState() when $default != null:
return $default(_that.notebookId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String? notebookId)  $default,) {final _that = this;
switch (_that) {
case _DefaultNotebookState():
return $default(_that.notebookId);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String? notebookId)?  $default,) {final _that = this;
switch (_that) {
case _DefaultNotebookState() when $default != null:
return $default(_that.notebookId);case _:
  return null;

}
}

}

/// @nodoc


class _DefaultNotebookState implements DefaultNotebookState {
  const _DefaultNotebookState({this.notebookId});
  

/// The notebook chosen; null is the first notebook.
@override final  String? notebookId;

/// Create a copy of DefaultNotebookState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DefaultNotebookStateCopyWith<_DefaultNotebookState> get copyWith => __$DefaultNotebookStateCopyWithImpl<_DefaultNotebookState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DefaultNotebookState&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'DefaultNotebookState(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class _$DefaultNotebookStateCopyWith<$Res> implements $DefaultNotebookStateCopyWith<$Res> {
  factory _$DefaultNotebookStateCopyWith(_DefaultNotebookState value, $Res Function(_DefaultNotebookState) _then) = __$DefaultNotebookStateCopyWithImpl;
@override @useResult
$Res call({
 String? notebookId
});




}
/// @nodoc
class __$DefaultNotebookStateCopyWithImpl<$Res>
    implements _$DefaultNotebookStateCopyWith<$Res> {
  __$DefaultNotebookStateCopyWithImpl(this._self, this._then);

  final _DefaultNotebookState _self;
  final $Res Function(_DefaultNotebookState) _then;

/// Create a copy of DefaultNotebookState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? notebookId = freezed,}) {
  return _then(_DefaultNotebookState(
notebookId: freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
