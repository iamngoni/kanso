// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'new_notebook_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NewNotebookEvent {

 String get name;
/// Create a copy of NewNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NewNotebookEventCopyWith<NewNotebookEvent> get copyWith => _$NewNotebookEventCopyWithImpl<NewNotebookEvent>(this as NewNotebookEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NewNotebookEvent&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,name);

@override
String toString() {
  return 'NewNotebookEvent(name: $name)';
}


}

/// @nodoc
abstract mixin class $NewNotebookEventCopyWith<$Res>  {
  factory $NewNotebookEventCopyWith(NewNotebookEvent value, $Res Function(NewNotebookEvent) _then) = _$NewNotebookEventCopyWithImpl;
@useResult
$Res call({
 String name
});




}
/// @nodoc
class _$NewNotebookEventCopyWithImpl<$Res>
    implements $NewNotebookEventCopyWith<$Res> {
  _$NewNotebookEventCopyWithImpl(this._self, this._then);

  final NewNotebookEvent _self;
  final $Res Function(NewNotebookEvent) _then;

/// Create a copy of NewNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? name = null,}) {
  return _then(_self.copyWith(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [NewNotebookEvent].
extension NewNotebookEventPatterns on NewNotebookEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NewNotebookSubmittedEvent value)?  submitted,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NewNotebookSubmittedEvent() when submitted != null:
return submitted(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NewNotebookSubmittedEvent value)  submitted,}){
final _that = this;
switch (_that) {
case NewNotebookSubmittedEvent():
return submitted(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NewNotebookSubmittedEvent value)?  submitted,}){
final _that = this;
switch (_that) {
case NewNotebookSubmittedEvent() when submitted != null:
return submitted(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String name)?  submitted,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NewNotebookSubmittedEvent() when submitted != null:
return submitted(_that.name);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String name)  submitted,}) {final _that = this;
switch (_that) {
case NewNotebookSubmittedEvent():
return submitted(_that.name);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String name)?  submitted,}) {final _that = this;
switch (_that) {
case NewNotebookSubmittedEvent() when submitted != null:
return submitted(_that.name);case _:
  return null;

}
}

}

/// @nodoc


class NewNotebookSubmittedEvent implements NewNotebookEvent {
  const NewNotebookSubmittedEvent(this.name);
  

@override final  String name;

/// Create a copy of NewNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NewNotebookSubmittedEventCopyWith<NewNotebookSubmittedEvent> get copyWith => _$NewNotebookSubmittedEventCopyWithImpl<NewNotebookSubmittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NewNotebookSubmittedEvent&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,name);

@override
String toString() {
  return 'NewNotebookEvent.submitted(name: $name)';
}


}

