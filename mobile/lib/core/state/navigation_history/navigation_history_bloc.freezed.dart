// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'navigation_history_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NavigationHistoryEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NavigationHistoryEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NavigationHistoryEvent()';
}


}

/// @nodoc
class $NavigationHistoryEventCopyWith<$Res>  {
$NavigationHistoryEventCopyWith(NavigationHistoryEvent _, $Res Function(NavigationHistoryEvent) __);
}


/// Adds pattern-matching-related methods to [NavigationHistoryEvent].
extension NavigationHistoryEventPatterns on NavigationHistoryEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NavigationHistoryVisitedEvent value)?  visited,TResult Function( NavigationHistoryBackRequestedEvent value)?  backRequested,TResult Function( NavigationHistoryForwardRequestedEvent value)?  forwardRequested,TResult Function( NavigationHistoryConsumedEvent value)?  consumed,TResult Function( NavigationHistoryTabsChangedEvent value)?  tabsChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NavigationHistoryVisitedEvent() when visited != null:
return visited(_that);case NavigationHistoryBackRequestedEvent() when backRequested != null:
return backRequested(_that);case NavigationHistoryForwardRequestedEvent() when forwardRequested != null:
return forwardRequested(_that);case NavigationHistoryConsumedEvent() when consumed != null:
return consumed(_that);case NavigationHistoryTabsChangedEvent() when tabsChanged != null:
return tabsChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NavigationHistoryVisitedEvent value)  visited,required TResult Function( NavigationHistoryBackRequestedEvent value)  backRequested,required TResult Function( NavigationHistoryForwardRequestedEvent value)  forwardRequested,required TResult Function( NavigationHistoryConsumedEvent value)  consumed,required TResult Function( NavigationHistoryTabsChangedEvent value)  tabsChanged,}){
final _that = this;
switch (_that) {
case NavigationHistoryVisitedEvent():
return visited(_that);case NavigationHistoryBackRequestedEvent():
return backRequested(_that);case NavigationHistoryForwardRequestedEvent():
return forwardRequested(_that);case NavigationHistoryConsumedEvent():
return consumed(_that);case NavigationHistoryTabsChangedEvent():
return tabsChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NavigationHistoryVisitedEvent value)?  visited,TResult? Function( NavigationHistoryBackRequestedEvent value)?  backRequested,TResult? Function( NavigationHistoryForwardRequestedEvent value)?  forwardRequested,TResult? Function( NavigationHistoryConsumedEvent value)?  consumed,TResult? Function( NavigationHistoryTabsChangedEvent value)?  tabsChanged,}){
final _that = this;
switch (_that) {
case NavigationHistoryVisitedEvent() when visited != null:
return visited(_that);case NavigationHistoryBackRequestedEvent() when backRequested != null:
return backRequested(_that);case NavigationHistoryForwardRequestedEvent() when forwardRequested != null:
return forwardRequested(_that);case NavigationHistoryConsumedEvent() when consumed != null:
return consumed(_that);case NavigationHistoryTabsChangedEvent() when tabsChanged != null:
return tabsChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  visited,TResult Function()?  backRequested,TResult Function()?  forwardRequested,TResult Function()?  consumed,TResult Function()?  tabsChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NavigationHistoryVisitedEvent() when visited != null:
return visited(_that.noteId);case NavigationHistoryBackRequestedEvent() when backRequested != null:
return backRequested();case NavigationHistoryForwardRequestedEvent() when forwardRequested != null:
return forwardRequested();case NavigationHistoryConsumedEvent() when consumed != null:
return consumed();case NavigationHistoryTabsChangedEvent() when tabsChanged != null:
return tabsChanged();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  visited,required TResult Function()  backRequested,required TResult Function()  forwardRequested,required TResult Function()  consumed,required TResult Function()  tabsChanged,}) {final _that = this;
switch (_that) {
case NavigationHistoryVisitedEvent():
return visited(_that.noteId);case NavigationHistoryBackRequestedEvent():
return backRequested();case NavigationHistoryForwardRequestedEvent():
return forwardRequested();case NavigationHistoryConsumedEvent():
return consumed();case NavigationHistoryTabsChangedEvent():
return tabsChanged();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  visited,TResult? Function()?  backRequested,TResult? Function()?  forwardRequested,TResult? Function()?  consumed,TResult? Function()?  tabsChanged,}) {final _that = this;
switch (_that) {
case NavigationHistoryVisitedEvent() when visited != null:
return visited(_that.noteId);case NavigationHistoryBackRequestedEvent() when backRequested != null:
return backRequested();case NavigationHistoryForwardRequestedEvent() when forwardRequested != null:
return forwardRequested();case NavigationHistoryConsumedEvent() when consumed != null:
return consumed();case NavigationHistoryTabsChangedEvent() when tabsChanged != null:
return tabsChanged();case _:
  return null;

}
}

}

