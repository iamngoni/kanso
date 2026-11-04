// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'send_feedback_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SendFeedbackEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SendFeedbackEvent()';
}


}

/// @nodoc
class $SendFeedbackEventCopyWith<$Res>  {
$SendFeedbackEventCopyWith(SendFeedbackEvent _, $Res Function(SendFeedbackEvent) __);
}


/// Adds pattern-matching-related methods to [SendFeedbackEvent].
extension SendFeedbackEventPatterns on SendFeedbackEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SendFeedbackStartedEvent value)?  started,TResult Function( SendFeedbackKindChangedEvent value)?  kindChanged,TResult Function( SendFeedbackTextChangedEvent value)?  textChanged,TResult Function( SendFeedbackCreditToggledEvent value)?  creditToggled,TResult Function( SendFeedbackCreditNameChangedEvent value)?  creditNameChanged,TResult Function( SendFeedbackAppInfoToggledEvent value)?  appInfoToggled,TResult Function( SendFeedbackScreenshotRequestedEvent value)?  screenshotRequested,TResult Function( SendFeedbackScreenshotRemovedEvent value)?  screenshotRemoved,TResult Function( SendFeedbackSentToggledEvent value)?  sentToggled,TResult Function( SendFeedbackSendRequestedEvent value)?  sendRequested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SendFeedbackStartedEvent() when started != null:
return started(_that);case SendFeedbackKindChangedEvent() when kindChanged != null:
return kindChanged(_that);case SendFeedbackTextChangedEvent() when textChanged != null:
return textChanged(_that);case SendFeedbackCreditToggledEvent() when creditToggled != null:
return creditToggled(_that);case SendFeedbackCreditNameChangedEvent() when creditNameChanged != null:
return creditNameChanged(_that);case SendFeedbackAppInfoToggledEvent() when appInfoToggled != null:
return appInfoToggled(_that);case SendFeedbackScreenshotRequestedEvent() when screenshotRequested != null:
return screenshotRequested(_that);case SendFeedbackScreenshotRemovedEvent() when screenshotRemoved != null:
return screenshotRemoved(_that);case SendFeedbackSentToggledEvent() when sentToggled != null:
return sentToggled(_that);case SendFeedbackSendRequestedEvent() when sendRequested != null:
return sendRequested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SendFeedbackStartedEvent value)  started,required TResult Function( SendFeedbackKindChangedEvent value)  kindChanged,required TResult Function( SendFeedbackTextChangedEvent value)  textChanged,required TResult Function( SendFeedbackCreditToggledEvent value)  creditToggled,required TResult Function( SendFeedbackCreditNameChangedEvent value)  creditNameChanged,required TResult Function( SendFeedbackAppInfoToggledEvent value)  appInfoToggled,required TResult Function( SendFeedbackScreenshotRequestedEvent value)  screenshotRequested,required TResult Function( SendFeedbackScreenshotRemovedEvent value)  screenshotRemoved,required TResult Function( SendFeedbackSentToggledEvent value)  sentToggled,required TResult Function( SendFeedbackSendRequestedEvent value)  sendRequested,}){
final _that = this;
switch (_that) {
case SendFeedbackStartedEvent():
return started(_that);case SendFeedbackKindChangedEvent():
return kindChanged(_that);case SendFeedbackTextChangedEvent():
return textChanged(_that);case SendFeedbackCreditToggledEvent():
return creditToggled(_that);case SendFeedbackCreditNameChangedEvent():
return creditNameChanged(_that);case SendFeedbackAppInfoToggledEvent():
return appInfoToggled(_that);case SendFeedbackScreenshotRequestedEvent():
return screenshotRequested(_that);case SendFeedbackScreenshotRemovedEvent():
return screenshotRemoved(_that);case SendFeedbackSentToggledEvent():
return sentToggled(_that);case SendFeedbackSendRequestedEvent():
return sendRequested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SendFeedbackStartedEvent value)?  started,TResult? Function( SendFeedbackKindChangedEvent value)?  kindChanged,TResult? Function( SendFeedbackTextChangedEvent value)?  textChanged,TResult? Function( SendFeedbackCreditToggledEvent value)?  creditToggled,TResult? Function( SendFeedbackCreditNameChangedEvent value)?  creditNameChanged,TResult? Function( SendFeedbackAppInfoToggledEvent value)?  appInfoToggled,TResult? Function( SendFeedbackScreenshotRequestedEvent value)?  screenshotRequested,TResult? Function( SendFeedbackScreenshotRemovedEvent value)?  screenshotRemoved,TResult? Function( SendFeedbackSentToggledEvent value)?  sentToggled,TResult? Function( SendFeedbackSendRequestedEvent value)?  sendRequested,}){
final _that = this;
switch (_that) {
case SendFeedbackStartedEvent() when started != null:
return started(_that);case SendFeedbackKindChangedEvent() when kindChanged != null:
return kindChanged(_that);case SendFeedbackTextChangedEvent() when textChanged != null:
return textChanged(_that);case SendFeedbackCreditToggledEvent() when creditToggled != null:
return creditToggled(_that);case SendFeedbackCreditNameChangedEvent() when creditNameChanged != null:
return creditNameChanged(_that);case SendFeedbackAppInfoToggledEvent() when appInfoToggled != null:
return appInfoToggled(_that);case SendFeedbackScreenshotRequestedEvent() when screenshotRequested != null:
return screenshotRequested(_that);case SendFeedbackScreenshotRemovedEvent() when screenshotRemoved != null:
return screenshotRemoved(_that);case SendFeedbackSentToggledEvent() when sentToggled != null:
return sentToggled(_that);case SendFeedbackSendRequestedEvent() when sendRequested != null:
return sendRequested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String screen)?  started,TResult Function( FeedbackKind kind)?  kindChanged,TResult Function( String text)?  textChanged,TResult Function( bool on)?  creditToggled,TResult Function( String name)?  creditNameChanged,TResult Function( bool on)?  appInfoToggled,TResult Function()?  screenshotRequested,TResult Function()?  screenshotRemoved,TResult Function()?  sentToggled,TResult Function()?  sendRequested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SendFeedbackStartedEvent() when started != null:
return started(_that.screen);case SendFeedbackKindChangedEvent() when kindChanged != null:
return kindChanged(_that.kind);case SendFeedbackTextChangedEvent() when textChanged != null:
return textChanged(_that.text);case SendFeedbackCreditToggledEvent() when creditToggled != null:
return creditToggled(_that.on);case SendFeedbackCreditNameChangedEvent() when creditNameChanged != null:
return creditNameChanged(_that.name);case SendFeedbackAppInfoToggledEvent() when appInfoToggled != null:
return appInfoToggled(_that.on);case SendFeedbackScreenshotRequestedEvent() when screenshotRequested != null:
return screenshotRequested();case SendFeedbackScreenshotRemovedEvent() when screenshotRemoved != null:
return screenshotRemoved();case SendFeedbackSentToggledEvent() when sentToggled != null:
return sentToggled();case SendFeedbackSendRequestedEvent() when sendRequested != null:
return sendRequested();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String screen)  started,required TResult Function( FeedbackKind kind)  kindChanged,required TResult Function( String text)  textChanged,required TResult Function( bool on)  creditToggled,required TResult Function( String name)  creditNameChanged,required TResult Function( bool on)  appInfoToggled,required TResult Function()  screenshotRequested,required TResult Function()  screenshotRemoved,required TResult Function()  sentToggled,required TResult Function()  sendRequested,}) {final _that = this;
switch (_that) {
case SendFeedbackStartedEvent():
return started(_that.screen);case SendFeedbackKindChangedEvent():
return kindChanged(_that.kind);case SendFeedbackTextChangedEvent():
return textChanged(_that.text);case SendFeedbackCreditToggledEvent():
return creditToggled(_that.on);case SendFeedbackCreditNameChangedEvent():
return creditNameChanged(_that.name);case SendFeedbackAppInfoToggledEvent():
return appInfoToggled(_that.on);case SendFeedbackScreenshotRequestedEvent():
return screenshotRequested();case SendFeedbackScreenshotRemovedEvent():
return screenshotRemoved();case SendFeedbackSentToggledEvent():
return sentToggled();case SendFeedbackSendRequestedEvent():
return sendRequested();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String screen)?  started,TResult? Function( FeedbackKind kind)?  kindChanged,TResult? Function( String text)?  textChanged,TResult? Function( bool on)?  creditToggled,TResult? Function( String name)?  creditNameChanged,TResult? Function( bool on)?  appInfoToggled,TResult? Function()?  screenshotRequested,TResult? Function()?  screenshotRemoved,TResult? Function()?  sentToggled,TResult? Function()?  sendRequested,}) {final _that = this;
switch (_that) {
case SendFeedbackStartedEvent() when started != null:
return started(_that.screen);case SendFeedbackKindChangedEvent() when kindChanged != null:
return kindChanged(_that.kind);case SendFeedbackTextChangedEvent() when textChanged != null:
return textChanged(_that.text);case SendFeedbackCreditToggledEvent() when creditToggled != null:
return creditToggled(_that.on);case SendFeedbackCreditNameChangedEvent() when creditNameChanged != null:
return creditNameChanged(_that.name);case SendFeedbackAppInfoToggledEvent() when appInfoToggled != null:
return appInfoToggled(_that.on);case SendFeedbackScreenshotRequestedEvent() when screenshotRequested != null:
return screenshotRequested();case SendFeedbackScreenshotRemovedEvent() when screenshotRemoved != null:
return screenshotRemoved();case SendFeedbackSentToggledEvent() when sentToggled != null:
return sentToggled();case SendFeedbackSendRequestedEvent() when sendRequested != null:
return sendRequested();case _:
  return null;

}
}

}

