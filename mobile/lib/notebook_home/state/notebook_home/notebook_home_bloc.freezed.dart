// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'notebook_home_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NotebookHomeEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotebookHomeEvent()';
}


}

/// @nodoc
class $NotebookHomeEventCopyWith<$Res>  {
$NotebookHomeEventCopyWith(NotebookHomeEvent _, $Res Function(NotebookHomeEvent) __);
}


/// Adds pattern-matching-related methods to [NotebookHomeEvent].
extension NotebookHomeEventPatterns on NotebookHomeEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NotebookHomeStartedEvent value)?  started,TResult Function( NotebookHomeReopenedEvent value)?  reopened,TResult Function( NotebookHomeMapRequestedEvent value)?  mapRequested,TResult Function( NotebookHomeRefreshAllRequestedEvent value)?  refreshAllRequested,TResult Function( NotebookHomeSectionRefreshRequestedEvent value)?  sectionRefreshRequested,TResult Function( NotebookHomePinnedEvent value)?  pinned,TResult Function( NotebookHomeHiddenEvent value)?  hidden,TResult Function( NotebookHomeHiddenShownEvent value)?  hiddenShown,TResult Function( NotebookHomeNotRightEvent value)?  notRight,TResult Function( NotebookHomeCopiedIntoHomeEvent value)?  copiedIntoHome,TResult Function( NotebookHomeTaskTickedEvent value)?  taskTicked,TResult Function( NotebookHomeAllShownEvent value)?  allShown,TResult Function( NotebookHomeCustomisedEvent value)?  customised,TResult Function( NotebookHomeLayoutResetEvent value)?  layoutReset,TResult Function( NotebookHomeHomeChosenEvent value)?  homeChosen,TResult Function( NotebookHomeNewHomeRequestedEvent value)?  newHomeRequested,TResult Function( NotebookHomeNoticeTakenEvent value)?  noticeTaken,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NotebookHomeStartedEvent() when started != null:
return started(_that);case NotebookHomeReopenedEvent() when reopened != null:
return reopened(_that);case NotebookHomeMapRequestedEvent() when mapRequested != null:
return mapRequested(_that);case NotebookHomeRefreshAllRequestedEvent() when refreshAllRequested != null:
return refreshAllRequested(_that);case NotebookHomeSectionRefreshRequestedEvent() when sectionRefreshRequested != null:
return sectionRefreshRequested(_that);case NotebookHomePinnedEvent() when pinned != null:
return pinned(_that);case NotebookHomeHiddenEvent() when hidden != null:
return hidden(_that);case NotebookHomeHiddenShownEvent() when hiddenShown != null:
return hiddenShown(_that);case NotebookHomeNotRightEvent() when notRight != null:
return notRight(_that);case NotebookHomeCopiedIntoHomeEvent() when copiedIntoHome != null:
return copiedIntoHome(_that);case NotebookHomeTaskTickedEvent() when taskTicked != null:
return taskTicked(_that);case NotebookHomeAllShownEvent() when allShown != null:
return allShown(_that);case NotebookHomeCustomisedEvent() when customised != null:
return customised(_that);case NotebookHomeLayoutResetEvent() when layoutReset != null:
return layoutReset(_that);case NotebookHomeHomeChosenEvent() when homeChosen != null:
return homeChosen(_that);case NotebookHomeNewHomeRequestedEvent() when newHomeRequested != null:
return newHomeRequested(_that);case NotebookHomeNoticeTakenEvent() when noticeTaken != null:
return noticeTaken(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NotebookHomeStartedEvent value)  started,required TResult Function( NotebookHomeReopenedEvent value)  reopened,required TResult Function( NotebookHomeMapRequestedEvent value)  mapRequested,required TResult Function( NotebookHomeRefreshAllRequestedEvent value)  refreshAllRequested,required TResult Function( NotebookHomeSectionRefreshRequestedEvent value)  sectionRefreshRequested,required TResult Function( NotebookHomePinnedEvent value)  pinned,required TResult Function( NotebookHomeHiddenEvent value)  hidden,required TResult Function( NotebookHomeHiddenShownEvent value)  hiddenShown,required TResult Function( NotebookHomeNotRightEvent value)  notRight,required TResult Function( NotebookHomeCopiedIntoHomeEvent value)  copiedIntoHome,required TResult Function( NotebookHomeTaskTickedEvent value)  taskTicked,required TResult Function( NotebookHomeAllShownEvent value)  allShown,required TResult Function( NotebookHomeCustomisedEvent value)  customised,required TResult Function( NotebookHomeLayoutResetEvent value)  layoutReset,required TResult Function( NotebookHomeHomeChosenEvent value)  homeChosen,required TResult Function( NotebookHomeNewHomeRequestedEvent value)  newHomeRequested,required TResult Function( NotebookHomeNoticeTakenEvent value)  noticeTaken,}){
final _that = this;
switch (_that) {
case NotebookHomeStartedEvent():
return started(_that);case NotebookHomeReopenedEvent():
return reopened(_that);case NotebookHomeMapRequestedEvent():
return mapRequested(_that);case NotebookHomeRefreshAllRequestedEvent():
return refreshAllRequested(_that);case NotebookHomeSectionRefreshRequestedEvent():
return sectionRefreshRequested(_that);case NotebookHomePinnedEvent():
return pinned(_that);case NotebookHomeHiddenEvent():
return hidden(_that);case NotebookHomeHiddenShownEvent():
return hiddenShown(_that);case NotebookHomeNotRightEvent():
return notRight(_that);case NotebookHomeCopiedIntoHomeEvent():
return copiedIntoHome(_that);case NotebookHomeTaskTickedEvent():
return taskTicked(_that);case NotebookHomeAllShownEvent():
return allShown(_that);case NotebookHomeCustomisedEvent():
return customised(_that);case NotebookHomeLayoutResetEvent():
return layoutReset(_that);case NotebookHomeHomeChosenEvent():
return homeChosen(_that);case NotebookHomeNewHomeRequestedEvent():
return newHomeRequested(_that);case NotebookHomeNoticeTakenEvent():
return noticeTaken(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NotebookHomeStartedEvent value)?  started,TResult? Function( NotebookHomeReopenedEvent value)?  reopened,TResult? Function( NotebookHomeMapRequestedEvent value)?  mapRequested,TResult? Function( NotebookHomeRefreshAllRequestedEvent value)?  refreshAllRequested,TResult? Function( NotebookHomeSectionRefreshRequestedEvent value)?  sectionRefreshRequested,TResult? Function( NotebookHomePinnedEvent value)?  pinned,TResult? Function( NotebookHomeHiddenEvent value)?  hidden,TResult? Function( NotebookHomeHiddenShownEvent value)?  hiddenShown,TResult? Function( NotebookHomeNotRightEvent value)?  notRight,TResult? Function( NotebookHomeCopiedIntoHomeEvent value)?  copiedIntoHome,TResult? Function( NotebookHomeTaskTickedEvent value)?  taskTicked,TResult? Function( NotebookHomeAllShownEvent value)?  allShown,TResult? Function( NotebookHomeCustomisedEvent value)?  customised,TResult? Function( NotebookHomeLayoutResetEvent value)?  layoutReset,TResult? Function( NotebookHomeHomeChosenEvent value)?  homeChosen,TResult? Function( NotebookHomeNewHomeRequestedEvent value)?  newHomeRequested,TResult? Function( NotebookHomeNoticeTakenEvent value)?  noticeTaken,}){
final _that = this;
switch (_that) {
case NotebookHomeStartedEvent() when started != null:
return started(_that);case NotebookHomeReopenedEvent() when reopened != null:
return reopened(_that);case NotebookHomeMapRequestedEvent() when mapRequested != null:
return mapRequested(_that);case NotebookHomeRefreshAllRequestedEvent() when refreshAllRequested != null:
return refreshAllRequested(_that);case NotebookHomeSectionRefreshRequestedEvent() when sectionRefreshRequested != null:
return sectionRefreshRequested(_that);case NotebookHomePinnedEvent() when pinned != null:
return pinned(_that);case NotebookHomeHiddenEvent() when hidden != null:
return hidden(_that);case NotebookHomeHiddenShownEvent() when hiddenShown != null:
return hiddenShown(_that);case NotebookHomeNotRightEvent() when notRight != null:
return notRight(_that);case NotebookHomeCopiedIntoHomeEvent() when copiedIntoHome != null:
return copiedIntoHome(_that);case NotebookHomeTaskTickedEvent() when taskTicked != null:
return taskTicked(_that);case NotebookHomeAllShownEvent() when allShown != null:
return allShown(_that);case NotebookHomeCustomisedEvent() when customised != null:
return customised(_that);case NotebookHomeLayoutResetEvent() when layoutReset != null:
return layoutReset(_that);case NotebookHomeHomeChosenEvent() when homeChosen != null:
return homeChosen(_that);case NotebookHomeNewHomeRequestedEvent() when newHomeRequested != null:
return newHomeRequested(_that);case NotebookHomeNoticeTakenEvent() when noticeTaken != null:
return noticeTaken(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String bookId)?  started,TResult Function()?  reopened,TResult Function()?  mapRequested,TResult Function()?  refreshAllRequested,TResult Function( String key)?  sectionRefreshRequested,TResult Function( String key,  bool on)?  pinned,TResult Function( String key,  bool hidden)?  hidden,TResult Function()?  hiddenShown,TResult Function( String key,  int index)?  notRight,TResult Function( String key)?  copiedIntoHome,TResult Function( PageTask task,  bool checked)?  taskTicked,TResult Function( String key)?  allShown,TResult Function( List<String> order,  List<String> hidden)?  customised,TResult Function()?  layoutReset,TResult Function( String noteId)?  homeChosen,TResult Function()?  newHomeRequested,TResult Function()?  noticeTaken,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NotebookHomeStartedEvent() when started != null:
return started(_that.bookId);case NotebookHomeReopenedEvent() when reopened != null:
return reopened();case NotebookHomeMapRequestedEvent() when mapRequested != null:
return mapRequested();case NotebookHomeRefreshAllRequestedEvent() when refreshAllRequested != null:
return refreshAllRequested();case NotebookHomeSectionRefreshRequestedEvent() when sectionRefreshRequested != null:
return sectionRefreshRequested(_that.key);case NotebookHomePinnedEvent() when pinned != null:
return pinned(_that.key,_that.on);case NotebookHomeHiddenEvent() when hidden != null:
return hidden(_that.key,_that.hidden);case NotebookHomeHiddenShownEvent() when hiddenShown != null:
return hiddenShown();case NotebookHomeNotRightEvent() when notRight != null:
return notRight(_that.key,_that.index);case NotebookHomeCopiedIntoHomeEvent() when copiedIntoHome != null:
return copiedIntoHome(_that.key);case NotebookHomeTaskTickedEvent() when taskTicked != null:
return taskTicked(_that.task,_that.checked);case NotebookHomeAllShownEvent() when allShown != null:
return allShown(_that.key);case NotebookHomeCustomisedEvent() when customised != null:
return customised(_that.order,_that.hidden);case NotebookHomeLayoutResetEvent() when layoutReset != null:
return layoutReset();case NotebookHomeHomeChosenEvent() when homeChosen != null:
return homeChosen(_that.noteId);case NotebookHomeNewHomeRequestedEvent() when newHomeRequested != null:
return newHomeRequested();case NotebookHomeNoticeTakenEvent() when noticeTaken != null:
return noticeTaken();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String bookId)  started,required TResult Function()  reopened,required TResult Function()  mapRequested,required TResult Function()  refreshAllRequested,required TResult Function( String key)  sectionRefreshRequested,required TResult Function( String key,  bool on)  pinned,required TResult Function( String key,  bool hidden)  hidden,required TResult Function()  hiddenShown,required TResult Function( String key,  int index)  notRight,required TResult Function( String key)  copiedIntoHome,required TResult Function( PageTask task,  bool checked)  taskTicked,required TResult Function( String key)  allShown,required TResult Function( List<String> order,  List<String> hidden)  customised,required TResult Function()  layoutReset,required TResult Function( String noteId)  homeChosen,required TResult Function()  newHomeRequested,required TResult Function()  noticeTaken,}) {final _that = this;
switch (_that) {
case NotebookHomeStartedEvent():
return started(_that.bookId);case NotebookHomeReopenedEvent():
return reopened();case NotebookHomeMapRequestedEvent():
return mapRequested();case NotebookHomeRefreshAllRequestedEvent():
return refreshAllRequested();case NotebookHomeSectionRefreshRequestedEvent():
return sectionRefreshRequested(_that.key);case NotebookHomePinnedEvent():
return pinned(_that.key,_that.on);case NotebookHomeHiddenEvent():
return hidden(_that.key,_that.hidden);case NotebookHomeHiddenShownEvent():
return hiddenShown();case NotebookHomeNotRightEvent():
return notRight(_that.key,_that.index);case NotebookHomeCopiedIntoHomeEvent():
return copiedIntoHome(_that.key);case NotebookHomeTaskTickedEvent():
return taskTicked(_that.task,_that.checked);case NotebookHomeAllShownEvent():
return allShown(_that.key);case NotebookHomeCustomisedEvent():
return customised(_that.order,_that.hidden);case NotebookHomeLayoutResetEvent():
return layoutReset();case NotebookHomeHomeChosenEvent():
return homeChosen(_that.noteId);case NotebookHomeNewHomeRequestedEvent():
return newHomeRequested();case NotebookHomeNoticeTakenEvent():
return noticeTaken();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String bookId)?  started,TResult? Function()?  reopened,TResult? Function()?  mapRequested,TResult? Function()?  refreshAllRequested,TResult? Function( String key)?  sectionRefreshRequested,TResult? Function( String key,  bool on)?  pinned,TResult? Function( String key,  bool hidden)?  hidden,TResult? Function()?  hiddenShown,TResult? Function( String key,  int index)?  notRight,TResult? Function( String key)?  copiedIntoHome,TResult? Function( PageTask task,  bool checked)?  taskTicked,TResult? Function( String key)?  allShown,TResult? Function( List<String> order,  List<String> hidden)?  customised,TResult? Function()?  layoutReset,TResult? Function( String noteId)?  homeChosen,TResult? Function()?  newHomeRequested,TResult? Function()?  noticeTaken,}) {final _that = this;
switch (_that) {
case NotebookHomeStartedEvent() when started != null:
return started(_that.bookId);case NotebookHomeReopenedEvent() when reopened != null:
return reopened();case NotebookHomeMapRequestedEvent() when mapRequested != null:
return mapRequested();case NotebookHomeRefreshAllRequestedEvent() when refreshAllRequested != null:
return refreshAllRequested();case NotebookHomeSectionRefreshRequestedEvent() when sectionRefreshRequested != null:
return sectionRefreshRequested(_that.key);case NotebookHomePinnedEvent() when pinned != null:
return pinned(_that.key,_that.on);case NotebookHomeHiddenEvent() when hidden != null:
return hidden(_that.key,_that.hidden);case NotebookHomeHiddenShownEvent() when hiddenShown != null:
return hiddenShown();case NotebookHomeNotRightEvent() when notRight != null:
return notRight(_that.key,_that.index);case NotebookHomeCopiedIntoHomeEvent() when copiedIntoHome != null:
return copiedIntoHome(_that.key);case NotebookHomeTaskTickedEvent() when taskTicked != null:
return taskTicked(_that.task,_that.checked);case NotebookHomeAllShownEvent() when allShown != null:
return allShown(_that.key);case NotebookHomeCustomisedEvent() when customised != null:
return customised(_that.order,_that.hidden);case NotebookHomeLayoutResetEvent() when layoutReset != null:
return layoutReset();case NotebookHomeHomeChosenEvent() when homeChosen != null:
return homeChosen(_that.noteId);case NotebookHomeNewHomeRequestedEvent() when newHomeRequested != null:
return newHomeRequested();case NotebookHomeNoticeTakenEvent() when noticeTaken != null:
return noticeTaken();case _:
  return null;

}
}

}