/// @nodoc


class NavigationHistoryVisitedEvent implements NavigationHistoryEvent {
  const NavigationHistoryVisitedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of NavigationHistoryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NavigationHistoryVisitedEventCopyWith<NavigationHistoryVisitedEvent> get copyWith => _$NavigationHistoryVisitedEventCopyWithImpl<NavigationHistoryVisitedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NavigationHistoryVisitedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'NavigationHistoryEvent.visited(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $NavigationHistoryVisitedEventCopyWith<$Res> implements $NavigationHistoryEventCopyWith<$Res> {
  factory $NavigationHistoryVisitedEventCopyWith(NavigationHistoryVisitedEvent value, $Res Function(NavigationHistoryVisitedEvent) _then) = _$NavigationHistoryVisitedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$NavigationHistoryVisitedEventCopyWithImpl<$Res>
    implements $NavigationHistoryVisitedEventCopyWith<$Res> {
  _$NavigationHistoryVisitedEventCopyWithImpl(this._self, this._then);

  final NavigationHistoryVisitedEvent _self;
  final $Res Function(NavigationHistoryVisitedEvent) _then;

/// Create a copy of NavigationHistoryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(NavigationHistoryVisitedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NavigationHistoryBackRequestedEvent implements NavigationHistoryEvent {
  const NavigationHistoryBackRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NavigationHistoryBackRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NavigationHistoryEvent.backRequested()';
}


}




/// @nodoc


class NavigationHistoryForwardRequestedEvent implements NavigationHistoryEvent {
  const NavigationHistoryForwardRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NavigationHistoryForwardRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NavigationHistoryEvent.forwardRequested()';
}


}




/// @nodoc


class NavigationHistoryConsumedEvent implements NavigationHistoryEvent {
  const NavigationHistoryConsumedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NavigationHistoryConsumedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NavigationHistoryEvent.consumed()';
}


}




/// @nodoc


class NavigationHistoryTabsChangedEvent implements NavigationHistoryEvent {
  const NavigationHistoryTabsChangedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NavigationHistoryTabsChangedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NavigationHistoryEvent.tabsChanged()';
}


}




/// @nodoc
mixin _$NavigationHistoryState {

/// The visits behind the open note, the open note and the visits ahead.
 NavigationHistory get history;/// The note going back or forward landed on, for the screen to open.
 String? get pendingNoteId;/// Whether going back would land on a note that is still there.
 bool get canGoBack;/// Whether going forward would land on a note that is still there.
 bool get canGoForward;
/// Create a copy of NavigationHistoryState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NavigationHistoryStateCopyWith<NavigationHistoryState> get copyWith => _$NavigationHistoryStateCopyWithImpl<NavigationHistoryState>(this as NavigationHistoryState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NavigationHistoryState&&(identical(other.history, history) || other.history == history)&&(identical(other.pendingNoteId, pendingNoteId) || other.pendingNoteId == pendingNoteId)&&(identical(other.canGoBack, canGoBack) || other.canGoBack == canGoBack)&&(identical(other.canGoForward, canGoForward) || other.canGoForward == canGoForward));
}


@override
int get hashCode => Object.hash(runtimeType,history,pendingNoteId,canGoBack,canGoForward);

@override
String toString() {
  return 'NavigationHistoryState(history: $history, pendingNoteId: $pendingNoteId, canGoBack: $canGoBack, canGoForward: $canGoForward)';
}


}

