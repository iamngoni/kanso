// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'agent_review_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AgentReviewEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentReviewEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentReviewEvent()';
}


}

/// @nodoc
class $AgentReviewEventCopyWith<$Res>  {
$AgentReviewEventCopyWith(AgentReviewEvent _, $Res Function(AgentReviewEvent) __);
}


/// Adds pattern-matching-related methods to [AgentReviewEvent].
extension AgentReviewEventPatterns on AgentReviewEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AgentReviewOpenedEvent value)?  opened,TResult Function( AgentReviewCardToggledEvent value)?  cardToggled,TResult Function( AgentReviewKeepToggledEvent value)?  keepToggled,TResult Function( AgentReviewAppliedEvent value)?  applied,TResult Function( AgentReviewUndoneEvent value)?  undone,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AgentReviewOpenedEvent() when opened != null:
return opened(_that);case AgentReviewCardToggledEvent() when cardToggled != null:
return cardToggled(_that);case AgentReviewKeepToggledEvent() when keepToggled != null:
return keepToggled(_that);case AgentReviewAppliedEvent() when applied != null:
return applied(_that);case AgentReviewUndoneEvent() when undone != null:
return undone(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AgentReviewOpenedEvent value)  opened,required TResult Function( AgentReviewCardToggledEvent value)  cardToggled,required TResult Function( AgentReviewKeepToggledEvent value)  keepToggled,required TResult Function( AgentReviewAppliedEvent value)  applied,required TResult Function( AgentReviewUndoneEvent value)  undone,}){
final _that = this;
switch (_that) {
case AgentReviewOpenedEvent():
return opened(_that);case AgentReviewCardToggledEvent():
return cardToggled(_that);case AgentReviewKeepToggledEvent():
return keepToggled(_that);case AgentReviewAppliedEvent():
return applied(_that);case AgentReviewUndoneEvent():
return undone(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AgentReviewOpenedEvent value)?  opened,TResult? Function( AgentReviewCardToggledEvent value)?  cardToggled,TResult? Function( AgentReviewKeepToggledEvent value)?  keepToggled,TResult? Function( AgentReviewAppliedEvent value)?  applied,TResult? Function( AgentReviewUndoneEvent value)?  undone,}){
final _that = this;
switch (_that) {
case AgentReviewOpenedEvent() when opened != null:
return opened(_that);case AgentReviewCardToggledEvent() when cardToggled != null:
return cardToggled(_that);case AgentReviewKeepToggledEvent() when keepToggled != null:
return keepToggled(_that);case AgentReviewAppliedEvent() when applied != null:
return applied(_that);case AgentReviewUndoneEvent() when undone != null:
return undone(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( int task)?  opened,TResult Function( int index,  bool on)?  cardToggled,TResult Function( int index,  bool on)?  keepToggled,TResult Function()?  applied,TResult Function()?  undone,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AgentReviewOpenedEvent() when opened != null:
return opened(_that.task);case AgentReviewCardToggledEvent() when cardToggled != null:
return cardToggled(_that.index,_that.on);case AgentReviewKeepToggledEvent() when keepToggled != null:
return keepToggled(_that.index,_that.on);case AgentReviewAppliedEvent() when applied != null:
return applied();case AgentReviewUndoneEvent() when undone != null:
return undone();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( int task)  opened,required TResult Function( int index,  bool on)  cardToggled,required TResult Function( int index,  bool on)  keepToggled,required TResult Function()  applied,required TResult Function()  undone,}) {final _that = this;
switch (_that) {
case AgentReviewOpenedEvent():
return opened(_that.task);case AgentReviewCardToggledEvent():
return cardToggled(_that.index,_that.on);case AgentReviewKeepToggledEvent():
return keepToggled(_that.index,_that.on);case AgentReviewAppliedEvent():
return applied();case AgentReviewUndoneEvent():
return undone();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( int task)?  opened,TResult? Function( int index,  bool on)?  cardToggled,TResult? Function( int index,  bool on)?  keepToggled,TResult? Function()?  applied,TResult? Function()?  undone,}) {final _that = this;
switch (_that) {
case AgentReviewOpenedEvent() when opened != null:
return opened(_that.task);case AgentReviewCardToggledEvent() when cardToggled != null:
return cardToggled(_that.index,_that.on);case AgentReviewKeepToggledEvent() when keepToggled != null:
return keepToggled(_that.index,_that.on);case AgentReviewAppliedEvent() when applied != null:
return applied();case AgentReviewUndoneEvent() when undone != null:
return undone();case _:
  return null;

}
}

}