/// @nodoc


class SendFeedbackStartedEvent implements SendFeedbackEvent {
  const SendFeedbackStartedEvent({this.screen = 'settings'});
  

@JsonKey() final  String screen;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SendFeedbackStartedEventCopyWith<SendFeedbackStartedEvent> get copyWith => _$SendFeedbackStartedEventCopyWithImpl<SendFeedbackStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackStartedEvent&&(identical(other.screen, screen) || other.screen == screen));
}


@override
int get hashCode => Object.hash(runtimeType,screen);

@override
String toString() {
  return 'SendFeedbackEvent.started(screen: $screen)';
}


}

/// @nodoc
abstract mixin class $SendFeedbackStartedEventCopyWith<$Res> implements $SendFeedbackEventCopyWith<$Res> {
  factory $SendFeedbackStartedEventCopyWith(SendFeedbackStartedEvent value, $Res Function(SendFeedbackStartedEvent) _then) = _$SendFeedbackStartedEventCopyWithImpl;
@useResult
$Res call({
 String screen
});




}
/// @nodoc
class _$SendFeedbackStartedEventCopyWithImpl<$Res>
    implements $SendFeedbackStartedEventCopyWith<$Res> {
  _$SendFeedbackStartedEventCopyWithImpl(this._self, this._then);

  final SendFeedbackStartedEvent _self;
  final $Res Function(SendFeedbackStartedEvent) _then;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? screen = null,}) {
  return _then(SendFeedbackStartedEvent(
screen: null == screen ? _self.screen : screen // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SendFeedbackKindChangedEvent implements SendFeedbackEvent {
  const SendFeedbackKindChangedEvent(this.kind);
  

 final  FeedbackKind kind;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SendFeedbackKindChangedEventCopyWith<SendFeedbackKindChangedEvent> get copyWith => _$SendFeedbackKindChangedEventCopyWithImpl<SendFeedbackKindChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackKindChangedEvent&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,kind);

@override
String toString() {
  return 'SendFeedbackEvent.kindChanged(kind: $kind)';
}


}

/// @nodoc
abstract mixin class $SendFeedbackKindChangedEventCopyWith<$Res> implements $SendFeedbackEventCopyWith<$Res> {
  factory $SendFeedbackKindChangedEventCopyWith(SendFeedbackKindChangedEvent value, $Res Function(SendFeedbackKindChangedEvent) _then) = _$SendFeedbackKindChangedEventCopyWithImpl;
@useResult
$Res call({
 FeedbackKind kind
});




}
/// @nodoc
class _$SendFeedbackKindChangedEventCopyWithImpl<$Res>
    implements $SendFeedbackKindChangedEventCopyWith<$Res> {
  _$SendFeedbackKindChangedEventCopyWithImpl(this._self, this._then);

  final SendFeedbackKindChangedEvent _self;
  final $Res Function(SendFeedbackKindChangedEvent) _then;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? kind = null,}) {
  return _then(SendFeedbackKindChangedEvent(
null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as FeedbackKind,
  ));
}


}

