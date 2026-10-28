// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'tabs_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$TabsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TabsEvent()';
}


}

/// @nodoc
class $TabsEventCopyWith<$Res>  {
$TabsEventCopyWith(TabsEvent _, $Res Function(TabsEvent) __);
}


/// Adds pattern-matching-related methods to [TabsEvent].
extension TabsEventPatterns on TabsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( TabsStartedEvent value)?  started,TResult Function( TabsEnabledChangedEvent value)?  enabledChanged,TResult Function( TabsNoteOpenedEvent value)?  noteOpened,TResult Function( TabsOpenedInNewTabEvent value)?  openedInNewTab,TResult Function( TabsActivatedEvent value)?  activated,TResult Function( TabsClosedEvent value)?  closed,TResult Function( TabsOthersClosedEvent value)?  othersClosed,TResult Function( TabsRightClosedEvent value)?  rightClosed,TResult Function( TabsReorderedEvent value)?  reordered,TResult Function( TabsReopenRequestedEvent value)?  reopenRequested,TResult Function( TabsNextRequestedEvent value)?  nextRequested,TResult Function( TabsPreviousRequestedEvent value)?  previousRequested,TResult Function( TabsNumberedEvent value)?  numbered,TResult Function( TabsStepRequestedEvent value)?  stepRequested,TResult Function( TabsConsumedEvent value)?  consumed,TResult Function( TabsChangesReceivedEvent value)?  changesReceived,required TResult orElse(),}){
final _that = this;
switch (_that) {
case TabsStartedEvent() when started != null:
return started(_that);case TabsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that);case TabsNoteOpenedEvent() when noteOpened != null:
return noteOpened(_that);case TabsOpenedInNewTabEvent() when openedInNewTab != null:
return openedInNewTab(_that);case TabsActivatedEvent() when activated != null:
return activated(_that);case TabsClosedEvent() when closed != null:
return closed(_that);case TabsOthersClosedEvent() when othersClosed != null:
return othersClosed(_that);case TabsRightClosedEvent() when rightClosed != null:
return rightClosed(_that);case TabsReorderedEvent() when reordered != null:
return reordered(_that);case TabsReopenRequestedEvent() when reopenRequested != null:
return reopenRequested(_that);case TabsNextRequestedEvent() when nextRequested != null:
return nextRequested(_that);case TabsPreviousRequestedEvent() when previousRequested != null:
return previousRequested(_that);case TabsNumberedEvent() when numbered != null:
return numbered(_that);case TabsStepRequestedEvent() when stepRequested != null:
return stepRequested(_that);case TabsConsumedEvent() when consumed != null:
return consumed(_that);case TabsChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( TabsStartedEvent value)  started,required TResult Function( TabsEnabledChangedEvent value)  enabledChanged,required TResult Function( TabsNoteOpenedEvent value)  noteOpened,required TResult Function( TabsOpenedInNewTabEvent value)  openedInNewTab,required TResult Function( TabsActivatedEvent value)  activated,required TResult Function( TabsClosedEvent value)  closed,required TResult Function( TabsOthersClosedEvent value)  othersClosed,required TResult Function( TabsRightClosedEvent value)  rightClosed,required TResult Function( TabsReorderedEvent value)  reordered,required TResult Function( TabsReopenRequestedEvent value)  reopenRequested,required TResult Function( TabsNextRequestedEvent value)  nextRequested,required TResult Function( TabsPreviousRequestedEvent value)  previousRequested,required TResult Function( TabsNumberedEvent value)  numbered,required TResult Function( TabsStepRequestedEvent value)  stepRequested,required TResult Function( TabsConsumedEvent value)  consumed,required TResult Function( TabsChangesReceivedEvent value)  changesReceived,}){
final _that = this;
switch (_that) {
case TabsStartedEvent():
return started(_that);case TabsEnabledChangedEvent():
return enabledChanged(_that);case TabsNoteOpenedEvent():
return noteOpened(_that);case TabsOpenedInNewTabEvent():
return openedInNewTab(_that);case TabsActivatedEvent():
return activated(_that);case TabsClosedEvent():
return closed(_that);case TabsOthersClosedEvent():
return othersClosed(_that);case TabsRightClosedEvent():
return rightClosed(_that);case TabsReorderedEvent():
return reordered(_that);case TabsReopenRequestedEvent():
return reopenRequested(_that);case TabsNextRequestedEvent():
return nextRequested(_that);case TabsPreviousRequestedEvent():
return previousRequested(_that);case TabsNumberedEvent():
return numbered(_that);case TabsStepRequestedEvent():
return stepRequested(_that);case TabsConsumedEvent():
return consumed(_that);case TabsChangesReceivedEvent():
return changesReceived(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( TabsStartedEvent value)?  started,TResult? Function( TabsEnabledChangedEvent value)?  enabledChanged,TResult? Function( TabsNoteOpenedEvent value)?  noteOpened,TResult? Function( TabsOpenedInNewTabEvent value)?  openedInNewTab,TResult? Function( TabsActivatedEvent value)?  activated,TResult? Function( TabsClosedEvent value)?  closed,TResult? Function( TabsOthersClosedEvent value)?  othersClosed,TResult? Function( TabsRightClosedEvent value)?  rightClosed,TResult? Function( TabsReorderedEvent value)?  reordered,TResult? Function( TabsReopenRequestedEvent value)?  reopenRequested,TResult? Function( TabsNextRequestedEvent value)?  nextRequested,TResult? Function( TabsPreviousRequestedEvent value)?  previousRequested,TResult? Function( TabsNumberedEvent value)?  numbered,TResult? Function( TabsStepRequestedEvent value)?  stepRequested,TResult? Function( TabsConsumedEvent value)?  consumed,TResult? Function( TabsChangesReceivedEvent value)?  changesReceived,}){
final _that = this;
switch (_that) {
case TabsStartedEvent() when started != null:
return started(_that);case TabsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that);case TabsNoteOpenedEvent() when noteOpened != null:
return noteOpened(_that);case TabsOpenedInNewTabEvent() when openedInNewTab != null:
return openedInNewTab(_that);case TabsActivatedEvent() when activated != null:
return activated(_that);case TabsClosedEvent() when closed != null:
return closed(_that);case TabsOthersClosedEvent() when othersClosed != null:
return othersClosed(_that);case TabsRightClosedEvent() when rightClosed != null:
return rightClosed(_that);case TabsReorderedEvent() when reordered != null:
return reordered(_that);case TabsReopenRequestedEvent() when reopenRequested != null:
return reopenRequested(_that);case TabsNextRequestedEvent() when nextRequested != null:
return nextRequested(_that);case TabsPreviousRequestedEvent() when previousRequested != null:
return previousRequested(_that);case TabsNumberedEvent() when numbered != null:
return numbered(_that);case TabsStepRequestedEvent() when stepRequested != null:
return stepRequested(_that);case TabsConsumedEvent() when consumed != null:
return consumed(_that);case TabsChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( bool enabled,  String? shownNoteId)?  enabledChanged,TResult Function( String noteId)?  noteOpened,TResult Function( String noteId)?  openedInNewTab,TResult Function( int index)?  activated,TResult Function( int index)?  closed,TResult Function( int index)?  othersClosed,TResult Function( int index)?  rightClosed,TResult Function( int from,  int to)?  reordered,TResult Function()?  reopenRequested,TResult Function()?  nextRequested,TResult Function()?  previousRequested,TResult Function( int number)?  numbered,TResult Function( bool back)?  stepRequested,TResult Function()?  consumed,TResult Function( List<String> ids)?  changesReceived,required TResult orElse(),}) {final _that = this;
switch (_that) {
case TabsStartedEvent() when started != null:
return started();case TabsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that.enabled,_that.shownNoteId);case TabsNoteOpenedEvent() when noteOpened != null:
return noteOpened(_that.noteId);case TabsOpenedInNewTabEvent() when openedInNewTab != null:
return openedInNewTab(_that.noteId);case TabsActivatedEvent() when activated != null:
return activated(_that.index);case TabsClosedEvent() when closed != null:
return closed(_that.index);case TabsOthersClosedEvent() when othersClosed != null:
return othersClosed(_that.index);case TabsRightClosedEvent() when rightClosed != null:
return rightClosed(_that.index);case TabsReorderedEvent() when reordered != null:
return reordered(_that.from,_that.to);case TabsReopenRequestedEvent() when reopenRequested != null:
return reopenRequested();case TabsNextRequestedEvent() when nextRequested != null:
return nextRequested();case TabsPreviousRequestedEvent() when previousRequested != null:
return previousRequested();case TabsNumberedEvent() when numbered != null:
return numbered(_that.number);case TabsStepRequestedEvent() when stepRequested != null:
return stepRequested(_that.back);case TabsConsumedEvent() when consumed != null:
return consumed();case TabsChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( bool enabled,  String? shownNoteId)  enabledChanged,required TResult Function( String noteId)  noteOpened,required TResult Function( String noteId)  openedInNewTab,required TResult Function( int index)  activated,required TResult Function( int index)  closed,required TResult Function( int index)  othersClosed,required TResult Function( int index)  rightClosed,required TResult Function( int from,  int to)  reordered,required TResult Function()  reopenRequested,required TResult Function()  nextRequested,required TResult Function()  previousRequested,required TResult Function( int number)  numbered,required TResult Function( bool back)  stepRequested,required TResult Function()  consumed,required TResult Function( List<String> ids)  changesReceived,}) {final _that = this;
switch (_that) {
case TabsStartedEvent():
return started();case TabsEnabledChangedEvent():
return enabledChanged(_that.enabled,_that.shownNoteId);case TabsNoteOpenedEvent():
return noteOpened(_that.noteId);case TabsOpenedInNewTabEvent():
return openedInNewTab(_that.noteId);case TabsActivatedEvent():
return activated(_that.index);case TabsClosedEvent():
return closed(_that.index);case TabsOthersClosedEvent():
return othersClosed(_that.index);case TabsRightClosedEvent():
return rightClosed(_that.index);case TabsReorderedEvent():
return reordered(_that.from,_that.to);case TabsReopenRequestedEvent():
return reopenRequested();case TabsNextRequestedEvent():
return nextRequested();case TabsPreviousRequestedEvent():
return previousRequested();case TabsNumberedEvent():
return numbered(_that.number);case TabsStepRequestedEvent():
return stepRequested(_that.back);case TabsConsumedEvent():
return consumed();case TabsChangesReceivedEvent():
return changesReceived(_that.ids);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( bool enabled,  String? shownNoteId)?  enabledChanged,TResult? Function( String noteId)?  noteOpened,TResult? Function( String noteId)?  openedInNewTab,TResult? Function( int index)?  activated,TResult? Function( int index)?  closed,TResult? Function( int index)?  othersClosed,TResult? Function( int index)?  rightClosed,TResult? Function( int from,  int to)?  reordered,TResult? Function()?  reopenRequested,TResult? Function()?  nextRequested,TResult? Function()?  previousRequested,TResult? Function( int number)?  numbered,TResult? Function( bool back)?  stepRequested,TResult? Function()?  consumed,TResult? Function( List<String> ids)?  changesReceived,}) {final _that = this;
switch (_that) {
case TabsStartedEvent() when started != null:
return started();case TabsEnabledChangedEvent() when enabledChanged != null:
return enabledChanged(_that.enabled,_that.shownNoteId);case TabsNoteOpenedEvent() when noteOpened != null:
return noteOpened(_that.noteId);case TabsOpenedInNewTabEvent() when openedInNewTab != null:
return openedInNewTab(_that.noteId);case TabsActivatedEvent() when activated != null:
return activated(_that.index);case TabsClosedEvent() when closed != null:
return closed(_that.index);case TabsOthersClosedEvent() when othersClosed != null:
return othersClosed(_that.index);case TabsRightClosedEvent() when rightClosed != null:
return rightClosed(_that.index);case TabsReorderedEvent() when reordered != null:
return reordered(_that.from,_that.to);case TabsReopenRequestedEvent() when reopenRequested != null:
return reopenRequested();case TabsNextRequestedEvent() when nextRequested != null:
return nextRequested();case TabsPreviousRequestedEvent() when previousRequested != null:
return previousRequested();case TabsNumberedEvent() when numbered != null:
return numbered(_that.number);case TabsStepRequestedEvent() when stepRequested != null:
return stepRequested(_that.back);case TabsConsumedEvent() when consumed != null:
return consumed();case TabsChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case _:
  return null;

}
}

}