/// @nodoc


class AgentReviewOpenedEvent implements AgentReviewEvent {
  const AgentReviewOpenedEvent(this.task);
  

 final  int task;

/// Create a copy of AgentReviewEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentReviewOpenedEventCopyWith<AgentReviewOpenedEvent> get copyWith => _$AgentReviewOpenedEventCopyWithImpl<AgentReviewOpenedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentReviewOpenedEvent&&(identical(other.task, task) || other.task == task));
}


@override
int get hashCode => Object.hash(runtimeType,task);

@override
String toString() {
  return 'AgentReviewEvent.opened(task: $task)';
}


}

/// @nodoc
abstract mixin class $AgentReviewOpenedEventCopyWith<$Res> implements $AgentReviewEventCopyWith<$Res> {
  factory $AgentReviewOpenedEventCopyWith(AgentReviewOpenedEvent value, $Res Function(AgentReviewOpenedEvent) _then) = _$AgentReviewOpenedEventCopyWithImpl;
@useResult
$Res call({
 int task
});




}
/// @nodoc
class _$AgentReviewOpenedEventCopyWithImpl<$Res>
    implements $AgentReviewOpenedEventCopyWith<$Res> {
  _$AgentReviewOpenedEventCopyWithImpl(this._self, this._then);

  final AgentReviewOpenedEvent _self;
  final $Res Function(AgentReviewOpenedEvent) _then;

/// Create a copy of AgentReviewEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? task = null,}) {
  return _then(AgentReviewOpenedEvent(
null == task ? _self.task : task // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class AgentReviewCardToggledEvent implements AgentReviewEvent {
  const AgentReviewCardToggledEvent(this.index, {required this.on});
  

 final  int index;
 final  bool on;

/// Create a copy of AgentReviewEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentReviewCardToggledEventCopyWith<AgentReviewCardToggledEvent> get copyWith => _$AgentReviewCardToggledEventCopyWithImpl<AgentReviewCardToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentReviewCardToggledEvent&&(identical(other.index, index) || other.index == index)&&(identical(other.on, on) || other.on == on));
}


@override
int get hashCode => Object.hash(runtimeType,index,on);

@override
String toString() {
  return 'AgentReviewEvent.cardToggled(index: $index, on: $on)';
}


}

/// @nodoc
abstract mixin class $AgentReviewCardToggledEventCopyWith<$Res> implements $AgentReviewEventCopyWith<$Res> {
  factory $AgentReviewCardToggledEventCopyWith(AgentReviewCardToggledEvent value, $Res Function(AgentReviewCardToggledEvent) _then) = _$AgentReviewCardToggledEventCopyWithImpl;
@useResult
$Res call({
 int index, bool on
});




}
/// @nodoc
class _$AgentReviewCardToggledEventCopyWithImpl<$Res>
    implements $AgentReviewCardToggledEventCopyWith<$Res> {
  _$AgentReviewCardToggledEventCopyWithImpl(this._self, this._then);

  final AgentReviewCardToggledEvent _self;
  final $Res Function(AgentReviewCardToggledEvent) _then;

/// Create a copy of AgentReviewEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? index = null,Object? on = null,}) {
  return _then(AgentReviewCardToggledEvent(
null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,on: null == on ? _self.on : on // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class AgentReviewKeepToggledEvent implements AgentReviewEvent {
  const AgentReviewKeepToggledEvent(this.index, {required this.on});
  

 final  int index;
 final  bool on;

/// Create a copy of AgentReviewEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentReviewKeepToggledEventCopyWith<AgentReviewKeepToggledEvent> get copyWith => _$AgentReviewKeepToggledEventCopyWithImpl<AgentReviewKeepToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentReviewKeepToggledEvent&&(identical(other.index, index) || other.index == index)&&(identical(other.on, on) || other.on == on));
}


@override
int get hashCode => Object.hash(runtimeType,index,on);

@override
String toString() {
  return 'AgentReviewEvent.keepToggled(index: $index, on: $on)';
}


}

/// @nodoc
abstract mixin class $AgentReviewKeepToggledEventCopyWith<$Res> implements $AgentReviewEventCopyWith<$Res> {
  factory $AgentReviewKeepToggledEventCopyWith(AgentReviewKeepToggledEvent value, $Res Function(AgentReviewKeepToggledEvent) _then) = _$AgentReviewKeepToggledEventCopyWithImpl;
@useResult
$Res call({
 int index, bool on
});




}
/// @nodoc
class _$AgentReviewKeepToggledEventCopyWithImpl<$Res>
    implements $AgentReviewKeepToggledEventCopyWith<$Res> {
  _$AgentReviewKeepToggledEventCopyWithImpl(this._self, this._then);

  final AgentReviewKeepToggledEvent _self;
  final $Res Function(AgentReviewKeepToggledEvent) _then;

/// Create a copy of AgentReviewEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? index = null,Object? on = null,}) {
  return _then(AgentReviewKeepToggledEvent(
null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,on: null == on ? _self.on : on // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class AgentReviewAppliedEvent implements AgentReviewEvent {
  const AgentReviewAppliedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentReviewAppliedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentReviewEvent.applied()';
}


}




/// @nodoc


class AgentReviewUndoneEvent implements AgentReviewEvent {
  const AgentReviewUndoneEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentReviewUndoneEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentReviewEvent.undone()';
}


}




/// @nodoc
mixin _$AgentReviewState {

 int get task; LoadStatus get status; AgentReview? get review; Set<int> get ticked; Set<int> get keep; bool get applying;/// The sentence to show once applied ("Applied 3 changes…"); Undo is
/// offered while it is set.
 String? get applied;/// The sentence to show once undone.
 String? get undone; String? get trouble;
/// Create a copy of AgentReviewState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentReviewStateCopyWith<AgentReviewState> get copyWith => _$AgentReviewStateCopyWithImpl<AgentReviewState>(this as AgentReviewState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentReviewState&&(identical(other.task, task) || other.task == task)&&(identical(other.status, status) || other.status == status)&&(identical(other.review, review) || other.review == review)&&const DeepCollectionEquality().equals(other.ticked, ticked)&&const DeepCollectionEquality().equals(other.keep, keep)&&(identical(other.applying, applying) || other.applying == applying)&&(identical(other.applied, applied) || other.applied == applied)&&(identical(other.undone, undone) || other.undone == undone)&&(identical(other.trouble, trouble) || other.trouble == trouble));
}


@override
int get hashCode => Object.hash(runtimeType,task,status,review,const DeepCollectionEquality().hash(ticked),const DeepCollectionEquality().hash(keep),applying,applied,undone,trouble);

@override
String toString() {
  return 'AgentReviewState(task: $task, status: $status, review: $review, ticked: $ticked, keep: $keep, applying: $applying, applied: $applied, undone: $undone, trouble: $trouble)';
}


}

/// @nodoc
abstract mixin class $AgentReviewStateCopyWith<$Res>  {
  factory $AgentReviewStateCopyWith(AgentReviewState value, $Res Function(AgentReviewState) _then) = _$AgentReviewStateCopyWithImpl;
@useResult
$Res call({
 int task, LoadStatus status, AgentReview? review, Set<int> ticked, Set<int> keep, bool applying, String? applied, String? undone, String? trouble
});


$AgentReviewCopyWith<$Res>? get review;

}
/// @nodoc
class _$AgentReviewStateCopyWithImpl<$Res>
    implements $AgentReviewStateCopyWith<$Res> {
  _$AgentReviewStateCopyWithImpl(this._self, this._then);

  final AgentReviewState _self;
  final $Res Function(AgentReviewState) _then;

/// Create a copy of AgentReviewState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? task = null,Object? status = null,Object? review = freezed,Object? ticked = null,Object? keep = null,Object? applying = null,Object? applied = freezed,Object? undone = freezed,Object? trouble = freezed,}) {
  return _then(_self.copyWith(
task: null == task ? _self.task : task // ignore: cast_nullable_to_non_nullable
as int,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,review: freezed == review ? _self.review : review // ignore: cast_nullable_to_non_nullable
as AgentReview?,ticked: null == ticked ? _self.ticked : ticked // ignore: cast_nullable_to_non_nullable
as Set<int>,keep: null == keep ? _self.keep : keep // ignore: cast_nullable_to_non_nullable
as Set<int>,applying: null == applying ? _self.applying : applying // ignore: cast_nullable_to_non_nullable
as bool,applied: freezed == applied ? _self.applied : applied // ignore: cast_nullable_to_non_nullable
as String?,undone: freezed == undone ? _self.undone : undone // ignore: cast_nullable_to_non_nullable
as String?,trouble: freezed == trouble ? _self.trouble : trouble // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}
/// Create a copy of AgentReviewState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AgentReviewCopyWith<$Res>? get review {
    if (_self.review == null) {
    return null;
  }

  return $AgentReviewCopyWith<$Res>(_self.review!, (value) {
    return _then(_self.copyWith(review: value));
  });
}
}


/// Adds pattern-matching-related methods to [AgentReviewState].
extension AgentReviewStatePatterns on AgentReviewState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AgentReviewState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AgentReviewState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AgentReviewState value)  $default,){
final _that = this;
switch (_that) {
case _AgentReviewState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AgentReviewState value)?  $default,){
final _that = this;
switch (_that) {
case _AgentReviewState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int task,  LoadStatus status,  AgentReview? review,  Set<int> ticked,  Set<int> keep,  bool applying,  String? applied,  String? undone,  String? trouble)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AgentReviewState() when $default != null:
return $default(_that.task,_that.status,_that.review,_that.ticked,_that.keep,_that.applying,_that.applied,_that.undone,_that.trouble);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int task,  LoadStatus status,  AgentReview? review,  Set<int> ticked,  Set<int> keep,  bool applying,  String? applied,  String? undone,  String? trouble)  $default,) {final _that = this;
switch (_that) {
case _AgentReviewState():
return $default(_that.task,_that.status,_that.review,_that.ticked,_that.keep,_that.applying,_that.applied,_that.undone,_that.trouble);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int task,  LoadStatus status,  AgentReview? review,  Set<int> ticked,  Set<int> keep,  bool applying,  String? applied,  String? undone,  String? trouble)?  $default,) {final _that = this;
switch (_that) {
case _AgentReviewState() when $default != null:
return $default(_that.task,_that.status,_that.review,_that.ticked,_that.keep,_that.applying,_that.applied,_that.undone,_that.trouble);case _:
  return null;

}
}

}