/// @nodoc


class SendFeedbackTextChangedEvent implements SendFeedbackEvent {
  const SendFeedbackTextChangedEvent(this.text);
  

 final  String text;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SendFeedbackTextChangedEventCopyWith<SendFeedbackTextChangedEvent> get copyWith => _$SendFeedbackTextChangedEventCopyWithImpl<SendFeedbackTextChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackTextChangedEvent&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'SendFeedbackEvent.textChanged(text: $text)';
}


}

/// @nodoc
abstract mixin class $SendFeedbackTextChangedEventCopyWith<$Res> implements $SendFeedbackEventCopyWith<$Res> {
  factory $SendFeedbackTextChangedEventCopyWith(SendFeedbackTextChangedEvent value, $Res Function(SendFeedbackTextChangedEvent) _then) = _$SendFeedbackTextChangedEventCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$SendFeedbackTextChangedEventCopyWithImpl<$Res>
    implements $SendFeedbackTextChangedEventCopyWith<$Res> {
  _$SendFeedbackTextChangedEventCopyWithImpl(this._self, this._then);

  final SendFeedbackTextChangedEvent _self;
  final $Res Function(SendFeedbackTextChangedEvent) _then;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(SendFeedbackTextChangedEvent(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SendFeedbackCreditToggledEvent implements SendFeedbackEvent {
  const SendFeedbackCreditToggledEvent({required this.on});
  

 final  bool on;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SendFeedbackCreditToggledEventCopyWith<SendFeedbackCreditToggledEvent> get copyWith => _$SendFeedbackCreditToggledEventCopyWithImpl<SendFeedbackCreditToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackCreditToggledEvent&&(identical(other.on, on) || other.on == on));
}


@override
int get hashCode => Object.hash(runtimeType,on);

@override
String toString() {
  return 'SendFeedbackEvent.creditToggled(on: $on)';
}


}

/// @nodoc
abstract mixin class $SendFeedbackCreditToggledEventCopyWith<$Res> implements $SendFeedbackEventCopyWith<$Res> {
  factory $SendFeedbackCreditToggledEventCopyWith(SendFeedbackCreditToggledEvent value, $Res Function(SendFeedbackCreditToggledEvent) _then) = _$SendFeedbackCreditToggledEventCopyWithImpl;
@useResult
$Res call({
 bool on
});




}
/// @nodoc
class _$SendFeedbackCreditToggledEventCopyWithImpl<$Res>
    implements $SendFeedbackCreditToggledEventCopyWith<$Res> {
  _$SendFeedbackCreditToggledEventCopyWithImpl(this._self, this._then);

  final SendFeedbackCreditToggledEvent _self;
  final $Res Function(SendFeedbackCreditToggledEvent) _then;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? on = null,}) {
  return _then(SendFeedbackCreditToggledEvent(
on: null == on ? _self.on : on // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class SendFeedbackCreditNameChangedEvent implements SendFeedbackEvent {
  const SendFeedbackCreditNameChangedEvent(this.name);
  

 final  String name;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SendFeedbackCreditNameChangedEventCopyWith<SendFeedbackCreditNameChangedEvent> get copyWith => _$SendFeedbackCreditNameChangedEventCopyWithImpl<SendFeedbackCreditNameChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackCreditNameChangedEvent&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,name);

@override
String toString() {
  return 'SendFeedbackEvent.creditNameChanged(name: $name)';
}


}

/// @nodoc
abstract mixin class $SendFeedbackCreditNameChangedEventCopyWith<$Res> implements $SendFeedbackEventCopyWith<$Res> {
  factory $SendFeedbackCreditNameChangedEventCopyWith(SendFeedbackCreditNameChangedEvent value, $Res Function(SendFeedbackCreditNameChangedEvent) _then) = _$SendFeedbackCreditNameChangedEventCopyWithImpl;
@useResult
$Res call({
 String name
});




}
/// @nodoc
class _$SendFeedbackCreditNameChangedEventCopyWithImpl<$Res>
    implements $SendFeedbackCreditNameChangedEventCopyWith<$Res> {
  _$SendFeedbackCreditNameChangedEventCopyWithImpl(this._self, this._then);

  final SendFeedbackCreditNameChangedEvent _self;
  final $Res Function(SendFeedbackCreditNameChangedEvent) _then;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? name = null,}) {
  return _then(SendFeedbackCreditNameChangedEvent(
null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SendFeedbackAppInfoToggledEvent implements SendFeedbackEvent {
  const SendFeedbackAppInfoToggledEvent({required this.on});
  

 final  bool on;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SendFeedbackAppInfoToggledEventCopyWith<SendFeedbackAppInfoToggledEvent> get copyWith => _$SendFeedbackAppInfoToggledEventCopyWithImpl<SendFeedbackAppInfoToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackAppInfoToggledEvent&&(identical(other.on, on) || other.on == on));
}


@override
int get hashCode => Object.hash(runtimeType,on);

@override
String toString() {
  return 'SendFeedbackEvent.appInfoToggled(on: $on)';
}


}

/// @nodoc
abstract mixin class $SendFeedbackAppInfoToggledEventCopyWith<$Res> implements $SendFeedbackEventCopyWith<$Res> {
  factory $SendFeedbackAppInfoToggledEventCopyWith(SendFeedbackAppInfoToggledEvent value, $Res Function(SendFeedbackAppInfoToggledEvent) _then) = _$SendFeedbackAppInfoToggledEventCopyWithImpl;
@useResult
$Res call({
 bool on
});




}
/// @nodoc
class _$SendFeedbackAppInfoToggledEventCopyWithImpl<$Res>
    implements $SendFeedbackAppInfoToggledEventCopyWith<$Res> {
  _$SendFeedbackAppInfoToggledEventCopyWithImpl(this._self, this._then);

  final SendFeedbackAppInfoToggledEvent _self;
  final $Res Function(SendFeedbackAppInfoToggledEvent) _then;

/// Create a copy of SendFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? on = null,}) {
  return _then(SendFeedbackAppInfoToggledEvent(
on: null == on ? _self.on : on // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class SendFeedbackScreenshotRequestedEvent implements SendFeedbackEvent {
  const SendFeedbackScreenshotRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackScreenshotRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SendFeedbackEvent.screenshotRequested()';
}


}




/// @nodoc


class SendFeedbackScreenshotRemovedEvent implements SendFeedbackEvent {
  const SendFeedbackScreenshotRemovedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackScreenshotRemovedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SendFeedbackEvent.screenshotRemoved()';
}


}




/// @nodoc


class SendFeedbackSentToggledEvent implements SendFeedbackEvent {
  const SendFeedbackSentToggledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackSentToggledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SendFeedbackEvent.sentToggled()';
}


}




/// @nodoc


class SendFeedbackSendRequestedEvent implements SendFeedbackEvent {
  const SendFeedbackSendRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackSendRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SendFeedbackEvent.sendRequested()';
}


}




