// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'preview_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PreviewEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewEvent()';
}


}

/// @nodoc
class $PreviewEventCopyWith<$Res>  {
$PreviewEventCopyWith(PreviewEvent _, $Res Function(PreviewEvent) __);
}


/// Adds pattern-matching-related methods to [PreviewEvent].
extension PreviewEventPatterns on PreviewEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PreviewStartedEvent value)?  started,TResult Function( PreviewThemeChangedEvent value)?  themeChanged,TResult Function( PreviewChangesReceivedEvent value)?  changesReceived,TResult Function( PreviewLinkTappedEvent value)?  linkTapped,TResult Function( PreviewActionHandledEvent value)?  actionHandled,TResult Function( PreviewNoticeDismissedEvent value)?  noticeDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PreviewStartedEvent() when started != null:
return started(_that);case PreviewThemeChangedEvent() when themeChanged != null:
return themeChanged(_that);case PreviewChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case PreviewLinkTappedEvent() when linkTapped != null:
return linkTapped(_that);case PreviewActionHandledEvent() when actionHandled != null:
return actionHandled(_that);case PreviewNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PreviewStartedEvent value)  started,required TResult Function( PreviewThemeChangedEvent value)  themeChanged,required TResult Function( PreviewChangesReceivedEvent value)  changesReceived,required TResult Function( PreviewLinkTappedEvent value)  linkTapped,required TResult Function( PreviewActionHandledEvent value)  actionHandled,required TResult Function( PreviewNoticeDismissedEvent value)  noticeDismissed,}){
final _that = this;
switch (_that) {
case PreviewStartedEvent():
return started(_that);case PreviewThemeChangedEvent():
return themeChanged(_that);case PreviewChangesReceivedEvent():
return changesReceived(_that);case PreviewLinkTappedEvent():
return linkTapped(_that);case PreviewActionHandledEvent():
return actionHandled(_that);case PreviewNoticeDismissedEvent():
return noticeDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PreviewStartedEvent value)?  started,TResult? Function( PreviewThemeChangedEvent value)?  themeChanged,TResult? Function( PreviewChangesReceivedEvent value)?  changesReceived,TResult? Function( PreviewLinkTappedEvent value)?  linkTapped,TResult? Function( PreviewActionHandledEvent value)?  actionHandled,TResult? Function( PreviewNoticeDismissedEvent value)?  noticeDismissed,}){
final _that = this;
switch (_that) {
case PreviewStartedEvent() when started != null:
return started(_that);case PreviewThemeChangedEvent() when themeChanged != null:
return themeChanged(_that);case PreviewChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case PreviewLinkTappedEvent() when linkTapped != null:
return linkTapped(_that);case PreviewActionHandledEvent() when actionHandled != null:
return actionHandled(_that);case PreviewNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId,  String themeId)?  started,TResult Function( String themeId)?  themeChanged,TResult Function()?  changesReceived,TResult Function( String url)?  linkTapped,TResult Function()?  actionHandled,TResult Function()?  noticeDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PreviewStartedEvent() when started != null:
return started(_that.noteId,_that.themeId);case PreviewThemeChangedEvent() when themeChanged != null:
return themeChanged(_that.themeId);case PreviewChangesReceivedEvent() when changesReceived != null:
return changesReceived();case PreviewLinkTappedEvent() when linkTapped != null:
return linkTapped(_that.url);case PreviewActionHandledEvent() when actionHandled != null:
return actionHandled();case PreviewNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId,  String themeId)  started,required TResult Function( String themeId)  themeChanged,required TResult Function()  changesReceived,required TResult Function( String url)  linkTapped,required TResult Function()  actionHandled,required TResult Function()  noticeDismissed,}) {final _that = this;
switch (_that) {
case PreviewStartedEvent():
return started(_that.noteId,_that.themeId);case PreviewThemeChangedEvent():
return themeChanged(_that.themeId);case PreviewChangesReceivedEvent():
return changesReceived();case PreviewLinkTappedEvent():
return linkTapped(_that.url);case PreviewActionHandledEvent():
return actionHandled();case PreviewNoticeDismissedEvent():
return noticeDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId,  String themeId)?  started,TResult? Function( String themeId)?  themeChanged,TResult? Function()?  changesReceived,TResult? Function( String url)?  linkTapped,TResult? Function()?  actionHandled,TResult? Function()?  noticeDismissed,}) {final _that = this;
switch (_that) {
case PreviewStartedEvent() when started != null:
return started(_that.noteId,_that.themeId);case PreviewThemeChangedEvent() when themeChanged != null:
return themeChanged(_that.themeId);case PreviewChangesReceivedEvent() when changesReceived != null:
return changesReceived();case PreviewLinkTappedEvent() when linkTapped != null:
return linkTapped(_that.url);case PreviewActionHandledEvent() when actionHandled != null:
return actionHandled();case PreviewNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case _:
  return null;

}
}

}

