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





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NewNotebookEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NewNotebookEvent()';
}


}

/// @nodoc
class $NewNotebookEventCopyWith<$Res>  {
$NewNotebookEventCopyWith(NewNotebookEvent _, $Res Function(NewNotebookEvent) __);
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NewNotebookOpenedEvent value)?  opened,TResult Function( NewNotebookSubmittedEvent value)?  submitted,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NewNotebookOpenedEvent() when opened != null:
return opened(_that);case NewNotebookSubmittedEvent() when submitted != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NewNotebookOpenedEvent value)  opened,required TResult Function( NewNotebookSubmittedEvent value)  submitted,}){
final _that = this;
switch (_that) {
case NewNotebookOpenedEvent():
return opened(_that);case NewNotebookSubmittedEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NewNotebookOpenedEvent value)?  opened,TResult? Function( NewNotebookSubmittedEvent value)?  submitted,}){
final _that = this;
switch (_that) {
case NewNotebookOpenedEvent() when opened != null:
return opened(_that);case NewNotebookSubmittedEvent() when submitted != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String? parentId)?  opened,TResult Function( String name,  NewNotebookKind kind)?  submitted,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NewNotebookOpenedEvent() when opened != null:
return opened(_that.parentId);case NewNotebookSubmittedEvent() when submitted != null:
return submitted(_that.name,_that.kind);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String? parentId)  opened,required TResult Function( String name,  NewNotebookKind kind)  submitted,}) {final _that = this;
switch (_that) {
case NewNotebookOpenedEvent():
return opened(_that.parentId);case NewNotebookSubmittedEvent():
return submitted(_that.name,_that.kind);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String? parentId)?  opened,TResult? Function( String name,  NewNotebookKind kind)?  submitted,}) {final _that = this;
switch (_that) {
case NewNotebookOpenedEvent() when opened != null:
return opened(_that.parentId);case NewNotebookSubmittedEvent() when submitted != null:
return submitted(_that.name,_that.kind);case _:
  return null;

}
}

}

/// @nodoc


class NewNotebookOpenedEvent implements NewNotebookEvent {
  const NewNotebookOpenedEvent({this.parentId});
  

 final  String? parentId;

/// Create a copy of NewNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NewNotebookOpenedEventCopyWith<NewNotebookOpenedEvent> get copyWith => _$NewNotebookOpenedEventCopyWithImpl<NewNotebookOpenedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NewNotebookOpenedEvent&&(identical(other.parentId, parentId) || other.parentId == parentId));
}


@override
int get hashCode => Object.hash(runtimeType,parentId);

@override
String toString() {
  return 'NewNotebookEvent.opened(parentId: $parentId)';
}


}