/// @nodoc


class NotebookHomeStartedEvent implements NotebookHomeEvent {
  const NotebookHomeStartedEvent(this.bookId);
  

 final  String bookId;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomeStartedEventCopyWith<NotebookHomeStartedEvent> get copyWith => _$NotebookHomeStartedEventCopyWithImpl<NotebookHomeStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeStartedEvent&&(identical(other.bookId, bookId) || other.bookId == bookId));
}


@override
int get hashCode => Object.hash(runtimeType,bookId);

@override
String toString() {
  return 'NotebookHomeEvent.started(bookId: $bookId)';
}


}

/// @nodoc
abstract mixin class $NotebookHomeStartedEventCopyWith<$Res> implements $NotebookHomeEventCopyWith<$Res> {
  factory $NotebookHomeStartedEventCopyWith(NotebookHomeStartedEvent value, $Res Function(NotebookHomeStartedEvent) _then) = _$NotebookHomeStartedEventCopyWithImpl;
@useResult
$Res call({
 String bookId
});




}
/// @nodoc
class _$NotebookHomeStartedEventCopyWithImpl<$Res>
    implements $NotebookHomeStartedEventCopyWith<$Res> {
  _$NotebookHomeStartedEventCopyWithImpl(this._self, this._then);

  final NotebookHomeStartedEvent _self;
  final $Res Function(NotebookHomeStartedEvent) _then;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? bookId = null,}) {
  return _then(NotebookHomeStartedEvent(
null == bookId ? _self.bookId : bookId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NotebookHomeReopenedEvent implements NotebookHomeEvent {
  const NotebookHomeReopenedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeReopenedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotebookHomeEvent.reopened()';
}


}




/// @nodoc


class NotebookHomeMapRequestedEvent implements NotebookHomeEvent {
  const NotebookHomeMapRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeMapRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotebookHomeEvent.mapRequested()';
}


}




/// @nodoc


class NotebookHomeRefreshAllRequestedEvent implements NotebookHomeEvent {
  const NotebookHomeRefreshAllRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeRefreshAllRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotebookHomeEvent.refreshAllRequested()';
}


}