/// @nodoc
mixin _$SendFeedbackState {

 FeedbackDraft get draft;/// The fixed screen the person came from.
 String get screen;/// Whether Show what's sent is open.
 bool get showSent;/// What a send would carry, while Show what's sent is open.
 FeedbackPayload get payload;/// The report is being saved and sent.
 bool get sending;/// The photo that was picked is over 5 MB.
 bool get screenshotTooBig;/// How the last send ended; null before one. A report that was sent or
/// saved closes the page; a refusal stays on it.
 FeedbackSendResult? get result;
/// Create a copy of SendFeedbackState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SendFeedbackStateCopyWith<SendFeedbackState> get copyWith => _$SendFeedbackStateCopyWithImpl<SendFeedbackState>(this as SendFeedbackState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SendFeedbackState&&(identical(other.draft, draft) || other.draft == draft)&&(identical(other.screen, screen) || other.screen == screen)&&(identical(other.showSent, showSent) || other.showSent == showSent)&&(identical(other.payload, payload) || other.payload == payload)&&(identical(other.sending, sending) || other.sending == sending)&&(identical(other.screenshotTooBig, screenshotTooBig) || other.screenshotTooBig == screenshotTooBig)&&(identical(other.result, result) || other.result == result));
}


@override
int get hashCode => Object.hash(runtimeType,draft,screen,showSent,payload,sending,screenshotTooBig,result);