/// @nodoc


class PreviewStartedEvent implements PreviewEvent {
  const PreviewStartedEvent(this.noteId, this.themeId);
  

 final  String noteId;
 final  String themeId;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewStartedEventCopyWith<PreviewStartedEvent> get copyWith => _$PreviewStartedEventCopyWithImpl<PreviewStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.themeId, themeId) || other.themeId == themeId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,themeId);

@override
String toString() {
  return 'PreviewEvent.started(noteId: $noteId, themeId: $themeId)';
}


}

/// @nodoc
abstract mixin class $PreviewStartedEventCopyWith<$Res> implements $PreviewEventCopyWith<$Res> {
  factory $PreviewStartedEventCopyWith(PreviewStartedEvent value, $Res Function(PreviewStartedEvent) _then) = _$PreviewStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId, String themeId
});




}
/// @nodoc
class _$PreviewStartedEventCopyWithImpl<$Res>
    implements $PreviewStartedEventCopyWith<$Res> {
  _$PreviewStartedEventCopyWithImpl(this._self, this._then);

  final PreviewStartedEvent _self;
  final $Res Function(PreviewStartedEvent) _then;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? themeId = null,}) {
  return _then(PreviewStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,null == themeId ? _self.themeId : themeId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewThemeChangedEvent implements PreviewEvent {
  const PreviewThemeChangedEvent(this.themeId);
  

 final  String themeId;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewThemeChangedEventCopyWith<PreviewThemeChangedEvent> get copyWith => _$PreviewThemeChangedEventCopyWithImpl<PreviewThemeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewThemeChangedEvent&&(identical(other.themeId, themeId) || other.themeId == themeId));
}


@override
int get hashCode => Object.hash(runtimeType,themeId);

@override
String toString() {
  return 'PreviewEvent.themeChanged(themeId: $themeId)';
}


}

/// @nodoc
abstract mixin class $PreviewThemeChangedEventCopyWith<$Res> implements $PreviewEventCopyWith<$Res> {
  factory $PreviewThemeChangedEventCopyWith(PreviewThemeChangedEvent value, $Res Function(PreviewThemeChangedEvent) _then) = _$PreviewThemeChangedEventCopyWithImpl;
@useResult
$Res call({
 String themeId
});




}
/// @nodoc
class _$PreviewThemeChangedEventCopyWithImpl<$Res>
    implements $PreviewThemeChangedEventCopyWith<$Res> {
  _$PreviewThemeChangedEventCopyWithImpl(this._self, this._then);

  final PreviewThemeChangedEvent _self;
  final $Res Function(PreviewThemeChangedEvent) _then;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? themeId = null,}) {
  return _then(PreviewThemeChangedEvent(
null == themeId ? _self.themeId : themeId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewChangesReceivedEvent implements PreviewEvent {
  const PreviewChangesReceivedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewChangesReceivedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewEvent.changesReceived()';
}


}




/// @nodoc


class PreviewLinkTappedEvent implements PreviewEvent {
  const PreviewLinkTappedEvent(this.url);
  

 final  String url;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewLinkTappedEventCopyWith<PreviewLinkTappedEvent> get copyWith => _$PreviewLinkTappedEventCopyWithImpl<PreviewLinkTappedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLinkTappedEvent&&(identical(other.url, url) || other.url == url));
}


@override
int get hashCode => Object.hash(runtimeType,url);

@override
String toString() {
  return 'PreviewEvent.linkTapped(url: $url)';
}


}