/// @nodoc


class _AgentReviewState implements AgentReviewState {
  const _AgentReviewState({this.task = 0, this.status = LoadStatus.loading, this.review, final  Set<int> ticked = const <int>{}, final  Set<int> keep = const <int>{}, this.applying = false, this.applied, this.undone, this.trouble}): _ticked = ticked,_keep = keep;
  

@override@JsonKey() final  int task;
@override@JsonKey() final  LoadStatus status;
@override final  AgentReview? review;
 final  Set<int> _ticked;
@override@JsonKey() Set<int> get ticked {
  if (_ticked is EqualUnmodifiableSetView) return _ticked;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_ticked);
}

 final  Set<int> _keep;
@override@JsonKey() Set<int> get keep {
  if (_keep is EqualUnmodifiableSetView) return _keep;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_keep);
}

@override@JsonKey() final  bool applying;
/// The sentence to show once applied ("Applied 3 changes…"); Undo is
/// offered while it is set.
@override final  String? applied;
/// The sentence to show once undone.
@override final  String? undone;
@override final  String? trouble;

/// Create a copy of AgentReviewState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AgentReviewStateCopyWith<_AgentReviewState> get copyWith => __$AgentReviewStateCopyWithImpl<_AgentReviewState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AgentReviewState&&(identical(other.task, task) || other.task == task)&&(identical(other.status, status) || other.status == status)&&(identical(other.review, review) || other.review == review)&&const DeepCollectionEquality().equals(other._ticked, _ticked)&&const DeepCollectionEquality().equals(other._keep, _keep)&&(identical(other.applying, applying) || other.applying == applying)&&(identical(other.applied, applied) || other.applied == applied)&&(identical(other.undone, undone) || other.undone == undone)&&(identical(other.trouble, trouble) || other.trouble == trouble));
}