@override
String toString() {
  return 'SendFeedbackState(draft: $draft, screen: $screen, showSent: $showSent, payload: $payload, sending: $sending, screenshotTooBig: $screenshotTooBig, result: $result)';
}


}

/// @nodoc
abstract mixin class $SendFeedbackStateCopyWith<$Res>  {
  factory $SendFeedbackStateCopyWith(SendFeedbackState value, $Res Function(SendFeedbackState) _then) = _$SendFeedbackStateCopyWithImpl;
@useResult
$Res call({
 FeedbackDraft draft, String screen, bool showSent, FeedbackPayload payload, bool sending, bool screenshotTooBig, FeedbackSendResult? result
});


$FeedbackDraftCopyWith<$Res> get draft;$FeedbackPayloadCopyWith<$Res> get payload;$FeedbackSendResultCopyWith<$Res>? get result;

}
/// @nodoc
class _$SendFeedbackStateCopyWithImpl<$Res>
    implements $SendFeedbackStateCopyWith<$Res> {
  _$SendFeedbackStateCopyWithImpl(this._self, this._then);

  final SendFeedbackState _self;
  final $Res Function(SendFeedbackState) _then;

/// Create a copy of SendFeedbackState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? draft = null,Object? screen = null,Object? showSent = null,Object? payload = null,Object? sending = null,Object? screenshotTooBig = null,Object? result = freezed,}) {
  return _then(_self.copyWith(
draft: null == draft ? _self.draft : draft // ignore: cast_nullable_to_non_nullable
as FeedbackDraft,screen: null == screen ? _self.screen : screen // ignore: cast_nullable_to_non_nullable
as String,showSent: null == showSent ? _self.showSent : showSent // ignore: cast_nullable_to_non_nullable
as bool,payload: null == payload ? _self.payload : payload // ignore: cast_nullable_to_non_nullable
as FeedbackPayload,sending: null == sending ? _self.sending : sending // ignore: cast_nullable_to_non_nullable
as bool,screenshotTooBig: null == screenshotTooBig ? _self.screenshotTooBig : screenshotTooBig // ignore: cast_nullable_to_non_nullable
as bool,result: freezed == result ? _self.result : result // ignore: cast_nullable_to_non_nullable
as FeedbackSendResult?,
  ));
}
/// Create a copy of SendFeedbackState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$FeedbackDraftCopyWith<$Res> get draft {
  
  return $FeedbackDraftCopyWith<$Res>(_self.draft, (value) {
    return _then(_self.copyWith(draft: value));
  });
}/// Create a copy of SendFeedbackState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$FeedbackPayloadCopyWith<$Res> get payload {
  
  return $FeedbackPayloadCopyWith<$Res>(_self.payload, (value) {
    return _then(_self.copyWith(payload: value));
  });
}/// Create a copy of SendFeedbackState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$FeedbackSendResultCopyWith<$Res>? get result {
    if (_self.result == null) {
    return null;
  }

  return $FeedbackSendResultCopyWith<$Res>(_self.result!, (value) {
    return _then(_self.copyWith(result: value));
  });
}
}