/// @nodoc
abstract mixin class $PreviewLinkTappedEventCopyWith<$Res> implements $PreviewEventCopyWith<$Res> {
  factory $PreviewLinkTappedEventCopyWith(PreviewLinkTappedEvent value, $Res Function(PreviewLinkTappedEvent) _then) = _$PreviewLinkTappedEventCopyWithImpl;
@useResult
$Res call({
 String url
});




}
/// @nodoc
class _$PreviewLinkTappedEventCopyWithImpl<$Res>
    implements $PreviewLinkTappedEventCopyWith<$Res> {
  _$PreviewLinkTappedEventCopyWithImpl(this._self, this._then);

  final PreviewLinkTappedEvent _self;
  final $Res Function(PreviewLinkTappedEvent) _then;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? url = null,}) {
  return _then(PreviewLinkTappedEvent(
null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewActionHandledEvent implements PreviewEvent {
  const PreviewActionHandledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewActionHandledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewEvent.actionHandled()';
}


}




/// @nodoc


class PreviewNoticeDismissedEvent implements PreviewEvent {
  const PreviewNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewEvent.noticeDismissed()';
}


}




/// @nodoc
mixin _$PreviewState {

/// The note drawn.
 String get noteId;/// The built-in theme it is drawn in.
 String get themeId;/// The page, once it is drawn; the last one stays while it is drawn
/// again.
 String? get html;/// The revision of the note the page was drawn from: a tick is made
/// against it.
 String get rev;/// Why the page could not be drawn, when it couldn't.
 ApplicationError? get error;/// What the screen is to do for the last tap, until it says it has.
 PreviewAction? get action;/// Why the last tap did nothing, until it is dismissed.
 ApplicationError? get notice;
/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewStateCopyWith<PreviewState> get copyWith => _$PreviewStateCopyWithImpl<PreviewState>(this as PreviewState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.themeId, themeId) || other.themeId == themeId)&&(identical(other.html, html) || other.html == html)&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.error, error) || other.error == error)&&(identical(other.action, action) || other.action == action)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,themeId,html,rev,error,action,notice);

@override
String toString() {
  return 'PreviewState(noteId: $noteId, themeId: $themeId, html: $html, rev: $rev, error: $error, action: $action, notice: $notice)';
}


}

/// @nodoc
abstract mixin class $PreviewStateCopyWith<$Res>  {
  factory $PreviewStateCopyWith(PreviewState value, $Res Function(PreviewState) _then) = _$PreviewStateCopyWithImpl;
@useResult
$Res call({
 String noteId, String themeId, String? html, String rev, ApplicationError? error, PreviewAction? action, ApplicationError? notice
});


$ApplicationErrorCopyWith<$Res>? get error;$PreviewActionCopyWith<$Res>? get action;$ApplicationErrorCopyWith<$Res>? get notice;

}
/// @nodoc
class _$PreviewStateCopyWithImpl<$Res>
    implements $PreviewStateCopyWith<$Res> {
  _$PreviewStateCopyWithImpl(this._self, this._then);

  final PreviewState _self;
  final $Res Function(PreviewState) _then;

/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? themeId = null,Object? html = freezed,Object? rev = null,Object? error = freezed,Object? action = freezed,Object? notice = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,themeId: null == themeId ? _self.themeId : themeId // ignore: cast_nullable_to_non_nullable
as String,html: freezed == html ? _self.html : html // ignore: cast_nullable_to_non_nullable
as String?,rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as PreviewAction?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of PreviewState
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
}/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PreviewActionCopyWith<$Res>? get action {
    if (_self.action == null) {
    return null;
  }

  return $PreviewActionCopyWith<$Res>(_self.action!, (value) {
    return _then(_self.copyWith(action: value));
  });
}/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}
}