/// @nodoc
abstract mixin class $NewNotebookOpenedEventCopyWith<$Res> implements $NewNotebookEventCopyWith<$Res> {
  factory $NewNotebookOpenedEventCopyWith(NewNotebookOpenedEvent value, $Res Function(NewNotebookOpenedEvent) _then) = _$NewNotebookOpenedEventCopyWithImpl;
@useResult
$Res call({
 String? parentId
});




}
/// @nodoc
class _$NewNotebookOpenedEventCopyWithImpl<$Res>
    implements $NewNotebookOpenedEventCopyWith<$Res> {
  _$NewNotebookOpenedEventCopyWithImpl(this._self, this._then);

  final NewNotebookOpenedEvent _self;
  final $Res Function(NewNotebookOpenedEvent) _then;

/// Create a copy of NewNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? parentId = freezed,}) {
  return _then(NewNotebookOpenedEvent(
parentId: freezed == parentId ? _self.parentId : parentId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class NewNotebookSubmittedEvent implements NewNotebookEvent {
  const NewNotebookSubmittedEvent(this.name, {this.kind = NewNotebookKind.blank});
  

 final  String name;
@JsonKey() final  NewNotebookKind kind;

/// Create a copy of NewNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NewNotebookSubmittedEventCopyWith<NewNotebookSubmittedEvent> get copyWith => _$NewNotebookSubmittedEventCopyWithImpl<NewNotebookSubmittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NewNotebookSubmittedEvent&&(identical(other.name, name) || other.name == name)&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,name,kind);

@override
String toString() {
  return 'NewNotebookEvent.submitted(name: $name, kind: $kind)';
}


}

/// @nodoc
abstract mixin class $NewNotebookSubmittedEventCopyWith<$Res> implements $NewNotebookEventCopyWith<$Res> {
  factory $NewNotebookSubmittedEventCopyWith(NewNotebookSubmittedEvent value, $Res Function(NewNotebookSubmittedEvent) _then) = _$NewNotebookSubmittedEventCopyWithImpl;
@useResult
$Res call({
 String name, NewNotebookKind kind
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
@pragma('vm:prefer-inline') $Res call({Object? name = null,Object? kind = null,}) {
  return _then(NewNotebookSubmittedEvent(
null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as NewNotebookKind,
  ));
}


}

/// @nodoc
mixin _$NewNotebookState {

/// What a notebook can be made from, in order; empty until the sheet
/// has opened.
 List<StarterInfo> get starters;/// The notebook the new one is made inside, if any.
 String? get parentId;/// Whether [parentId] is locked, or inside a locked notebook: the
/// starters that make notes can't be made there.
 bool get parentLocked;/// Whether the notebook is being made.
 bool get creating;/// The notebook made, once it is.
 MadeNotebook? get created;/// Why none was made.
 NewNotebookFailure? get failure;
/// Create a copy of NewNotebookState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NewNotebookStateCopyWith<NewNotebookState> get copyWith => _$NewNotebookStateCopyWithImpl<NewNotebookState>(this as NewNotebookState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NewNotebookState&&const DeepCollectionEquality().equals(other.starters, starters)&&(identical(other.parentId, parentId) || other.parentId == parentId)&&(identical(other.parentLocked, parentLocked) || other.parentLocked == parentLocked)&&(identical(other.creating, creating) || other.creating == creating)&&(identical(other.created, created) || other.created == created)&&(identical(other.failure, failure) || other.failure == failure));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(starters),parentId,parentLocked,creating,created,failure);

@override
String toString() {
  return 'NewNotebookState(starters: $starters, parentId: $parentId, parentLocked: $parentLocked, creating: $creating, created: $created, failure: $failure)';
}


}

/// @nodoc
abstract mixin class $NewNotebookStateCopyWith<$Res>  {
  factory $NewNotebookStateCopyWith(NewNotebookState value, $Res Function(NewNotebookState) _then) = _$NewNotebookStateCopyWithImpl;
@useResult
$Res call({
 List<StarterInfo> starters, String? parentId, bool parentLocked, bool creating, MadeNotebook? created, NewNotebookFailure? failure
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
@pragma('vm:prefer-inline') @override $Res call({Object? starters = null,Object? parentId = freezed,Object? parentLocked = null,Object? creating = null,Object? created = freezed,Object? failure = freezed,}) {
  return _then(_self.copyWith(
starters: null == starters ? _self.starters : starters // ignore: cast_nullable_to_non_nullable
as List<StarterInfo>,parentId: freezed == parentId ? _self.parentId : parentId // ignore: cast_nullable_to_non_nullable
as String?,parentLocked: null == parentLocked ? _self.parentLocked : parentLocked // ignore: cast_nullable_to_non_nullable
as bool,creating: null == creating ? _self.creating : creating // ignore: cast_nullable_to_non_nullable
as bool,created: freezed == created ? _self.created : created // ignore: cast_nullable_to_non_nullable
as MadeNotebook?,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<StarterInfo> starters,  String? parentId,  bool parentLocked,  bool creating,  MadeNotebook? created,  NewNotebookFailure? failure)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NewNotebookState() when $default != null:
return $default(_that.starters,_that.parentId,_that.parentLocked,_that.creating,_that.created,_that.failure);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<StarterInfo> starters,  String? parentId,  bool parentLocked,  bool creating,  MadeNotebook? created,  NewNotebookFailure? failure)  $default,) {final _that = this;
switch (_that) {
case _NewNotebookState():
return $default(_that.starters,_that.parentId,_that.parentLocked,_that.creating,_that.created,_that.failure);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<StarterInfo> starters,  String? parentId,  bool parentLocked,  bool creating,  MadeNotebook? created,  NewNotebookFailure? failure)?  $default,) {final _that = this;
switch (_that) {
case _NewNotebookState() when $default != null:
return $default(_that.starters,_that.parentId,_that.parentLocked,_that.creating,_that.created,_that.failure);case _:
  return null;

}
}

}

/// @nodoc


class _NewNotebookState implements NewNotebookState {
  const _NewNotebookState({final  List<StarterInfo> starters = const <StarterInfo>[], this.parentId, this.parentLocked = false, this.creating = false, this.created, this.failure}): _starters = starters;
  

/// What a notebook can be made from, in order; empty until the sheet
/// has opened.
 final  List<StarterInfo> _starters;
/// What a notebook can be made from, in order; empty until the sheet
/// has opened.
@override@JsonKey() List<StarterInfo> get starters {
  if (_starters is EqualUnmodifiableListView) return _starters;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_starters);
}

/// The notebook the new one is made inside, if any.
@override final  String? parentId;
/// Whether [parentId] is locked, or inside a locked notebook: the
/// starters that make notes can't be made there.
@override@JsonKey() final  bool parentLocked;
/// Whether the notebook is being made.
@override@JsonKey() final  bool creating;
/// The notebook made, once it is.
@override final  MadeNotebook? created;
/// Why none was made.
@override final  NewNotebookFailure? failure;

/// Create a copy of NewNotebookState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NewNotebookStateCopyWith<_NewNotebookState> get copyWith => __$NewNotebookStateCopyWithImpl<_NewNotebookState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NewNotebookState&&const DeepCollectionEquality().equals(other._starters, _starters)&&(identical(other.parentId, parentId) || other.parentId == parentId)&&(identical(other.parentLocked, parentLocked) || other.parentLocked == parentLocked)&&(identical(other.creating, creating) || other.creating == creating)&&(identical(other.created, created) || other.created == created)&&(identical(other.failure, failure) || other.failure == failure));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_starters),parentId,parentLocked,creating,created,failure);

@override
String toString() {
  return 'NewNotebookState(starters: $starters, parentId: $parentId, parentLocked: $parentLocked, creating: $creating, created: $created, failure: $failure)';
}


}

/// @nodoc
abstract mixin class _$NewNotebookStateCopyWith<$Res> implements $NewNotebookStateCopyWith<$Res> {
  factory _$NewNotebookStateCopyWith(_NewNotebookState value, $Res Function(_NewNotebookState) _then) = __$NewNotebookStateCopyWithImpl;
@override @useResult
$Res call({
 List<StarterInfo> starters, String? parentId, bool parentLocked, bool creating, MadeNotebook? created, NewNotebookFailure? failure
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
@override @pragma('vm:prefer-inline') $Res call({Object? starters = null,Object? parentId = freezed,Object? parentLocked = null,Object? creating = null,Object? created = freezed,Object? failure = freezed,}) {
  return _then(_NewNotebookState(
starters: null == starters ? _self._starters : starters // ignore: cast_nullable_to_non_nullable
as List<StarterInfo>,parentId: freezed == parentId ? _self.parentId : parentId // ignore: cast_nullable_to_non_nullable
as String?,parentLocked: null == parentLocked ? _self.parentLocked : parentLocked // ignore: cast_nullable_to_non_nullable
as bool,creating: null == creating ? _self.creating : creating // ignore: cast_nullable_to_non_nullable
as bool,created: freezed == created ? _self.created : created // ignore: cast_nullable_to_non_nullable
as MadeNotebook?,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as NewNotebookFailure?,
  ));
}


}

// dart format on