/// @nodoc


class NotebookHomeSectionRefreshRequestedEvent implements NotebookHomeEvent {
  const NotebookHomeSectionRefreshRequestedEvent(this.key);
  

 final  String key;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomeSectionRefreshRequestedEventCopyWith<NotebookHomeSectionRefreshRequestedEvent> get copyWith => _$NotebookHomeSectionRefreshRequestedEventCopyWithImpl<NotebookHomeSectionRefreshRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeSectionRefreshRequestedEvent&&(identical(other.key, key) || other.key == key));
}


@override
int get hashCode => Object.hash(runtimeType,key);

@override
String toString() {
  return 'NotebookHomeEvent.sectionRefreshRequested(key: $key)';
}


}

/// @nodoc
abstract mixin class $NotebookHomeSectionRefreshRequestedEventCopyWith<$Res> implements $NotebookHomeEventCopyWith<$Res> {
  factory $NotebookHomeSectionRefreshRequestedEventCopyWith(NotebookHomeSectionRefreshRequestedEvent value, $Res Function(NotebookHomeSectionRefreshRequestedEvent) _then) = _$NotebookHomeSectionRefreshRequestedEventCopyWithImpl;
@useResult
$Res call({
 String key
});




}
/// @nodoc
class _$NotebookHomeSectionRefreshRequestedEventCopyWithImpl<$Res>
    implements $NotebookHomeSectionRefreshRequestedEventCopyWith<$Res> {
  _$NotebookHomeSectionRefreshRequestedEventCopyWithImpl(this._self, this._then);

  final NotebookHomeSectionRefreshRequestedEvent _self;
  final $Res Function(NotebookHomeSectionRefreshRequestedEvent) _then;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = null,}) {
  return _then(NotebookHomeSectionRefreshRequestedEvent(
null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NotebookHomePinnedEvent implements NotebookHomeEvent {
  const NotebookHomePinnedEvent(this.key, {required this.on});
  

 final  String key;
 final  bool on;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomePinnedEventCopyWith<NotebookHomePinnedEvent> get copyWith => _$NotebookHomePinnedEventCopyWithImpl<NotebookHomePinnedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomePinnedEvent&&(identical(other.key, key) || other.key == key)&&(identical(other.on, on) || other.on == on));
}


@override
int get hashCode => Object.hash(runtimeType,key,on);

@override
String toString() {
  return 'NotebookHomeEvent.pinned(key: $key, on: $on)';
}


}

/// @nodoc
abstract mixin class $NotebookHomePinnedEventCopyWith<$Res> implements $NotebookHomeEventCopyWith<$Res> {
  factory $NotebookHomePinnedEventCopyWith(NotebookHomePinnedEvent value, $Res Function(NotebookHomePinnedEvent) _then) = _$NotebookHomePinnedEventCopyWithImpl;
@useResult
$Res call({
 String key, bool on
});




}
/// @nodoc
class _$NotebookHomePinnedEventCopyWithImpl<$Res>
    implements $NotebookHomePinnedEventCopyWith<$Res> {
  _$NotebookHomePinnedEventCopyWithImpl(this._self, this._then);

  final NotebookHomePinnedEvent _self;
  final $Res Function(NotebookHomePinnedEvent) _then;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = null,Object? on = null,}) {
  return _then(NotebookHomePinnedEvent(
null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,on: null == on ? _self.on : on // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class NotebookHomeHiddenEvent implements NotebookHomeEvent {
  const NotebookHomeHiddenEvent(this.key, {required this.hidden});
  

 final  String key;
 final  bool hidden;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomeHiddenEventCopyWith<NotebookHomeHiddenEvent> get copyWith => _$NotebookHomeHiddenEventCopyWithImpl<NotebookHomeHiddenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeHiddenEvent&&(identical(other.key, key) || other.key == key)&&(identical(other.hidden, hidden) || other.hidden == hidden));
}


@override
int get hashCode => Object.hash(runtimeType,key,hidden);

@override
String toString() {
  return 'NotebookHomeEvent.hidden(key: $key, hidden: $hidden)';
}


}