/// Adds pattern-matching-related methods to [SendFeedbackState].
extension SendFeedbackStatePatterns on SendFeedbackState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SendFeedbackState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SendFeedbackState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SendFeedbackState value)  $default,){
final _that = this;
switch (_that) {
case _SendFeedbackState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SendFeedbackState value)?  $default,){
final _that = this;
switch (_that) {
case _SendFeedbackState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( FeedbackDraft draft,  String screen,  bool showSent,  FeedbackPayload payload,  bool sending,  bool screenshotTooBig,  FeedbackSendResult? result)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SendFeedbackState() when $default != null:
return $default(_that.draft,_that.screen,_that.showSent,_that.payload,_that.sending,_that.screenshotTooBig,_that.result);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( FeedbackDraft draft,  String screen,  bool showSent,  FeedbackPayload payload,  bool sending,  bool screenshotTooBig,  FeedbackSendResult? result)  $default,) {final _that = this;
switch (_that) {
case _SendFeedbackState():
return $default(_that.draft,_that.screen,_that.showSent,_that.payload,_that.sending,_that.screenshotTooBig,_that.result);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( FeedbackDraft draft,  String screen,  bool showSent,  FeedbackPayload payload,  bool sending,  bool screenshotTooBig,  FeedbackSendResult? result)?  $default,) {final _that = this;
switch (_that) {
case _SendFeedbackState() when $default != null:
return $default(_that.draft,_that.screen,_that.showSent,_that.payload,_that.sending,_that.screenshotTooBig,_that.result);case _:
  return null;

}
}

}

/// @nodoc


class _SendFeedbackState implements SendFeedbackState {
  const _SendFeedbackState({this.draft = const FeedbackDraft(), this.screen = 'settings', this.showSent = false, this.payload = const FeedbackPayload(), this.sending = false, this.screenshotTooBig = false, this.result});
  

@override@JsonKey() final  FeedbackDraft draft;
/// The fixed screen the person came from.
@override@JsonKey() final  String screen;
/// Whether Show what's sent is open.
@override@JsonKey() final  bool showSent;
/// What a send would carry, while Show what's sent is open.
@override@JsonKey() final  FeedbackPayload payload;
/// The report is being saved and sent.
@override@JsonKey() final  bool sending;
/// The photo that was picked is over 5 MB.
@override@JsonKey() final  bool screenshotTooBig;
/// How the last send ended; null before one. A report that was sent or
/// saved closes the page; a refusal stays on it.
@override final  FeedbackSendResult? result;

/// Create a copy of SendFeedbackState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SendFeedbackStateCopyWith<_SendFeedbackState> get copyWith => __$SendFeedbackStateCopyWithImpl<_SendFeedbackState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SendFeedbackState&&(identical(other.draft, draft) || other.draft == draft)&&(identical(other.screen, screen) || other.screen == screen)&&(identical(other.showSent, showSent) || other.showSent == showSent)&&(identical(other.payload, payload) || other.payload == payload)&&(identical(other.sending, sending) || other.sending == sending)&&(identical(other.screenshotTooBig, screenshotTooBig) || other.screenshotTooBig == screenshotTooBig)&&(identical(other.result, result) || other.result == result));
}


@override
int get hashCode => Object.hash(runtimeType,draft,screen,showSent,payload,sending,screenshotTooBig,result);

@override
String toString() {
  return 'SendFeedbackState(draft: $draft, screen: $screen, showSent: $showSent, payload: $payload, sending: $sending, screenshotTooBig: $screenshotTooBig, result: $result)';
}


}