/// Adds pattern-matching-related methods to [PreviewState].
extension PreviewStatePatterns on PreviewState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PreviewState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PreviewState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PreviewState value)  $default,){
final _that = this;
switch (_that) {
case _PreviewState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PreviewState value)?  $default,){
final _that = this;
switch (_that) {
case _PreviewState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  String themeId,  String? html,  String rev,  ApplicationError? error,  PreviewAction? action,  ApplicationError? notice)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PreviewState() when $default != null:
return $default(_that.noteId,_that.themeId,_that.html,_that.rev,_that.error,_that.action,_that.notice);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  String themeId,  String? html,  String rev,  ApplicationError? error,  PreviewAction? action,  ApplicationError? notice)  $default,) {final _that = this;
switch (_that) {
case _PreviewState():
return $default(_that.noteId,_that.themeId,_that.html,_that.rev,_that.error,_that.action,_that.notice);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  String themeId,  String? html,  String rev,  ApplicationError? error,  PreviewAction? action,  ApplicationError? notice)?  $default,) {final _that = this;
switch (_that) {
case _PreviewState() when $default != null:
return $default(_that.noteId,_that.themeId,_that.html,_that.rev,_that.error,_that.action,_that.notice);case _:
  return null;

}
}

}

/// @nodoc


class _PreviewState implements PreviewState {
  const _PreviewState({this.noteId = '', this.themeId = '', this.html, this.rev = '', this.error, this.action, this.notice});
  

/// The note drawn.
@override@JsonKey() final  String noteId;
/// The built-in theme it is drawn in.
@override@JsonKey() final  String themeId;
/// The page, once it is drawn; the last one stays while it is drawn
/// again.
@override final  String? html;
/// The revision of the note the page was drawn from: a tick is made
/// against it.
@override@JsonKey() final  String rev;
/// Why the page could not be drawn, when it couldn't.
@override final  ApplicationError? error;
/// What the screen is to do for the last tap, until it says it has.
@override final  PreviewAction? action;
/// Why the last tap did nothing, until it is dismissed.
@override final  ApplicationError? notice;

/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PreviewStateCopyWith<_PreviewState> get copyWith => __$PreviewStateCopyWithImpl<_PreviewState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PreviewState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.themeId, themeId) || other.themeId == themeId)&&(identical(other.html, html) || other.html == html)&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.error, error) || other.error == error)&&(identical(other.action, action) || other.action == action)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,themeId,html,rev,error,action,notice);

@override
String toString() {
  return 'PreviewState(noteId: $noteId, themeId: $themeId, html: $html, rev: $rev, error: $error, action: $action, notice: $notice)';
}


}

/// @nodoc
abstract mixin class _$PreviewStateCopyWith<$Res> implements $PreviewStateCopyWith<$Res> {
  factory _$PreviewStateCopyWith(_PreviewState value, $Res Function(_PreviewState) _then) = __$PreviewStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, String themeId, String? html, String rev, ApplicationError? error, PreviewAction? action, ApplicationError? notice
});


@override $ApplicationErrorCopyWith<$Res>? get error;@override $PreviewActionCopyWith<$Res>? get action;@override $ApplicationErrorCopyWith<$Res>? get notice;

}
/// @nodoc
class __$PreviewStateCopyWithImpl<$Res>
    implements _$PreviewStateCopyWith<$Res> {
  __$PreviewStateCopyWithImpl(this._self, this._then);

  final _PreviewState _self;
  final $Res Function(_PreviewState) _then;

/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? themeId = null,Object? html = freezed,Object? rev = null,Object? error = freezed,Object? action = freezed,Object? notice = freezed,}) {
  return _then(_PreviewState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,themeId: null == themeId ? _self.themeId : themeId // ignore: cast_nullable_to_non_nullable
as String,html: freezed == html ? _self.html : html // ignore: cast_nullable_to_non_nullable
as String?,rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as PreviewAction?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of PreviewState
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
}/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PreviewActionCopyWith<$Res>? get action {
    if (_self.action == null) {
    return null;
  }

  return $PreviewActionCopyWith<$Res>(_self.action!, (value) {
    return _then(_self.copyWith(action: value));
  });
}/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}
}

// dart format on