/// @nodoc
abstract mixin class $NotebookHomeHiddenEventCopyWith<$Res> implements $NotebookHomeEventCopyWith<$Res> {
  factory $NotebookHomeHiddenEventCopyWith(NotebookHomeHiddenEvent value, $Res Function(NotebookHomeHiddenEvent) _then) = _$NotebookHomeHiddenEventCopyWithImpl;
@useResult
$Res call({
 String key, bool hidden
});




}
/// @nodoc
class _$NotebookHomeHiddenEventCopyWithImpl<$Res>
    implements $NotebookHomeHiddenEventCopyWith<$Res> {
  _$NotebookHomeHiddenEventCopyWithImpl(this._self, this._then);

  final NotebookHomeHiddenEvent _self;
  final $Res Function(NotebookHomeHiddenEvent) _then;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = null,Object? hidden = null,}) {
  return _then(NotebookHomeHiddenEvent(
null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,hidden: null == hidden ? _self.hidden : hidden // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class NotebookHomeHiddenShownEvent implements NotebookHomeEvent {
  const NotebookHomeHiddenShownEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeHiddenShownEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotebookHomeEvent.hiddenShown()';
}


}




/// @nodoc


class NotebookHomeNotRightEvent implements NotebookHomeEvent {
  const NotebookHomeNotRightEvent(this.key, this.index);
  

 final  String key;
 final  int index;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomeNotRightEventCopyWith<NotebookHomeNotRightEvent> get copyWith => _$NotebookHomeNotRightEventCopyWithImpl<NotebookHomeNotRightEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeNotRightEvent&&(identical(other.key, key) || other.key == key)&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,key,index);

@override
String toString() {
  return 'NotebookHomeEvent.notRight(key: $key, index: $index)';
}


}