/// @nodoc
abstract mixin class $NewNotebookSubmittedEventCopyWith<$Res> implements $NewNotebookEventCopyWith<$Res> {
  factory $NewNotebookSubmittedEventCopyWith(NewNotebookSubmittedEvent value, $Res Function(NewNotebookSubmittedEvent) _then) = _$NewNotebookSubmittedEventCopyWithImpl;
@override @useResult
$Res call({
 String name
});




}
/// @nodoc
class _$NewNotebookSubmittedEventCopyWithImpl<$Res>
    implements $NewNotebookSubmittedEventCopyWith<$Res> {
  _$NewNotebookSubmittedEventCopyWithImpl(this._self, this._then);

  final NewNotebookSubmittedEvent _self;
  final $Res Function(NewNotebookSubmittedEvent) _then;

/// Create a copy of NewNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? name = null,}) {
  return _then(NewNotebookSubmittedEvent(
null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$NewNotebookState {

/// Whether the notebook is being made.
 bool get creating;/// The notebook made, once it is.
 NotebookRow? get created;/// Why none was made.
 NewNotebookFailure? get failure;
/// Create a copy of NewNotebookState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NewNotebookStateCopyWith<NewNotebookState> get copyWith => _$NewNotebookStateCopyWithImpl<NewNotebookState>(this as NewNotebookState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NewNotebookState&&(identical(other.creating, creating) || other.creating == creating)&&(identical(other.created, created) || other.created == created)&&(identical(other.failure, failure) || other.failure == failure));
}


@override
int get hashCode => Object.hash(runtimeType,creating,created,failure);

@override
String toString() {
  return 'NewNotebookState(creating: $creating, created: $created, failure: $failure)';
}


}

/// @nodoc
abstract mixin class $NewNotebookStateCopyWith<$Res>  {
  factory $NewNotebookStateCopyWith(NewNotebookState value, $Res Function(NewNotebookState) _then) = _$NewNotebookStateCopyWithImpl;
@useResult
$Res call({
 bool creating, NotebookRow? created, NewNotebookFailure? failure
});




}
/// @nodoc
class _$NewNotebookStateCopyWithImpl<$Res>
    implements $NewNotebookStateCopyWith<$Res> {
  _$NewNotebookStateCopyWithImpl(this._self, this._then);

  final NewNotebookState _self;
  final $Res Function(NewNotebookState) _then;

/// Create a copy of NewNotebookState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? creating = null,Object? created = freezed,Object? failure = freezed,}) {
  return _then(_self.copyWith(
creating: null == creating ? _self.creating : creating // ignore: cast_nullable_to_non_nullable
as bool,created: freezed == created ? _self.created : created // ignore: cast_nullable_to_non_nullable
as NotebookRow?,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as NewNotebookFailure?,
  ));
}

}


/// Adds pattern-matching-related methods to [NewNotebookState].
extension NewNotebookStatePatterns on NewNotebookState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NewNotebookState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NewNotebookState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NewNotebookState value)  $default,){
final _that = this;
switch (_that) {
case _NewNotebookState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NewNotebookState value)?  $default,){
final _that = this;
switch (_that) {
case _NewNotebookState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool creating,  NotebookRow? created,  NewNotebookFailure? failure)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NewNotebookState() when $default != null:
return $default(_that.creating,_that.created,_that.failure);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool creating,  NotebookRow? created,  NewNotebookFailure? failure)  $default,) {final _that = this;
switch (_that) {
case _NewNotebookState():
return $default(_that.creating,_that.created,_that.failure);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool creating,  NotebookRow? created,  NewNotebookFailure? failure)?  $default,) {final _that = this;
switch (_that) {
case _NewNotebookState() when $default != null:
return $default(_that.creating,_that.created,_that.failure);case _:
  return null;

}
}

}

/// @nodoc


class _NewNotebookState implements NewNotebookState {
  const _NewNotebookState({this.creating = false, this.created, this.failure});
  

/// Whether the notebook is being made.
@override@JsonKey() final  bool creating;
/// The notebook made, once it is.
@override final  NotebookRow? created;
/// Why none was made.
@override final  NewNotebookFailure? failure;

/// Create a copy of NewNotebookState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NewNotebookStateCopyWith<_NewNotebookState> get copyWith => __$NewNotebookStateCopyWithImpl<_NewNotebookState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NewNotebookState&&(identical(other.creating, creating) || other.creating == creating)&&(identical(other.created, created) || other.created == created)&&(identical(other.failure, failure) || other.failure == failure));
}


@override
int get hashCode => Object.hash(runtimeType,creating,created,failure);

@override
String toString() {
  return 'NewNotebookState(creating: $creating, created: $created, failure: $failure)';
}


}

/// @nodoc
abstract mixin class _$NewNotebookStateCopyWith<$Res> implements $NewNotebookStateCopyWith<$Res> {
  factory _$NewNotebookStateCopyWith(_NewNotebookState value, $Res Function(_NewNotebookState) _then) = __$NewNotebookStateCopyWithImpl;
@override @useResult
$Res call({
 bool creating, NotebookRow? created, NewNotebookFailure? failure
});




}
/// @nodoc
class __$NewNotebookStateCopyWithImpl<$Res>
    implements _$NewNotebookStateCopyWith<$Res> {
  __$NewNotebookStateCopyWithImpl(this._self, this._then);

  final _NewNotebookState _self;
  final $Res Function(_NewNotebookState) _then;

/// Create a copy of NewNotebookState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? creating = null,Object? created = freezed,Object? failure = freezed,}) {
  return _then(_NewNotebookState(
creating: null == creating ? _self.creating : creating // ignore: cast_nullable_to_non_nullable
as bool,created: freezed == created ? _self.created : created // ignore: cast_nullable_to_non_nullable
as NotebookRow?,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as NewNotebookFailure?,
  ));
}


}

// dart format on