/// @nodoc


class TabsStartedEvent implements TabsEvent {
  const TabsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TabsEvent.started()';
}


}




/// @nodoc


class TabsEnabledChangedEvent implements TabsEvent {
  const TabsEnabledChangedEvent({required this.enabled, this.shownNoteId});
  

 final  bool enabled;
 final  String? shownNoteId;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsEnabledChangedEventCopyWith<TabsEnabledChangedEvent> get copyWith => _$TabsEnabledChangedEventCopyWithImpl<TabsEnabledChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsEnabledChangedEvent&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.shownNoteId, shownNoteId) || other.shownNoteId == shownNoteId));
}


@override
int get hashCode => Object.hash(runtimeType,enabled,shownNoteId);

@override
String toString() {
  return 'TabsEvent.enabledChanged(enabled: $enabled, shownNoteId: $shownNoteId)';
}


}

/// @nodoc
abstract mixin class $TabsEnabledChangedEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsEnabledChangedEventCopyWith(TabsEnabledChangedEvent value, $Res Function(TabsEnabledChangedEvent) _then) = _$TabsEnabledChangedEventCopyWithImpl;
@useResult
$Res call({
 bool enabled, String? shownNoteId
});




}
/// @nodoc
class _$TabsEnabledChangedEventCopyWithImpl<$Res>
    implements $TabsEnabledChangedEventCopyWith<$Res> {
  _$TabsEnabledChangedEventCopyWithImpl(this._self, this._then);

  final TabsEnabledChangedEvent _self;
  final $Res Function(TabsEnabledChangedEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? enabled = null,Object? shownNoteId = freezed,}) {
  return _then(TabsEnabledChangedEvent(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,shownNoteId: freezed == shownNoteId ? _self.shownNoteId : shownNoteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class TabsNoteOpenedEvent implements TabsEvent {
  const TabsNoteOpenedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsNoteOpenedEventCopyWith<TabsNoteOpenedEvent> get copyWith => _$TabsNoteOpenedEventCopyWithImpl<TabsNoteOpenedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsNoteOpenedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'TabsEvent.noteOpened(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $TabsNoteOpenedEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsNoteOpenedEventCopyWith(TabsNoteOpenedEvent value, $Res Function(TabsNoteOpenedEvent) _then) = _$TabsNoteOpenedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$TabsNoteOpenedEventCopyWithImpl<$Res>
    implements $TabsNoteOpenedEventCopyWith<$Res> {
  _$TabsNoteOpenedEventCopyWithImpl(this._self, this._then);

  final TabsNoteOpenedEvent _self;
  final $Res Function(TabsNoteOpenedEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(TabsNoteOpenedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class TabsOpenedInNewTabEvent implements TabsEvent {
  const TabsOpenedInNewTabEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsOpenedInNewTabEventCopyWith<TabsOpenedInNewTabEvent> get copyWith => _$TabsOpenedInNewTabEventCopyWithImpl<TabsOpenedInNewTabEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsOpenedInNewTabEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'TabsEvent.openedInNewTab(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $TabsOpenedInNewTabEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsOpenedInNewTabEventCopyWith(TabsOpenedInNewTabEvent value, $Res Function(TabsOpenedInNewTabEvent) _then) = _$TabsOpenedInNewTabEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$TabsOpenedInNewTabEventCopyWithImpl<$Res>
    implements $TabsOpenedInNewTabEventCopyWith<$Res> {
  _$TabsOpenedInNewTabEventCopyWithImpl(this._self, this._then);

  final TabsOpenedInNewTabEvent _self;
  final $Res Function(TabsOpenedInNewTabEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(TabsOpenedInNewTabEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class TabsActivatedEvent implements TabsEvent {
  const TabsActivatedEvent(this.index);
  

 final  int index;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsActivatedEventCopyWith<TabsActivatedEvent> get copyWith => _$TabsActivatedEventCopyWithImpl<TabsActivatedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsActivatedEvent&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,index);

@override
String toString() {
  return 'TabsEvent.activated(index: $index)';
}


}

/// @nodoc
abstract mixin class $TabsActivatedEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsActivatedEventCopyWith(TabsActivatedEvent value, $Res Function(TabsActivatedEvent) _then) = _$TabsActivatedEventCopyWithImpl;
@useResult
$Res call({
 int index
});




}
/// @nodoc
class _$TabsActivatedEventCopyWithImpl<$Res>
    implements $TabsActivatedEventCopyWith<$Res> {
  _$TabsActivatedEventCopyWithImpl(this._self, this._then);

  final TabsActivatedEvent _self;
  final $Res Function(TabsActivatedEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? index = null,}) {
  return _then(TabsActivatedEvent(
null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class TabsClosedEvent implements TabsEvent {
  const TabsClosedEvent(this.index);
  

 final  int index;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsClosedEventCopyWith<TabsClosedEvent> get copyWith => _$TabsClosedEventCopyWithImpl<TabsClosedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsClosedEvent&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,index);

@override
String toString() {
  return 'TabsEvent.closed(index: $index)';
}


}

/// @nodoc
abstract mixin class $TabsClosedEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsClosedEventCopyWith(TabsClosedEvent value, $Res Function(TabsClosedEvent) _then) = _$TabsClosedEventCopyWithImpl;
@useResult
$Res call({
 int index
});




}
/// @nodoc
class _$TabsClosedEventCopyWithImpl<$Res>
    implements $TabsClosedEventCopyWith<$Res> {
  _$TabsClosedEventCopyWithImpl(this._self, this._then);

  final TabsClosedEvent _self;
  final $Res Function(TabsClosedEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? index = null,}) {
  return _then(TabsClosedEvent(
null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class TabsOthersClosedEvent implements TabsEvent {
  const TabsOthersClosedEvent(this.index);
  

 final  int index;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsOthersClosedEventCopyWith<TabsOthersClosedEvent> get copyWith => _$TabsOthersClosedEventCopyWithImpl<TabsOthersClosedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsOthersClosedEvent&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,index);

@override
String toString() {
  return 'TabsEvent.othersClosed(index: $index)';
}


}

/// @nodoc
abstract mixin class $TabsOthersClosedEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsOthersClosedEventCopyWith(TabsOthersClosedEvent value, $Res Function(TabsOthersClosedEvent) _then) = _$TabsOthersClosedEventCopyWithImpl;
@useResult
$Res call({
 int index
});




}
/// @nodoc
class _$TabsOthersClosedEventCopyWithImpl<$Res>
    implements $TabsOthersClosedEventCopyWith<$Res> {
  _$TabsOthersClosedEventCopyWithImpl(this._self, this._then);

  final TabsOthersClosedEvent _self;
  final $Res Function(TabsOthersClosedEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? index = null,}) {
  return _then(TabsOthersClosedEvent(
null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class TabsRightClosedEvent implements TabsEvent {
  const TabsRightClosedEvent(this.index);
  

 final  int index;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsRightClosedEventCopyWith<TabsRightClosedEvent> get copyWith => _$TabsRightClosedEventCopyWithImpl<TabsRightClosedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsRightClosedEvent&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,index);

@override
String toString() {
  return 'TabsEvent.rightClosed(index: $index)';
}


}

/// @nodoc
abstract mixin class $TabsRightClosedEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsRightClosedEventCopyWith(TabsRightClosedEvent value, $Res Function(TabsRightClosedEvent) _then) = _$TabsRightClosedEventCopyWithImpl;
@useResult
$Res call({
 int index
});




}
/// @nodoc
class _$TabsRightClosedEventCopyWithImpl<$Res>
    implements $TabsRightClosedEventCopyWith<$Res> {
  _$TabsRightClosedEventCopyWithImpl(this._self, this._then);

  final TabsRightClosedEvent _self;
  final $Res Function(TabsRightClosedEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? index = null,}) {
  return _then(TabsRightClosedEvent(
null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class TabsReorderedEvent implements TabsEvent {
  const TabsReorderedEvent(this.from, this.to);
  

 final  int from;
 final  int to;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsReorderedEventCopyWith<TabsReorderedEvent> get copyWith => _$TabsReorderedEventCopyWithImpl<TabsReorderedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsReorderedEvent&&(identical(other.from, from) || other.from == from)&&(identical(other.to, to) || other.to == to));
}


@override
int get hashCode => Object.hash(runtimeType,from,to);

@override
String toString() {
  return 'TabsEvent.reordered(from: $from, to: $to)';
}


}

/// @nodoc
abstract mixin class $TabsReorderedEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsReorderedEventCopyWith(TabsReorderedEvent value, $Res Function(TabsReorderedEvent) _then) = _$TabsReorderedEventCopyWithImpl;
@useResult
$Res call({
 int from, int to
});




}
/// @nodoc
class _$TabsReorderedEventCopyWithImpl<$Res>
    implements $TabsReorderedEventCopyWith<$Res> {
  _$TabsReorderedEventCopyWithImpl(this._self, this._then);

  final TabsReorderedEvent _self;
  final $Res Function(TabsReorderedEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? from = null,Object? to = null,}) {
  return _then(TabsReorderedEvent(
null == from ? _self.from : from // ignore: cast_nullable_to_non_nullable
as int,null == to ? _self.to : to // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class TabsReopenRequestedEvent implements TabsEvent {
  const TabsReopenRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsReopenRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TabsEvent.reopenRequested()';
}


}




/// @nodoc


class TabsNextRequestedEvent implements TabsEvent {
  const TabsNextRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsNextRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TabsEvent.nextRequested()';
}


}




/// @nodoc


class TabsPreviousRequestedEvent implements TabsEvent {
  const TabsPreviousRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsPreviousRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TabsEvent.previousRequested()';
}


}




/// @nodoc


class TabsNumberedEvent implements TabsEvent {
  const TabsNumberedEvent(this.number);
  

 final  int number;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsNumberedEventCopyWith<TabsNumberedEvent> get copyWith => _$TabsNumberedEventCopyWithImpl<TabsNumberedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsNumberedEvent&&(identical(other.number, number) || other.number == number));
}


@override
int get hashCode => Object.hash(runtimeType,number);

@override
String toString() {
  return 'TabsEvent.numbered(number: $number)';
}


}

/// @nodoc
abstract mixin class $TabsNumberedEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsNumberedEventCopyWith(TabsNumberedEvent value, $Res Function(TabsNumberedEvent) _then) = _$TabsNumberedEventCopyWithImpl;
@useResult
$Res call({
 int number
});




}
/// @nodoc
class _$TabsNumberedEventCopyWithImpl<$Res>
    implements $TabsNumberedEventCopyWith<$Res> {
  _$TabsNumberedEventCopyWithImpl(this._self, this._then);

  final TabsNumberedEvent _self;
  final $Res Function(TabsNumberedEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? number = null,}) {
  return _then(TabsNumberedEvent(
null == number ? _self.number : number // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class TabsStepRequestedEvent implements TabsEvent {
  const TabsStepRequestedEvent({required this.back});
  

 final  bool back;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsStepRequestedEventCopyWith<TabsStepRequestedEvent> get copyWith => _$TabsStepRequestedEventCopyWithImpl<TabsStepRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsStepRequestedEvent&&(identical(other.back, back) || other.back == back));
}


@override
int get hashCode => Object.hash(runtimeType,back);

@override
String toString() {
  return 'TabsEvent.stepRequested(back: $back)';
}


}

/// @nodoc
abstract mixin class $TabsStepRequestedEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsStepRequestedEventCopyWith(TabsStepRequestedEvent value, $Res Function(TabsStepRequestedEvent) _then) = _$TabsStepRequestedEventCopyWithImpl;
@useResult
$Res call({
 bool back
});




}
/// @nodoc
class _$TabsStepRequestedEventCopyWithImpl<$Res>
    implements $TabsStepRequestedEventCopyWith<$Res> {
  _$TabsStepRequestedEventCopyWithImpl(this._self, this._then);

  final TabsStepRequestedEvent _self;
  final $Res Function(TabsStepRequestedEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? back = null,}) {
  return _then(TabsStepRequestedEvent(
back: null == back ? _self.back : back // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class TabsConsumedEvent implements TabsEvent {
  const TabsConsumedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsConsumedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TabsEvent.consumed()';
}


}




/// @nodoc


class TabsChangesReceivedEvent implements TabsEvent {
  const TabsChangesReceivedEvent(final  List<String> ids): _ids = ids;
  

 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsChangesReceivedEventCopyWith<TabsChangesReceivedEvent> get copyWith => _$TabsChangesReceivedEventCopyWithImpl<TabsChangesReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsChangesReceivedEvent&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'TabsEvent.changesReceived(ids: $ids)';
}


}

/// @nodoc
abstract mixin class $TabsChangesReceivedEventCopyWith<$Res> implements $TabsEventCopyWith<$Res> {
  factory $TabsChangesReceivedEventCopyWith(TabsChangesReceivedEvent value, $Res Function(TabsChangesReceivedEvent) _then) = _$TabsChangesReceivedEventCopyWithImpl;
@useResult
$Res call({
 List<String> ids
});




}
/// @nodoc
class _$TabsChangesReceivedEventCopyWithImpl<$Res>
    implements $TabsChangesReceivedEventCopyWith<$Res> {
  _$TabsChangesReceivedEventCopyWithImpl(this._self, this._then);

  final TabsChangesReceivedEvent _self;
  final $Res Function(TabsChangesReceivedEvent) _then;

/// Create a copy of TabsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ids = null,}) {
  return _then(TabsChangesReceivedEvent(
null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc
mixin _$TabsState {

/// The open tabs, the one showing, and the ones that can be reopened.
 OpenTabs get tabs;/// Whether the window has panes for tabs. Off, the tabs are dormant: a
/// phone has none, and navigation is the app's one history.
 bool get enabled;/// The note the tabs want shown, for the screen to open in the place
/// where the person is; set when a tab was chosen, closed, reopened or
/// stepped, never for a note the address already shows.
 String? get pendingNoteId;
/// Create a copy of TabsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabsStateCopyWith<TabsState> get copyWith => _$TabsStateCopyWithImpl<TabsState>(this as TabsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabsState&&(identical(other.tabs, tabs) || other.tabs == tabs)&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.pendingNoteId, pendingNoteId) || other.pendingNoteId == pendingNoteId));
}


@override
int get hashCode => Object.hash(runtimeType,tabs,enabled,pendingNoteId);

@override
String toString() {
  return 'TabsState(tabs: $tabs, enabled: $enabled, pendingNoteId: $pendingNoteId)';
}


}

/// @nodoc
abstract mixin class $TabsStateCopyWith<$Res>  {
  factory $TabsStateCopyWith(TabsState value, $Res Function(TabsState) _then) = _$TabsStateCopyWithImpl;
@useResult
$Res call({
 OpenTabs tabs, bool enabled, String? pendingNoteId
});


$OpenTabsCopyWith<$Res> get tabs;

}
/// @nodoc
class _$TabsStateCopyWithImpl<$Res>
    implements $TabsStateCopyWith<$Res> {
  _$TabsStateCopyWithImpl(this._self, this._then);

  final TabsState _self;
  final $Res Function(TabsState) _then;

/// Create a copy of TabsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? tabs = null,Object? enabled = null,Object? pendingNoteId = freezed,}) {
  return _then(_self.copyWith(
tabs: null == tabs ? _self.tabs : tabs // ignore: cast_nullable_to_non_nullable
as OpenTabs,enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,pendingNoteId: freezed == pendingNoteId ? _self.pendingNoteId : pendingNoteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}
/// Create a copy of TabsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$OpenTabsCopyWith<$Res> get tabs {
  
  return $OpenTabsCopyWith<$Res>(_self.tabs, (value) {
    return _then(_self.copyWith(tabs: value));
  });
}
}


/// Adds pattern-matching-related methods to [TabsState].
extension TabsStatePatterns on TabsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TabsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TabsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TabsState value)  $default,){
final _that = this;
switch (_that) {
case _TabsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TabsState value)?  $default,){
final _that = this;
switch (_that) {
case _TabsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( OpenTabs tabs,  bool enabled,  String? pendingNoteId)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TabsState() when $default != null:
return $default(_that.tabs,_that.enabled,_that.pendingNoteId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( OpenTabs tabs,  bool enabled,  String? pendingNoteId)  $default,) {final _that = this;
switch (_that) {
case _TabsState():
return $default(_that.tabs,_that.enabled,_that.pendingNoteId);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( OpenTabs tabs,  bool enabled,  String? pendingNoteId)?  $default,) {final _that = this;
switch (_that) {
case _TabsState() when $default != null:
return $default(_that.tabs,_that.enabled,_that.pendingNoteId);case _:
  return null;

}
}

}

/// @nodoc


class _TabsState extends TabsState {
  const _TabsState({this.tabs = const OpenTabs(), this.enabled = false, this.pendingNoteId}): super._();
  

/// The open tabs, the one showing, and the ones that can be reopened.
@override@JsonKey() final  OpenTabs tabs;
/// Whether the window has panes for tabs. Off, the tabs are dormant: a
/// phone has none, and navigation is the app's one history.
@override@JsonKey() final  bool enabled;
/// The note the tabs want shown, for the screen to open in the place
/// where the person is; set when a tab was chosen, closed, reopened or
/// stepped, never for a note the address already shows.
@override final  String? pendingNoteId;

/// Create a copy of TabsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TabsStateCopyWith<_TabsState> get copyWith => __$TabsStateCopyWithImpl<_TabsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TabsState&&(identical(other.tabs, tabs) || other.tabs == tabs)&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.pendingNoteId, pendingNoteId) || other.pendingNoteId == pendingNoteId));
}


@override
int get hashCode => Object.hash(runtimeType,tabs,enabled,pendingNoteId);

@override
String toString() {
  return 'TabsState(tabs: $tabs, enabled: $enabled, pendingNoteId: $pendingNoteId)';
}


}

/// @nodoc
abstract mixin class _$TabsStateCopyWith<$Res> implements $TabsStateCopyWith<$Res> {
  factory _$TabsStateCopyWith(_TabsState value, $Res Function(_TabsState) _then) = __$TabsStateCopyWithImpl;
@override @useResult
$Res call({
 OpenTabs tabs, bool enabled, String? pendingNoteId
});


@override $OpenTabsCopyWith<$Res> get tabs;

}
/// @nodoc
class __$TabsStateCopyWithImpl<$Res>
    implements _$TabsStateCopyWith<$Res> {
  __$TabsStateCopyWithImpl(this._self, this._then);

  final _TabsState _self;
  final $Res Function(_TabsState) _then;

/// Create a copy of TabsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? tabs = null,Object? enabled = null,Object? pendingNoteId = freezed,}) {
  return _then(_TabsState(
tabs: null == tabs ? _self.tabs : tabs // ignore: cast_nullable_to_non_nullable
as OpenTabs,enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,pendingNoteId: freezed == pendingNoteId ? _self.pendingNoteId : pendingNoteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of TabsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$OpenTabsCopyWith<$Res> get tabs {
  
  return $OpenTabsCopyWith<$Res>(_self.tabs, (value) {
    return _then(_self.copyWith(tabs: value));
  });
}
}

// dart format on