/// @nodoc
abstract mixin class $NotebookHomeNotRightEventCopyWith<$Res> implements $NotebookHomeEventCopyWith<$Res> {
  factory $NotebookHomeNotRightEventCopyWith(NotebookHomeNotRightEvent value, $Res Function(NotebookHomeNotRightEvent) _then) = _$NotebookHomeNotRightEventCopyWithImpl;
@useResult
$Res call({
 String key, int index
});




}
/// @nodoc
class _$NotebookHomeNotRightEventCopyWithImpl<$Res>
    implements $NotebookHomeNotRightEventCopyWith<$Res> {
  _$NotebookHomeNotRightEventCopyWithImpl(this._self, this._then);

  final NotebookHomeNotRightEvent _self;
  final $Res Function(NotebookHomeNotRightEvent) _then;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = null,Object? index = null,}) {
  return _then(NotebookHomeNotRightEvent(
null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class NotebookHomeCopiedIntoHomeEvent implements NotebookHomeEvent {
  const NotebookHomeCopiedIntoHomeEvent(this.key);
  

 final  String key;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomeCopiedIntoHomeEventCopyWith<NotebookHomeCopiedIntoHomeEvent> get copyWith => _$NotebookHomeCopiedIntoHomeEventCopyWithImpl<NotebookHomeCopiedIntoHomeEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeCopiedIntoHomeEvent&&(identical(other.key, key) || other.key == key));
}


@override
int get hashCode => Object.hash(runtimeType,key);

@override
String toString() {
  return 'NotebookHomeEvent.copiedIntoHome(key: $key)';
}


}

/// @nodoc
abstract mixin class $NotebookHomeCopiedIntoHomeEventCopyWith<$Res> implements $NotebookHomeEventCopyWith<$Res> {
  factory $NotebookHomeCopiedIntoHomeEventCopyWith(NotebookHomeCopiedIntoHomeEvent value, $Res Function(NotebookHomeCopiedIntoHomeEvent) _then) = _$NotebookHomeCopiedIntoHomeEventCopyWithImpl;
@useResult
$Res call({
 String key
});




}
/// @nodoc
class _$NotebookHomeCopiedIntoHomeEventCopyWithImpl<$Res>
    implements $NotebookHomeCopiedIntoHomeEventCopyWith<$Res> {
  _$NotebookHomeCopiedIntoHomeEventCopyWithImpl(this._self, this._then);

  final NotebookHomeCopiedIntoHomeEvent _self;
  final $Res Function(NotebookHomeCopiedIntoHomeEvent) _then;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = null,}) {
  return _then(NotebookHomeCopiedIntoHomeEvent(
null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NotebookHomeTaskTickedEvent implements NotebookHomeEvent {
  const NotebookHomeTaskTickedEvent(this.task, {required this.checked});
  

 final  PageTask task;
 final  bool checked;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomeTaskTickedEventCopyWith<NotebookHomeTaskTickedEvent> get copyWith => _$NotebookHomeTaskTickedEventCopyWithImpl<NotebookHomeTaskTickedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeTaskTickedEvent&&(identical(other.task, task) || other.task == task)&&(identical(other.checked, checked) || other.checked == checked));
}


@override
int get hashCode => Object.hash(runtimeType,task,checked);

@override
String toString() {
  return 'NotebookHomeEvent.taskTicked(task: $task, checked: $checked)';
}


}

/// @nodoc
abstract mixin class $NotebookHomeTaskTickedEventCopyWith<$Res> implements $NotebookHomeEventCopyWith<$Res> {
  factory $NotebookHomeTaskTickedEventCopyWith(NotebookHomeTaskTickedEvent value, $Res Function(NotebookHomeTaskTickedEvent) _then) = _$NotebookHomeTaskTickedEventCopyWithImpl;
@useResult
$Res call({
 PageTask task, bool checked
});




}
/// @nodoc
class _$NotebookHomeTaskTickedEventCopyWithImpl<$Res>
    implements $NotebookHomeTaskTickedEventCopyWith<$Res> {
  _$NotebookHomeTaskTickedEventCopyWithImpl(this._self, this._then);

  final NotebookHomeTaskTickedEvent _self;
  final $Res Function(NotebookHomeTaskTickedEvent) _then;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? task = null,Object? checked = null,}) {
  return _then(NotebookHomeTaskTickedEvent(
null == task ? _self.task : task // ignore: cast_nullable_to_non_nullable
as PageTask,checked: null == checked ? _self.checked : checked // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class NotebookHomeAllShownEvent implements NotebookHomeEvent {
  const NotebookHomeAllShownEvent(this.key);
  

 final  String key;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomeAllShownEventCopyWith<NotebookHomeAllShownEvent> get copyWith => _$NotebookHomeAllShownEventCopyWithImpl<NotebookHomeAllShownEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeAllShownEvent&&(identical(other.key, key) || other.key == key));
}


@override
int get hashCode => Object.hash(runtimeType,key);

@override
String toString() {
  return 'NotebookHomeEvent.allShown(key: $key)';
}


}