/// @nodoc
abstract mixin class _$SendFeedbackStateCopyWith<$Res> implements $SendFeedbackStateCopyWith<$Res> {
  factory _$SendFeedbackStateCopyWith(_SendFeedbackState value, $Res Function(_SendFeedbackState) _then) = __$SendFeedbackStateCopyWithImpl;
@override @useResult
$Res call({
 FeedbackDraft draft, String screen, bool showSent, FeedbackPayload payload, bool sending, bool screenshotTooBig, FeedbackSendResult? result
});


@override $FeedbackDraftCopyWith<$Res> get draft;@override $FeedbackPayloadCopyWith<$Res> get payload;@override $FeedbackSendResultCopyWith<$Res>? get result;

}
/// @nodoc
class __$SendFeedbackStateCopyWithImpl<$Res>
    implements _$SendFeedbackStateCopyWith<$Res> {
  __$SendFeedbackStateCopyWithImpl(this._self, this._then);

  final _SendFeedbackState _self;
  final $Res Function(_SendFeedbackState) _then;

/// Create a copy of SendFeedbackState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? draft = null,Object? screen = null,Object? showSent = null,Object? payload = null,Object? sending = null,Object? screenshotTooBig = null,Object? result = freezed,}) {
  return _then(_SendFeedbackState(
draft: null == draft ? _self.draft : draft // ignore: cast_nullable_to_non_nullable
as FeedbackDraft,screen: null == screen ? _self.screen : screen // ignore: cast_nullable_to_non_nullable
as String,showSent: null == showSent ? _self.showSent : showSent // ignore: cast_nullable_to_non_nullable
as bool,payload: null == payload ? _self.payload : payload // ignore: cast_nullable_to_non_nullable
as FeedbackPayload,sending: null == sending ? _self.sending : sending // ignore: cast_nullable_to_non_nullable
as bool,screenshotTooBig: null == screenshotTooBig ? _self.screenshotTooBig : screenshotTooBig // ignore: cast_nullable_to_non_nullable
as bool,result: freezed == result ? _self.result : result // ignore: cast_nullable_to_non_nullable
as FeedbackSendResult?,
  ));
}

/// Create a copy of SendFeedbackState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$FeedbackDraftCopyWith<$Res> get draft {
  
  return $FeedbackDraftCopyWith<$Res>(_self.draft, (value) {
    return _then(_self.copyWith(draft: value));
  });
}/// Create a copy of SendFeedbackState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$FeedbackPayloadCopyWith<$Res> get payload {
  
  return $FeedbackPayloadCopyWith<$Res>(_self.payload, (value) {
    return _then(_self.copyWith(payload: value));
  });
}/// Create a copy of SendFeedbackState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$FeedbackSendResultCopyWith<$Res>? get result {
    if (_self.result == null) {
    return null;
  }

  return $FeedbackSendResultCopyWith<$Res>(_self.result!, (value) {
    return _then(_self.copyWith(result: value));
  });
}
}

// dart format on