@override
int get hashCode => Object.hash(runtimeType,task,status,review,const DeepCollectionEquality().hash(_ticked),const DeepCollectionEquality().hash(_keep),applying,applied,undone,trouble);

@override
String toString() {
  return 'AgentReviewState(task: $task, status: $status, review: $review, ticked: $ticked, keep: $keep, applying: $applying, applied: $applied, undone: $undone, trouble: $trouble)';
}


}

/// @nodoc
abstract mixin class _$AgentReviewStateCopyWith<$Res> implements $AgentReviewStateCopyWith<$Res> {
  factory _$AgentReviewStateCopyWith(_AgentReviewState value, $Res Function(_AgentReviewState) _then) = __$AgentReviewStateCopyWithImpl;
@override @useResult
$Res call({
 int task, LoadStatus status, AgentReview? review, Set<int> ticked, Set<int> keep, bool applying, String? applied, String? undone, String? trouble
});


@override $AgentReviewCopyWith<$Res>? get review;

}
/// @nodoc
class __$AgentReviewStateCopyWithImpl<$Res>
    implements _$AgentReviewStateCopyWith<$Res> {
  __$AgentReviewStateCopyWithImpl(this._self, this._then);

  final _AgentReviewState _self;
  final $Res Function(_AgentReviewState) _then;

/// Create a copy of AgentReviewState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? task = null,Object? status = null,Object? review = freezed,Object? ticked = null,Object? keep = null,Object? applying = null,Object? applied = freezed,Object? undone = freezed,Object? trouble = freezed,}) {
  return _then(_AgentReviewState(
task: null == task ? _self.task : task // ignore: cast_nullable_to_non_nullable
as int,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,review: freezed == review ? _self.review : review // ignore: cast_nullable_to_non_nullable
as AgentReview?,ticked: null == ticked ? _self._ticked : ticked // ignore: cast_nullable_to_non_nullable
as Set<int>,keep: null == keep ? _self._keep : keep // ignore: cast_nullable_to_non_nullable
as Set<int>,applying: null == applying ? _self.applying : applying // ignore: cast_nullable_to_non_nullable
as bool,applied: freezed == applied ? _self.applied : applied // ignore: cast_nullable_to_non_nullable
as String?,undone: freezed == undone ? _self.undone : undone // ignore: cast_nullable_to_non_nullable
as String?,trouble: freezed == trouble ? _self.trouble : trouble // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of AgentReviewState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AgentReviewCopyWith<$Res>? get review {
    if (_self.review == null) {
    return null;
  }

  return $AgentReviewCopyWith<$Res>(_self.review!, (value) {
    return _then(_self.copyWith(review: value));
  });
}
}

// dart format on