/// @nodoc
abstract mixin class $NotebookHomeAllShownEventCopyWith<$Res> implements $NotebookHomeEventCopyWith<$Res> {
  factory $NotebookHomeAllShownEventCopyWith(NotebookHomeAllShownEvent value, $Res Function(NotebookHomeAllShownEvent) _then) = _$NotebookHomeAllShownEventCopyWithImpl;
@useResult
$Res call({
 String key
});




}
/// @nodoc
class _$NotebookHomeAllShownEventCopyWithImpl<$Res>
    implements $NotebookHomeAllShownEventCopyWith<$Res> {
  _$NotebookHomeAllShownEventCopyWithImpl(this._self, this._then);

  final NotebookHomeAllShownEvent _self;
  final $Res Function(NotebookHomeAllShownEvent) _then;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = null,}) {
  return _then(NotebookHomeAllShownEvent(
null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NotebookHomeCustomisedEvent implements NotebookHomeEvent {
  const NotebookHomeCustomisedEvent(final  List<String> order, final  List<String> hidden): _order = order,_hidden = hidden;
  

 final  List<String> _order;
 List<String> get order {
  if (_order is EqualUnmodifiableListView) return _order;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_order);
}

 final  List<String> _hidden;
 List<String> get hidden {
  if (_hidden is EqualUnmodifiableListView) return _hidden;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_hidden);
}


/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomeCustomisedEventCopyWith<NotebookHomeCustomisedEvent> get copyWith => _$NotebookHomeCustomisedEventCopyWithImpl<NotebookHomeCustomisedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeCustomisedEvent&&const DeepCollectionEquality().equals(other._order, _order)&&const DeepCollectionEquality().equals(other._hidden, _hidden));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_order),const DeepCollectionEquality().hash(_hidden));

@override
String toString() {
  return 'NotebookHomeEvent.customised(order: $order, hidden: $hidden)';
}


}

/// @nodoc
abstract mixin class $NotebookHomeCustomisedEventCopyWith<$Res> implements $NotebookHomeEventCopyWith<$Res> {
  factory $NotebookHomeCustomisedEventCopyWith(NotebookHomeCustomisedEvent value, $Res Function(NotebookHomeCustomisedEvent) _then) = _$NotebookHomeCustomisedEventCopyWithImpl;
@useResult
$Res call({
 List<String> order, List<String> hidden
});




}
/// @nodoc
class _$NotebookHomeCustomisedEventCopyWithImpl<$Res>
    implements $NotebookHomeCustomisedEventCopyWith<$Res> {
  _$NotebookHomeCustomisedEventCopyWithImpl(this._self, this._then);

  final NotebookHomeCustomisedEvent _self;
  final $Res Function(NotebookHomeCustomisedEvent) _then;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? order = null,Object? hidden = null,}) {
  return _then(NotebookHomeCustomisedEvent(
null == order ? _self._order : order // ignore: cast_nullable_to_non_nullable
as List<String>,null == hidden ? _self._hidden : hidden // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class NotebookHomeLayoutResetEvent implements NotebookHomeEvent {
  const NotebookHomeLayoutResetEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeLayoutResetEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotebookHomeEvent.layoutReset()';
}


}




/// @nodoc


class NotebookHomeHomeChosenEvent implements NotebookHomeEvent {
  const NotebookHomeHomeChosenEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomeHomeChosenEventCopyWith<NotebookHomeHomeChosenEvent> get copyWith => _$NotebookHomeHomeChosenEventCopyWithImpl<NotebookHomeHomeChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeHomeChosenEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'NotebookHomeEvent.homeChosen(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $NotebookHomeHomeChosenEventCopyWith<$Res> implements $NotebookHomeEventCopyWith<$Res> {
  factory $NotebookHomeHomeChosenEventCopyWith(NotebookHomeHomeChosenEvent value, $Res Function(NotebookHomeHomeChosenEvent) _then) = _$NotebookHomeHomeChosenEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$NotebookHomeHomeChosenEventCopyWithImpl<$Res>
    implements $NotebookHomeHomeChosenEventCopyWith<$Res> {
  _$NotebookHomeHomeChosenEventCopyWithImpl(this._self, this._then);

  final NotebookHomeHomeChosenEvent _self;
  final $Res Function(NotebookHomeHomeChosenEvent) _then;

/// Create a copy of NotebookHomeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(NotebookHomeHomeChosenEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NotebookHomeNewHomeRequestedEvent implements NotebookHomeEvent {
  const NotebookHomeNewHomeRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeNewHomeRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotebookHomeEvent.newHomeRequested()';
}


}




/// @nodoc


class NotebookHomeNoticeTakenEvent implements NotebookHomeEvent {
  const NotebookHomeNoticeTakenEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeNoticeTakenEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NotebookHomeEvent.noticeTaken()';
}


}