/// @nodoc
abstract mixin class $NavigationHistoryStateCopyWith<$Res>  {
  factory $NavigationHistoryStateCopyWith(NavigationHistoryState value, $Res Function(NavigationHistoryState) _then) = _$NavigationHistoryStateCopyWithImpl;
@useResult
$Res call({
 NavigationHistory history, String? pendingNoteId, bool canGoBack, bool canGoForward
});


$NavigationHistoryCopyWith<$Res> get history;

}
/// @nodoc
class _$NavigationHistoryStateCopyWithImpl<$Res>
    implements $NavigationHistoryStateCopyWith<$Res> {
  _$NavigationHistoryStateCopyWithImpl(this._self, this._then);

  final NavigationHistoryState _self;
  final $Res Function(NavigationHistoryState) _then;

/// Create a copy of NavigationHistoryState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? history = null,Object? pendingNoteId = freezed,Object? canGoBack = null,Object? canGoForward = null,}) {
  return _then(_self.copyWith(
history: null == history ? _self.history : history // ignore: cast_nullable_to_non_nullable
as NavigationHistory,pendingNoteId: freezed == pendingNoteId ? _self.pendingNoteId : pendingNoteId // ignore: cast_nullable_to_non_nullable
as String?,canGoBack: null == canGoBack ? _self.canGoBack : canGoBack // ignore: cast_nullable_to_non_nullable
as bool,canGoForward: null == canGoForward ? _self.canGoForward : canGoForward // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}
/// Create a copy of NavigationHistoryState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$NavigationHistoryCopyWith<$Res> get history {
  
  return $NavigationHistoryCopyWith<$Res>(_self.history, (value) {
    return _then(_self.copyWith(history: value));
  });
}
}


/// Adds pattern-matching-related methods to [NavigationHistoryState].
extension NavigationHistoryStatePatterns on NavigationHistoryState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NavigationHistoryState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NavigationHistoryState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NavigationHistoryState value)  $default,){
final _that = this;
switch (_that) {
case _NavigationHistoryState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NavigationHistoryState value)?  $default,){
final _that = this;
switch (_that) {
case _NavigationHistoryState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( NavigationHistory history,  String? pendingNoteId,  bool canGoBack,  bool canGoForward)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NavigationHistoryState() when $default != null:
return $default(_that.history,_that.pendingNoteId,_that.canGoBack,_that.canGoForward);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( NavigationHistory history,  String? pendingNoteId,  bool canGoBack,  bool canGoForward)  $default,) {final _that = this;
switch (_that) {
case _NavigationHistoryState():
return $default(_that.history,_that.pendingNoteId,_that.canGoBack,_that.canGoForward);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( NavigationHistory history,  String? pendingNoteId,  bool canGoBack,  bool canGoForward)?  $default,) {final _that = this;
switch (_that) {
case _NavigationHistoryState() when $default != null:
return $default(_that.history,_that.pendingNoteId,_that.canGoBack,_that.canGoForward);case _:
  return null;

}
}

}

/// @nodoc


class _NavigationHistoryState implements NavigationHistoryState {
  const _NavigationHistoryState({this.history = const NavigationHistory(), this.pendingNoteId, this.canGoBack = false, this.canGoForward = false});
  

/// The visits behind the open note, the open note and the visits ahead.
@override@JsonKey() final  NavigationHistory history;
/// The note going back or forward landed on, for the screen to open.
@override final  String? pendingNoteId;
/// Whether going back would land on a note that is still there.
@override@JsonKey() final  bool canGoBack;
/// Whether going forward would land on a note that is still there.
@override@JsonKey() final  bool canGoForward;

/// Create a copy of NavigationHistoryState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NavigationHistoryStateCopyWith<_NavigationHistoryState> get copyWith => __$NavigationHistoryStateCopyWithImpl<_NavigationHistoryState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NavigationHistoryState&&(identical(other.history, history) || other.history == history)&&(identical(other.pendingNoteId, pendingNoteId) || other.pendingNoteId == pendingNoteId)&&(identical(other.canGoBack, canGoBack) || other.canGoBack == canGoBack)&&(identical(other.canGoForward, canGoForward) || other.canGoForward == canGoForward));
}


@override
int get hashCode => Object.hash(runtimeType,history,pendingNoteId,canGoBack,canGoForward);

@override
String toString() {
  return 'NavigationHistoryState(history: $history, pendingNoteId: $pendingNoteId, canGoBack: $canGoBack, canGoForward: $canGoForward)';
}


}

/// @nodoc
abstract mixin class _$NavigationHistoryStateCopyWith<$Res> implements $NavigationHistoryStateCopyWith<$Res> {
  factory _$NavigationHistoryStateCopyWith(_NavigationHistoryState value, $Res Function(_NavigationHistoryState) _then) = __$NavigationHistoryStateCopyWithImpl;
@override @useResult
$Res call({
 NavigationHistory history, String? pendingNoteId, bool canGoBack, bool canGoForward
});


@override $NavigationHistoryCopyWith<$Res> get history;

}
/// @nodoc
class __$NavigationHistoryStateCopyWithImpl<$Res>
    implements _$NavigationHistoryStateCopyWith<$Res> {
  __$NavigationHistoryStateCopyWithImpl(this._self, this._then);

  final _NavigationHistoryState _self;
  final $Res Function(_NavigationHistoryState) _then;

/// Create a copy of NavigationHistoryState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? history = null,Object? pendingNoteId = freezed,Object? canGoBack = null,Object? canGoForward = null,}) {
  return _then(_NavigationHistoryState(
history: null == history ? _self.history : history // ignore: cast_nullable_to_non_nullable
as NavigationHistory,pendingNoteId: freezed == pendingNoteId ? _self.pendingNoteId : pendingNoteId // ignore: cast_nullable_to_non_nullable
as String?,canGoBack: null == canGoBack ? _self.canGoBack : canGoBack // ignore: cast_nullable_to_non_nullable
as bool,canGoForward: null == canGoForward ? _self.canGoForward : canGoForward // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

/// Create a copy of NavigationHistoryState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$NavigationHistoryCopyWith<$Res> get history {
  
  return $NavigationHistoryCopyWith<$Res>(_self.history, (value) {
    return _then(_self.copyWith(history: value));
  });
}
}

// dart format on