/// @nodoc
mixin _$NotebookHomeState {

 String get bookId; LoadStatus get status;/// The core's own words; null until read.
 HomeWords? get words; HomePanel? get panel;/// The map, read after the sections are shown; null until then.
 HomeMap? get map;/// Section keys being rewritten now.
 Set<String> get working;/// Section keys that failed to rewrite, with the model's own words.
 Map<String, String> get trouble;/// A section was copied into the Home note: the page says so once.
 bool get copied;/// A new Home note was made: the page opens it once.
 String? get madeNoteId;
/// Create a copy of NotebookHomeState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotebookHomeStateCopyWith<NotebookHomeState> get copyWith => _$NotebookHomeStateCopyWithImpl<NotebookHomeState>(this as NotebookHomeState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotebookHomeState&&(identical(other.bookId, bookId) || other.bookId == bookId)&&(identical(other.status, status) || other.status == status)&&(identical(other.words, words) || other.words == words)&&(identical(other.panel, panel) || other.panel == panel)&&(identical(other.map, map) || other.map == map)&&const DeepCollectionEquality().equals(other.working, working)&&const DeepCollectionEquality().equals(other.trouble, trouble)&&(identical(other.copied, copied) || other.copied == copied)&&(identical(other.madeNoteId, madeNoteId) || other.madeNoteId == madeNoteId));
}


@override
int get hashCode => Object.hash(runtimeType,bookId,status,words,panel,map,const DeepCollectionEquality().hash(working),const DeepCollectionEquality().hash(trouble),copied,madeNoteId);

@override
String toString() {
  return 'NotebookHomeState(bookId: $bookId, status: $status, words: $words, panel: $panel, map: $map, working: $working, trouble: $trouble, copied: $copied, madeNoteId: $madeNoteId)';
}


}

/// @nodoc
abstract mixin class $NotebookHomeStateCopyWith<$Res>  {
  factory $NotebookHomeStateCopyWith(NotebookHomeState value, $Res Function(NotebookHomeState) _then) = _$NotebookHomeStateCopyWithImpl;
@useResult
$Res call({
 String bookId, LoadStatus status, HomeWords? words, HomePanel? panel, HomeMap? map, Set<String> working, Map<String, String> trouble, bool copied, String? madeNoteId
});




}
/// @nodoc
class _$NotebookHomeStateCopyWithImpl<$Res>
    implements $NotebookHomeStateCopyWith<$Res> {
  _$NotebookHomeStateCopyWithImpl(this._self, this._then);

  final NotebookHomeState _self;
  final $Res Function(NotebookHomeState) _then;

/// Create a copy of NotebookHomeState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? bookId = null,Object? status = null,Object? words = freezed,Object? panel = freezed,Object? map = freezed,Object? working = null,Object? trouble = null,Object? copied = null,Object? madeNoteId = freezed,}) {
  return _then(_self.copyWith(
bookId: null == bookId ? _self.bookId : bookId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,words: freezed == words ? _self.words : words // ignore: cast_nullable_to_non_nullable
as HomeWords?,panel: freezed == panel ? _self.panel : panel // ignore: cast_nullable_to_non_nullable
as HomePanel?,map: freezed == map ? _self.map : map // ignore: cast_nullable_to_non_nullable
as HomeMap?,working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as Set<String>,trouble: null == trouble ? _self.trouble : trouble // ignore: cast_nullable_to_non_nullable
as Map<String, String>,copied: null == copied ? _self.copied : copied // ignore: cast_nullable_to_non_nullable
as bool,madeNoteId: freezed == madeNoteId ? _self.madeNoteId : madeNoteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [NotebookHomeState].
extension NotebookHomeStatePatterns on NotebookHomeState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NotebookHomeState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NotebookHomeState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NotebookHomeState value)  $default,){
final _that = this;
switch (_that) {
case _NotebookHomeState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NotebookHomeState value)?  $default,){
final _that = this;
switch (_that) {
case _NotebookHomeState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String bookId,  LoadStatus status,  HomeWords? words,  HomePanel? panel,  HomeMap? map,  Set<String> working,  Map<String, String> trouble,  bool copied,  String? madeNoteId)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NotebookHomeState() when $default != null:
return $default(_that.bookId,_that.status,_that.words,_that.panel,_that.map,_that.working,_that.trouble,_that.copied,_that.madeNoteId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String bookId,  LoadStatus status,  HomeWords? words,  HomePanel? panel,  HomeMap? map,  Set<String> working,  Map<String, String> trouble,  bool copied,  String? madeNoteId)  $default,) {final _that = this;
switch (_that) {
case _NotebookHomeState():
return $default(_that.bookId,_that.status,_that.words,_that.panel,_that.map,_that.working,_that.trouble,_that.copied,_that.madeNoteId);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String bookId,  LoadStatus status,  HomeWords? words,  HomePanel? panel,  HomeMap? map,  Set<String> working,  Map<String, String> trouble,  bool copied,  String? madeNoteId)?  $default,) {final _that = this;
switch (_that) {
case _NotebookHomeState() when $default != null:
return $default(_that.bookId,_that.status,_that.words,_that.panel,_that.map,_that.working,_that.trouble,_that.copied,_that.madeNoteId);case _:
  return null;

}
}

}

/// @nodoc


class _NotebookHomeState implements NotebookHomeState {
  const _NotebookHomeState({this.bookId = '', this.status = LoadStatus.loading, this.words, this.panel, this.map, final  Set<String> working = const <String>{}, final  Map<String, String> trouble = const <String, String>{}, this.copied = false, this.madeNoteId}): _working = working,_trouble = trouble;
  

@override@JsonKey() final  String bookId;
@override@JsonKey() final  LoadStatus status;
/// The core's own words; null until read.
@override final  HomeWords? words;
@override final  HomePanel? panel;
/// The map, read after the sections are shown; null until then.
@override final  HomeMap? map;
/// Section keys being rewritten now.
 final  Set<String> _working;
/// Section keys being rewritten now.
@override@JsonKey() Set<String> get working {
  if (_working is EqualUnmodifiableSetView) return _working;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_working);
}

/// Section keys that failed to rewrite, with the model's own words.
 final  Map<String, String> _trouble;
/// Section keys that failed to rewrite, with the model's own words.
@override@JsonKey() Map<String, String> get trouble {
  if (_trouble is EqualUnmodifiableMapView) return _trouble;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_trouble);
}

/// A section was copied into the Home note: the page says so once.
@override@JsonKey() final  bool copied;
/// A new Home note was made: the page opens it once.
@override final  String? madeNoteId;

/// Create a copy of NotebookHomeState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NotebookHomeStateCopyWith<_NotebookHomeState> get copyWith => __$NotebookHomeStateCopyWithImpl<_NotebookHomeState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NotebookHomeState&&(identical(other.bookId, bookId) || other.bookId == bookId)&&(identical(other.status, status) || other.status == status)&&(identical(other.words, words) || other.words == words)&&(identical(other.panel, panel) || other.panel == panel)&&(identical(other.map, map) || other.map == map)&&const DeepCollectionEquality().equals(other._working, _working)&&const DeepCollectionEquality().equals(other._trouble, _trouble)&&(identical(other.copied, copied) || other.copied == copied)&&(identical(other.madeNoteId, madeNoteId) || other.madeNoteId == madeNoteId));
}


@override
int get hashCode => Object.hash(runtimeType,bookId,status,words,panel,map,const DeepCollectionEquality().hash(_working),const DeepCollectionEquality().hash(_trouble),copied,madeNoteId);

@override
String toString() {
  return 'NotebookHomeState(bookId: $bookId, status: $status, words: $words, panel: $panel, map: $map, working: $working, trouble: $trouble, copied: $copied, madeNoteId: $madeNoteId)';
}


}

/// @nodoc
abstract mixin class _$NotebookHomeStateCopyWith<$Res> implements $NotebookHomeStateCopyWith<$Res> {
  factory _$NotebookHomeStateCopyWith(_NotebookHomeState value, $Res Function(_NotebookHomeState) _then) = __$NotebookHomeStateCopyWithImpl;
@override @useResult
$Res call({
 String bookId, LoadStatus status, HomeWords? words, HomePanel? panel, HomeMap? map, Set<String> working, Map<String, String> trouble, bool copied, String? madeNoteId
});




}
/// @nodoc
class __$NotebookHomeStateCopyWithImpl<$Res>
    implements _$NotebookHomeStateCopyWith<$Res> {
  __$NotebookHomeStateCopyWithImpl(this._self, this._then);

  final _NotebookHomeState _self;
  final $Res Function(_NotebookHomeState) _then;

/// Create a copy of NotebookHomeState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? bookId = null,Object? status = null,Object? words = freezed,Object? panel = freezed,Object? map = freezed,Object? working = null,Object? trouble = null,Object? copied = null,Object? madeNoteId = freezed,}) {
  return _then(_NotebookHomeState(
bookId: null == bookId ? _self.bookId : bookId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,words: freezed == words ? _self.words : words // ignore: cast_nullable_to_non_nullable
as HomeWords?,panel: freezed == panel ? _self.panel : panel // ignore: cast_nullable_to_non_nullable
as HomePanel?,map: freezed == map ? _self.map : map // ignore: cast_nullable_to_non_nullable
as HomeMap?,working: null == working ? _self._working : working // ignore: cast_nullable_to_non_nullable
as Set<String>,trouble: null == trouble ? _self._trouble : trouble // ignore: cast_nullable_to_non_nullable
as Map<String, String>,copied: null == copied ? _self.copied : copied // ignore: cast_nullable_to_non_nullable
as bool,madeNoteId: freezed == madeNoteId ? _self.madeNoteId : madeNoteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
