// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'editor_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$EditorEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent()';
}


}




/// Adds pattern-matching-related methods to [EditorEvent].
extension EditorEventPatterns on EditorEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( EditorStartedEvent value)?  started,TResult Function( EditorSignalReceivedEvent value)?  signalReceived,TResult Function( EditorEditedEvent value)?  edited,TResult Function( EditorCaretMovedEvent value)?  caretMoved,TResult Function( EditorCompletionDismissedEvent value)?  completionDismissed,TResult Function( EditorPropertiesToggledEvent value)?  propertiesToggled,TResult Function( EditorPropertySetEvent value)?  propertySet,TResult Function( EditorPropertyRemovedEvent value)?  propertyRemoved,TResult Function( EditorWordGoalChangedEvent value)?  wordGoalChanged,TResult Function( EditorKeepMineRequestedEvent value)?  keepMineRequested,TResult Function( EditorUseTheirsRequestedEvent value)?  useTheirsRequested,TResult Function( EditorFlushRequestedEvent value)?  flushRequested,TResult Function( EditorLifecycleChangedEvent value)?  lifecycleChanged,TResult Function( EditorLinkOpenRequestedEvent value)?  linkOpenRequested,TResult Function( EditorDestinationHandledEvent value)?  destinationHandled,TResult Function( EditorClosedEvent value)?  closed,TResult Function( EditorAttachRequestedEvent value)?  attachRequested,TResult Function( EditorAttachNoticeDismissedEvent value)?  attachNoticeDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case EditorStartedEvent() when started != null:
return started(_that);case EditorSignalReceivedEvent() when signalReceived != null:
return signalReceived(_that);case EditorEditedEvent() when edited != null:
return edited(_that);case EditorCaretMovedEvent() when caretMoved != null:
return caretMoved(_that);case EditorCompletionDismissedEvent() when completionDismissed != null:
return completionDismissed(_that);case EditorPropertiesToggledEvent() when propertiesToggled != null:
return propertiesToggled(_that);case EditorPropertySetEvent() when propertySet != null:
return propertySet(_that);case EditorPropertyRemovedEvent() when propertyRemoved != null:
return propertyRemoved(_that);case EditorWordGoalChangedEvent() when wordGoalChanged != null:
return wordGoalChanged(_that);case EditorKeepMineRequestedEvent() when keepMineRequested != null:
return keepMineRequested(_that);case EditorUseTheirsRequestedEvent() when useTheirsRequested != null:
return useTheirsRequested(_that);case EditorFlushRequestedEvent() when flushRequested != null:
return flushRequested(_that);case EditorLifecycleChangedEvent() when lifecycleChanged != null:
return lifecycleChanged(_that);case EditorLinkOpenRequestedEvent() when linkOpenRequested != null:
return linkOpenRequested(_that);case EditorDestinationHandledEvent() when destinationHandled != null:
return destinationHandled(_that);case EditorClosedEvent() when closed != null:
return closed(_that);case EditorAttachRequestedEvent() when attachRequested != null:
return attachRequested(_that);case EditorAttachNoticeDismissedEvent() when attachNoticeDismissed != null:
return attachNoticeDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( EditorStartedEvent value)  started,required TResult Function( EditorSignalReceivedEvent value)  signalReceived,required TResult Function( EditorEditedEvent value)  edited,required TResult Function( EditorCaretMovedEvent value)  caretMoved,required TResult Function( EditorCompletionDismissedEvent value)  completionDismissed,required TResult Function( EditorPropertiesToggledEvent value)  propertiesToggled,required TResult Function( EditorPropertySetEvent value)  propertySet,required TResult Function( EditorPropertyRemovedEvent value)  propertyRemoved,required TResult Function( EditorWordGoalChangedEvent value)  wordGoalChanged,required TResult Function( EditorKeepMineRequestedEvent value)  keepMineRequested,required TResult Function( EditorUseTheirsRequestedEvent value)  useTheirsRequested,required TResult Function( EditorFlushRequestedEvent value)  flushRequested,required TResult Function( EditorLifecycleChangedEvent value)  lifecycleChanged,required TResult Function( EditorLinkOpenRequestedEvent value)  linkOpenRequested,required TResult Function( EditorDestinationHandledEvent value)  destinationHandled,required TResult Function( EditorClosedEvent value)  closed,required TResult Function( EditorAttachRequestedEvent value)  attachRequested,required TResult Function( EditorAttachNoticeDismissedEvent value)  attachNoticeDismissed,}){
final _that = this;
switch (_that) {
case EditorStartedEvent():
return started(_that);case EditorSignalReceivedEvent():
return signalReceived(_that);case EditorEditedEvent():
return edited(_that);case EditorCaretMovedEvent():
return caretMoved(_that);case EditorCompletionDismissedEvent():
return completionDismissed(_that);case EditorPropertiesToggledEvent():
return propertiesToggled(_that);case EditorPropertySetEvent():
return propertySet(_that);case EditorPropertyRemovedEvent():
return propertyRemoved(_that);case EditorWordGoalChangedEvent():
return wordGoalChanged(_that);case EditorKeepMineRequestedEvent():
return keepMineRequested(_that);case EditorUseTheirsRequestedEvent():
return useTheirsRequested(_that);case EditorFlushRequestedEvent():
return flushRequested(_that);case EditorLifecycleChangedEvent():
return lifecycleChanged(_that);case EditorLinkOpenRequestedEvent():
return linkOpenRequested(_that);case EditorDestinationHandledEvent():
return destinationHandled(_that);case EditorClosedEvent():
return closed(_that);case EditorAttachRequestedEvent():
return attachRequested(_that);case EditorAttachNoticeDismissedEvent():
return attachNoticeDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( EditorStartedEvent value)?  started,TResult? Function( EditorSignalReceivedEvent value)?  signalReceived,TResult? Function( EditorEditedEvent value)?  edited,TResult? Function( EditorCaretMovedEvent value)?  caretMoved,TResult? Function( EditorCompletionDismissedEvent value)?  completionDismissed,TResult? Function( EditorPropertiesToggledEvent value)?  propertiesToggled,TResult? Function( EditorPropertySetEvent value)?  propertySet,TResult? Function( EditorPropertyRemovedEvent value)?  propertyRemoved,TResult? Function( EditorWordGoalChangedEvent value)?  wordGoalChanged,TResult? Function( EditorKeepMineRequestedEvent value)?  keepMineRequested,TResult? Function( EditorUseTheirsRequestedEvent value)?  useTheirsRequested,TResult? Function( EditorFlushRequestedEvent value)?  flushRequested,TResult? Function( EditorLifecycleChangedEvent value)?  lifecycleChanged,TResult? Function( EditorLinkOpenRequestedEvent value)?  linkOpenRequested,TResult? Function( EditorDestinationHandledEvent value)?  destinationHandled,TResult? Function( EditorClosedEvent value)?  closed,TResult? Function( EditorAttachRequestedEvent value)?  attachRequested,TResult? Function( EditorAttachNoticeDismissedEvent value)?  attachNoticeDismissed,}){
final _that = this;
switch (_that) {
case EditorStartedEvent() when started != null:
return started(_that);case EditorSignalReceivedEvent() when signalReceived != null:
return signalReceived(_that);case EditorEditedEvent() when edited != null:
return edited(_that);case EditorCaretMovedEvent() when caretMoved != null:
return caretMoved(_that);case EditorCompletionDismissedEvent() when completionDismissed != null:
return completionDismissed(_that);case EditorPropertiesToggledEvent() when propertiesToggled != null:
return propertiesToggled(_that);case EditorPropertySetEvent() when propertySet != null:
return propertySet(_that);case EditorPropertyRemovedEvent() when propertyRemoved != null:
return propertyRemoved(_that);case EditorWordGoalChangedEvent() when wordGoalChanged != null:
return wordGoalChanged(_that);case EditorKeepMineRequestedEvent() when keepMineRequested != null:
return keepMineRequested(_that);case EditorUseTheirsRequestedEvent() when useTheirsRequested != null:
return useTheirsRequested(_that);case EditorFlushRequestedEvent() when flushRequested != null:
return flushRequested(_that);case EditorLifecycleChangedEvent() when lifecycleChanged != null:
return lifecycleChanged(_that);case EditorLinkOpenRequestedEvent() when linkOpenRequested != null:
return linkOpenRequested(_that);case EditorDestinationHandledEvent() when destinationHandled != null:
return destinationHandled(_that);case EditorClosedEvent() when closed != null:
return closed(_that);case EditorAttachRequestedEvent() when attachRequested != null:
return attachRequested(_that);case EditorAttachNoticeDismissedEvent() when attachNoticeDismissed != null:
return attachNoticeDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  started,TResult Function( EditorSignal signal)?  signalReceived,TResult Function()?  edited,TResult Function( EditorContext context)?  caretMoved,TResult Function()?  completionDismissed,TResult Function()?  propertiesToggled,TResult Function( String key,  String value,  int? index)?  propertySet,TResult Function( int index)?  propertyRemoved,TResult Function( core.GoalSetting setting)?  wordGoalChanged,TResult Function()?  keepMineRequested,TResult Function()?  useTheirsRequested,TResult Function( FlushReason reason)?  flushRequested,TResult Function( AppLifecycleState lifecycle)?  lifecycleChanged,TResult Function( LinkTarget target)?  linkOpenRequested,TResult Function()?  destinationHandled,TResult Function()?  closed,TResult Function( MediaSource source)?  attachRequested,TResult Function()?  attachNoticeDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case EditorStartedEvent() when started != null:
return started(_that.noteId);case EditorSignalReceivedEvent() when signalReceived != null:
return signalReceived(_that.signal);case EditorEditedEvent() when edited != null:
return edited();case EditorCaretMovedEvent() when caretMoved != null:
return caretMoved(_that.context);case EditorCompletionDismissedEvent() when completionDismissed != null:
return completionDismissed();case EditorPropertiesToggledEvent() when propertiesToggled != null:
return propertiesToggled();case EditorPropertySetEvent() when propertySet != null:
return propertySet(_that.key,_that.value,_that.index);case EditorPropertyRemovedEvent() when propertyRemoved != null:
return propertyRemoved(_that.index);case EditorWordGoalChangedEvent() when wordGoalChanged != null:
return wordGoalChanged(_that.setting);case EditorKeepMineRequestedEvent() when keepMineRequested != null:
return keepMineRequested();case EditorUseTheirsRequestedEvent() when useTheirsRequested != null:
return useTheirsRequested();case EditorFlushRequestedEvent() when flushRequested != null:
return flushRequested(_that.reason);case EditorLifecycleChangedEvent() when lifecycleChanged != null:
return lifecycleChanged(_that.lifecycle);case EditorLinkOpenRequestedEvent() when linkOpenRequested != null:
return linkOpenRequested(_that.target);case EditorDestinationHandledEvent() when destinationHandled != null:
return destinationHandled();case EditorClosedEvent() when closed != null:
return closed();case EditorAttachRequestedEvent() when attachRequested != null:
return attachRequested(_that.source);case EditorAttachNoticeDismissedEvent() when attachNoticeDismissed != null:
return attachNoticeDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  started,required TResult Function( EditorSignal signal)  signalReceived,required TResult Function()  edited,required TResult Function( EditorContext context)  caretMoved,required TResult Function()  completionDismissed,required TResult Function()  propertiesToggled,required TResult Function( String key,  String value,  int? index)  propertySet,required TResult Function( int index)  propertyRemoved,required TResult Function( core.GoalSetting setting)  wordGoalChanged,required TResult Function()  keepMineRequested,required TResult Function()  useTheirsRequested,required TResult Function( FlushReason reason)  flushRequested,required TResult Function( AppLifecycleState lifecycle)  lifecycleChanged,required TResult Function( LinkTarget target)  linkOpenRequested,required TResult Function()  destinationHandled,required TResult Function()  closed,required TResult Function( MediaSource source)  attachRequested,required TResult Function()  attachNoticeDismissed,}) {final _that = this;
switch (_that) {
case EditorStartedEvent():
return started(_that.noteId);case EditorSignalReceivedEvent():
return signalReceived(_that.signal);case EditorEditedEvent():
return edited();case EditorCaretMovedEvent():
return caretMoved(_that.context);case EditorCompletionDismissedEvent():
return completionDismissed();case EditorPropertiesToggledEvent():
return propertiesToggled();case EditorPropertySetEvent():
return propertySet(_that.key,_that.value,_that.index);case EditorPropertyRemovedEvent():
return propertyRemoved(_that.index);case EditorWordGoalChangedEvent():
return wordGoalChanged(_that.setting);case EditorKeepMineRequestedEvent():
return keepMineRequested();case EditorUseTheirsRequestedEvent():
return useTheirsRequested();case EditorFlushRequestedEvent():
return flushRequested(_that.reason);case EditorLifecycleChangedEvent():
return lifecycleChanged(_that.lifecycle);case EditorLinkOpenRequestedEvent():
return linkOpenRequested(_that.target);case EditorDestinationHandledEvent():
return destinationHandled();case EditorClosedEvent():
return closed();case EditorAttachRequestedEvent():
return attachRequested(_that.source);case EditorAttachNoticeDismissedEvent():
return attachNoticeDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  started,TResult? Function( EditorSignal signal)?  signalReceived,TResult? Function()?  edited,TResult? Function( EditorContext context)?  caretMoved,TResult? Function()?  completionDismissed,TResult? Function()?  propertiesToggled,TResult? Function( String key,  String value,  int? index)?  propertySet,TResult? Function( int index)?  propertyRemoved,TResult? Function( core.GoalSetting setting)?  wordGoalChanged,TResult? Function()?  keepMineRequested,TResult? Function()?  useTheirsRequested,TResult? Function( FlushReason reason)?  flushRequested,TResult? Function( AppLifecycleState lifecycle)?  lifecycleChanged,TResult? Function( LinkTarget target)?  linkOpenRequested,TResult? Function()?  destinationHandled,TResult? Function()?  closed,TResult? Function( MediaSource source)?  attachRequested,TResult? Function()?  attachNoticeDismissed,}) {final _that = this;
switch (_that) {
case EditorStartedEvent() when started != null:
return started(_that.noteId);case EditorSignalReceivedEvent() when signalReceived != null:
return signalReceived(_that.signal);case EditorEditedEvent() when edited != null:
return edited();case EditorCaretMovedEvent() when caretMoved != null:
return caretMoved(_that.context);case EditorCompletionDismissedEvent() when completionDismissed != null:
return completionDismissed();case EditorPropertiesToggledEvent() when propertiesToggled != null:
return propertiesToggled();case EditorPropertySetEvent() when propertySet != null:
return propertySet(_that.key,_that.value,_that.index);case EditorPropertyRemovedEvent() when propertyRemoved != null:
return propertyRemoved(_that.index);case EditorWordGoalChangedEvent() when wordGoalChanged != null:
return wordGoalChanged(_that.setting);case EditorKeepMineRequestedEvent() when keepMineRequested != null:
return keepMineRequested();case EditorUseTheirsRequestedEvent() when useTheirsRequested != null:
return useTheirsRequested();case EditorFlushRequestedEvent() when flushRequested != null:
return flushRequested(_that.reason);case EditorLifecycleChangedEvent() when lifecycleChanged != null:
return lifecycleChanged(_that.lifecycle);case EditorLinkOpenRequestedEvent() when linkOpenRequested != null:
return linkOpenRequested(_that.target);case EditorDestinationHandledEvent() when destinationHandled != null:
return destinationHandled();case EditorClosedEvent() when closed != null:
return closed();case EditorAttachRequestedEvent() when attachRequested != null:
return attachRequested(_that.source);case EditorAttachNoticeDismissedEvent() when attachNoticeDismissed != null:
return attachNoticeDismissed();case _:
  return null;

}
}

}

/// @nodoc


class EditorStartedEvent implements EditorEvent {
  const EditorStartedEvent(this.noteId);
  

 final  String noteId;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'EditorEvent.started(noteId: $noteId)';
}


}




/// @nodoc


class EditorSignalReceivedEvent implements EditorEvent {
  const EditorSignalReceivedEvent(this.signal);
  

 final  EditorSignal signal;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorSignalReceivedEvent&&(identical(other.signal, signal) || other.signal == signal));
}


@override
int get hashCode => Object.hash(runtimeType,signal);

@override
String toString() {
  return 'EditorEvent.signalReceived(signal: $signal)';
}


}




/// @nodoc


class EditorEditedEvent implements EditorEvent {
  const EditorEditedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEditedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent.edited()';
}


}




/// @nodoc


class EditorCaretMovedEvent implements EditorEvent {
  const EditorCaretMovedEvent(this.context);
  

 final  EditorContext context;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorCaretMovedEvent&&(identical(other.context, context) || other.context == context));
}


@override
int get hashCode => Object.hash(runtimeType,context);

@override
String toString() {
  return 'EditorEvent.caretMoved(context: $context)';
}


}




/// @nodoc


class EditorCompletionDismissedEvent implements EditorEvent {
  const EditorCompletionDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorCompletionDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent.completionDismissed()';
}


}




/// @nodoc


class EditorPropertiesToggledEvent implements EditorEvent {
  const EditorPropertiesToggledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorPropertiesToggledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent.propertiesToggled()';
}


}




/// @nodoc


class EditorPropertySetEvent implements EditorEvent {
  const EditorPropertySetEvent({required this.key, required this.value, this.index});
  

 final  String key;
 final  String value;
 final  int? index;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorPropertySetEvent&&(identical(other.key, key) || other.key == key)&&(identical(other.value, value) || other.value == value)&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,key,value,index);

@override
String toString() {
  return 'EditorEvent.propertySet(key: $key, value: $value, index: $index)';
}


}




/// @nodoc


class EditorPropertyRemovedEvent implements EditorEvent {
  const EditorPropertyRemovedEvent(this.index);
  

 final  int index;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorPropertyRemovedEvent&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,index);

@override
String toString() {
  return 'EditorEvent.propertyRemoved(index: $index)';
}


}




/// @nodoc


class EditorWordGoalChangedEvent implements EditorEvent {
  const EditorWordGoalChangedEvent(this.setting);
  

 final  core.GoalSetting setting;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorWordGoalChangedEvent&&(identical(other.setting, setting) || other.setting == setting));
}


@override
int get hashCode => Object.hash(runtimeType,setting);

@override
String toString() {
  return 'EditorEvent.wordGoalChanged(setting: $setting)';
}


}




/// @nodoc


class EditorKeepMineRequestedEvent implements EditorEvent {
  const EditorKeepMineRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorKeepMineRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent.keepMineRequested()';
}


}




/// @nodoc


class EditorUseTheirsRequestedEvent implements EditorEvent {
  const EditorUseTheirsRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorUseTheirsRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent.useTheirsRequested()';
}


}




/// @nodoc


class EditorFlushRequestedEvent implements EditorEvent {
  const EditorFlushRequestedEvent(this.reason);
  

 final  FlushReason reason;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorFlushRequestedEvent&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'EditorEvent.flushRequested(reason: $reason)';
}


}




/// @nodoc


class EditorLifecycleChangedEvent implements EditorEvent {
  const EditorLifecycleChangedEvent(this.lifecycle);
  

 final  AppLifecycleState lifecycle;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorLifecycleChangedEvent&&(identical(other.lifecycle, lifecycle) || other.lifecycle == lifecycle));
}


@override
int get hashCode => Object.hash(runtimeType,lifecycle);

@override
String toString() {
  return 'EditorEvent.lifecycleChanged(lifecycle: $lifecycle)';
}


}




/// @nodoc


class EditorLinkOpenRequestedEvent implements EditorEvent {
  const EditorLinkOpenRequestedEvent(this.target);
  

 final  LinkTarget target;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorLinkOpenRequestedEvent&&(identical(other.target, target) || other.target == target));
}


@override
int get hashCode => Object.hash(runtimeType,target);

@override
String toString() {
  return 'EditorEvent.linkOpenRequested(target: $target)';
}


}




/// @nodoc


class EditorDestinationHandledEvent implements EditorEvent {
  const EditorDestinationHandledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorDestinationHandledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent.destinationHandled()';
}


}




/// @nodoc


class EditorClosedEvent implements EditorEvent {
  const EditorClosedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorClosedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent.closed()';
}


}




/// @nodoc


class EditorAttachRequestedEvent implements EditorEvent {
  const EditorAttachRequestedEvent(this.source);
  

 final  MediaSource source;




@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorAttachRequestedEvent&&(identical(other.source, source) || other.source == source));
}


@override
int get hashCode => Object.hash(runtimeType,source);

@override
String toString() {
  return 'EditorEvent.attachRequested(source: $source)';
}


}




/// @nodoc


class EditorAttachNoticeDismissedEvent implements EditorEvent {
  const EditorAttachNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorAttachNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent.attachNoticeDismissed()';
}


}




/// @nodoc
mixin _$EditorState {

/// The note being edited.
 String get noteId; EditorStatus get status;/// The open note, while [status] is ready.
 EditorDocument? get document;/// The note as the editor last took it from the core (opened, reloaded
/// or taken from the other version): the text for the field to show.
/// Null unless [status] is ready.
 EditorSnapshot? get snapshot;/// Whether the note is saved, and the question of a change made
/// elsewhere.
 SaveState? get save;/// The note's properties.
 List<Property> get properties; bool get propertiesOpen;/// Why the last change to the properties did not work.
 ApplicationError? get propertyError;/// What the caret is in, and so what the keyboard bar may do.
 EditorContext? get context;/// What is offered where the caret is, if anything.
 Completion? get completion;/// Where the person asked to go (a link followed), for the screen to go
/// to; it says so with `destinationHandled` when it has.
 EditorDestination? get destination;/// Why a link could not be opened.
 ApplicationError? get linkError;/// How many times the field must take the text of [snapshot] again.
 int get reloadSeq;/// The change the last attachment made, for the field to take; a new
/// one each time [attachSeq] goes up.
 EditOutcome? get attached; int get attachSeq;/// Why things could not be attached, one each, until it is dismissed.
 List<ApplicationError> get attachProblems;/// The note's own goal, the library default, and the resolved target.
 core.GoalSetting? get wordGoalSetting; int? get defaultWordGoal; int? get effectiveWordGoal; ApplicationError? get wordGoalError;/// Why opening failed, while [status] is failed.
 ApplicationError? get error;
/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditorStateCopyWith<EditorState> get copyWith => _$EditorStateCopyWithImpl<EditorState>(this as EditorState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.document, document) || other.document == document)&&(identical(other.snapshot, snapshot) || other.snapshot == snapshot)&&(identical(other.save, save) || other.save == save)&&const DeepCollectionEquality().equals(other.properties, properties)&&(identical(other.propertiesOpen, propertiesOpen) || other.propertiesOpen == propertiesOpen)&&(identical(other.propertyError, propertyError) || other.propertyError == propertyError)&&(identical(other.context, context) || other.context == context)&&(identical(other.completion, completion) || other.completion == completion)&&(identical(other.destination, destination) || other.destination == destination)&&(identical(other.linkError, linkError) || other.linkError == linkError)&&(identical(other.reloadSeq, reloadSeq) || other.reloadSeq == reloadSeq)&&(identical(other.attached, attached) || other.attached == attached)&&(identical(other.attachSeq, attachSeq) || other.attachSeq == attachSeq)&&const DeepCollectionEquality().equals(other.attachProblems, attachProblems)&&(identical(other.wordGoalSetting, wordGoalSetting) || other.wordGoalSetting == wordGoalSetting)&&(identical(other.defaultWordGoal, defaultWordGoal) || other.defaultWordGoal == defaultWordGoal)&&(identical(other.effectiveWordGoal, effectiveWordGoal) || other.effectiveWordGoal == effectiveWordGoal)&&(identical(other.wordGoalError, wordGoalError) || other.wordGoalError == wordGoalError)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hashAll([runtimeType,noteId,status,document,snapshot,save,const DeepCollectionEquality().hash(properties),propertiesOpen,propertyError,context,completion,destination,linkError,reloadSeq,attached,attachSeq,const DeepCollectionEquality().hash(attachProblems),wordGoalSetting,defaultWordGoal,effectiveWordGoal,wordGoalError,error]);

@override
String toString() {
  return 'EditorState(noteId: $noteId, status: $status, document: $document, snapshot: $snapshot, save: $save, properties: $properties, propertiesOpen: $propertiesOpen, propertyError: $propertyError, context: $context, completion: $completion, destination: $destination, linkError: $linkError, reloadSeq: $reloadSeq, attached: $attached, attachSeq: $attachSeq, attachProblems: $attachProblems, wordGoalSetting: $wordGoalSetting, defaultWordGoal: $defaultWordGoal, effectiveWordGoal: $effectiveWordGoal, wordGoalError: $wordGoalError, error: $error)';
}


}

/// @nodoc
abstract mixin class $EditorStateCopyWith<$Res>  {
  factory $EditorStateCopyWith(EditorState value, $Res Function(EditorState) _then) = _$EditorStateCopyWithImpl;
@useResult
$Res call({
 String noteId, EditorStatus status, EditorDocument? document, EditorSnapshot? snapshot, SaveState? save, List<Property> properties, bool propertiesOpen, ApplicationError? propertyError, EditorContext? context, Completion? completion, EditorDestination? destination, ApplicationError? linkError, int reloadSeq, EditOutcome? attached, int attachSeq, List<ApplicationError> attachProblems, core.GoalSetting? wordGoalSetting, int? defaultWordGoal, int? effectiveWordGoal, ApplicationError? wordGoalError, ApplicationError? error
});


$SaveStateCopyWith<$Res>? get save;$ApplicationErrorCopyWith<$Res>? get propertyError;$EditorDestinationCopyWith<$Res>? get destination;$ApplicationErrorCopyWith<$Res>? get linkError;$GoalSettingCopyWith<$Res>? get wordGoalSetting;$ApplicationErrorCopyWith<$Res>? get wordGoalError;$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$EditorStateCopyWithImpl<$Res>
    implements $EditorStateCopyWith<$Res> {
  _$EditorStateCopyWithImpl(this._self, this._then);

  final EditorState _self;
  final $Res Function(EditorState) _then;

/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? status = null,Object? document = freezed,Object? snapshot = freezed,Object? save = freezed,Object? properties = null,Object? propertiesOpen = null,Object? propertyError = freezed,Object? context = freezed,Object? completion = freezed,Object? destination = freezed,Object? linkError = freezed,Object? reloadSeq = null,Object? attached = freezed,Object? attachSeq = null,Object? attachProblems = null,Object? wordGoalSetting = freezed,Object? defaultWordGoal = freezed,Object? effectiveWordGoal = freezed,Object? wordGoalError = freezed,Object? error = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as EditorStatus,document: freezed == document ? _self.document : document // ignore: cast_nullable_to_non_nullable
as EditorDocument?,snapshot: freezed == snapshot ? _self.snapshot : snapshot // ignore: cast_nullable_to_non_nullable
as EditorSnapshot?,save: freezed == save ? _self.save : save // ignore: cast_nullable_to_non_nullable
as SaveState?,properties: null == properties ? _self.properties : properties // ignore: cast_nullable_to_non_nullable
as List<Property>,propertiesOpen: null == propertiesOpen ? _self.propertiesOpen : propertiesOpen // ignore: cast_nullable_to_non_nullable
as bool,propertyError: freezed == propertyError ? _self.propertyError : propertyError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,context: freezed == context ? _self.context : context // ignore: cast_nullable_to_non_nullable
as EditorContext?,completion: freezed == completion ? _self.completion : completion // ignore: cast_nullable_to_non_nullable
as Completion?,destination: freezed == destination ? _self.destination : destination // ignore: cast_nullable_to_non_nullable
as EditorDestination?,linkError: freezed == linkError ? _self.linkError : linkError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,reloadSeq: null == reloadSeq ? _self.reloadSeq : reloadSeq // ignore: cast_nullable_to_non_nullable
as int,attached: freezed == attached ? _self.attached : attached // ignore: cast_nullable_to_non_nullable
as EditOutcome?,attachSeq: null == attachSeq ? _self.attachSeq : attachSeq // ignore: cast_nullable_to_non_nullable
as int,attachProblems: null == attachProblems ? _self.attachProblems : attachProblems // ignore: cast_nullable_to_non_nullable
as List<ApplicationError>,wordGoalSetting: freezed == wordGoalSetting ? _self.wordGoalSetting : wordGoalSetting // ignore: cast_nullable_to_non_nullable
as core.GoalSetting?,defaultWordGoal: freezed == defaultWordGoal ? _self.defaultWordGoal : defaultWordGoal // ignore: cast_nullable_to_non_nullable
as int?,effectiveWordGoal: freezed == effectiveWordGoal ? _self.effectiveWordGoal : effectiveWordGoal // ignore: cast_nullable_to_non_nullable
as int?,wordGoalError: freezed == wordGoalError ? _self.wordGoalError : wordGoalError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$SaveStateCopyWith<$Res>? get save {
    if (_self.save == null) {
    return null;
  }

  return $SaveStateCopyWith<$Res>(_self.save!, (value) {
    return _then(_self.copyWith(save: value));
  });
}/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get propertyError {
    if (_self.propertyError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.propertyError!, (value) {
    return _then(_self.copyWith(propertyError: value));
  });
}/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$EditorDestinationCopyWith<$Res>? get destination {
    if (_self.destination == null) {
    return null;
  }

  return $EditorDestinationCopyWith<$Res>(_self.destination!, (value) {
    return _then(_self.copyWith(destination: value));
  });
}/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get linkError {
    if (_self.linkError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.linkError!, (value) {
    return _then(_self.copyWith(linkError: value));
  });
}/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$GoalSettingCopyWith<$Res>? get wordGoalSetting {
    if (_self.wordGoalSetting == null) {
    return null;
  }

  return $GoalSettingCopyWith<$Res>(_self.wordGoalSetting!, (value) {
    return _then(_self.copyWith(wordGoalSetting: value));
  });
}/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get wordGoalError {
    if (_self.wordGoalError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.wordGoalError!, (value) {
    return _then(_self.copyWith(wordGoalError: value));
  });
}/// Create a copy of EditorState
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
}
}


/// Adds pattern-matching-related methods to [EditorState].
extension EditorStatePatterns on EditorState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _EditorState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _EditorState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _EditorState value)  $default,){
final _that = this;
switch (_that) {
case _EditorState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _EditorState value)?  $default,){
final _that = this;
switch (_that) {
case _EditorState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  EditorStatus status,  EditorDocument? document,  EditorSnapshot? snapshot,  SaveState? save,  List<Property> properties,  bool propertiesOpen,  ApplicationError? propertyError,  EditorContext? context,  Completion? completion,  EditorDestination? destination,  ApplicationError? linkError,  int reloadSeq,  EditOutcome? attached,  int attachSeq,  List<ApplicationError> attachProblems,  core.GoalSetting? wordGoalSetting,  int? defaultWordGoal,  int? effectiveWordGoal,  ApplicationError? wordGoalError,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _EditorState() when $default != null:
return $default(_that.noteId,_that.status,_that.document,_that.snapshot,_that.save,_that.properties,_that.propertiesOpen,_that.propertyError,_that.context,_that.completion,_that.destination,_that.linkError,_that.reloadSeq,_that.attached,_that.attachSeq,_that.attachProblems,_that.wordGoalSetting,_that.defaultWordGoal,_that.effectiveWordGoal,_that.wordGoalError,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  EditorStatus status,  EditorDocument? document,  EditorSnapshot? snapshot,  SaveState? save,  List<Property> properties,  bool propertiesOpen,  ApplicationError? propertyError,  EditorContext? context,  Completion? completion,  EditorDestination? destination,  ApplicationError? linkError,  int reloadSeq,  EditOutcome? attached,  int attachSeq,  List<ApplicationError> attachProblems,  core.GoalSetting? wordGoalSetting,  int? defaultWordGoal,  int? effectiveWordGoal,  ApplicationError? wordGoalError,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _EditorState():
return $default(_that.noteId,_that.status,_that.document,_that.snapshot,_that.save,_that.properties,_that.propertiesOpen,_that.propertyError,_that.context,_that.completion,_that.destination,_that.linkError,_that.reloadSeq,_that.attached,_that.attachSeq,_that.attachProblems,_that.wordGoalSetting,_that.defaultWordGoal,_that.effectiveWordGoal,_that.wordGoalError,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  EditorStatus status,  EditorDocument? document,  EditorSnapshot? snapshot,  SaveState? save,  List<Property> properties,  bool propertiesOpen,  ApplicationError? propertyError,  EditorContext? context,  Completion? completion,  EditorDestination? destination,  ApplicationError? linkError,  int reloadSeq,  EditOutcome? attached,  int attachSeq,  List<ApplicationError> attachProblems,  core.GoalSetting? wordGoalSetting,  int? defaultWordGoal,  int? effectiveWordGoal,  ApplicationError? wordGoalError,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _EditorState() when $default != null:
return $default(_that.noteId,_that.status,_that.document,_that.snapshot,_that.save,_that.properties,_that.propertiesOpen,_that.propertyError,_that.context,_that.completion,_that.destination,_that.linkError,_that.reloadSeq,_that.attached,_that.attachSeq,_that.attachProblems,_that.wordGoalSetting,_that.defaultWordGoal,_that.effectiveWordGoal,_that.wordGoalError,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _EditorState implements EditorState {
  const _EditorState({this.noteId = '', this.status = EditorStatus.opening, this.document, this.snapshot, this.save, final  List<Property> properties = const <Property>[], this.propertiesOpen = false, this.propertyError, this.context, this.completion, this.destination, this.linkError, this.reloadSeq = 0, this.attached, this.attachSeq = 0, final  List<ApplicationError> attachProblems = const <ApplicationError>[], this.wordGoalSetting, this.defaultWordGoal, this.effectiveWordGoal, this.wordGoalError, this.error}): _properties = properties,_attachProblems = attachProblems;
  

/// The note being edited.
@override@JsonKey() final  String noteId;
@override@JsonKey() final  EditorStatus status;
/// The open note, while [status] is ready.
@override final  EditorDocument? document;
/// The note as the editor last took it from the core (opened, reloaded
/// or taken from the other version): the text for the field to show.
/// Null unless [status] is ready.
@override final  EditorSnapshot? snapshot;
/// Whether the note is saved, and the question of a change made
/// elsewhere.
@override final  SaveState? save;
/// The note's properties.
 final  List<Property> _properties;
/// The note's properties.
@override@JsonKey() List<Property> get properties {
  if (_properties is EqualUnmodifiableListView) return _properties;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_properties);
}

@override@JsonKey() final  bool propertiesOpen;
/// Why the last change to the properties did not work.
@override final  ApplicationError? propertyError;
/// What the caret is in, and so what the keyboard bar may do.
@override final  EditorContext? context;
/// What is offered where the caret is, if anything.
@override final  Completion? completion;
/// Where the person asked to go (a link followed), for the screen to go
/// to; it says so with `destinationHandled` when it has.
@override final  EditorDestination? destination;
/// Why a link could not be opened.
@override final  ApplicationError? linkError;
/// How many times the field must take the text of [snapshot] again.
@override@JsonKey() final  int reloadSeq;
/// The change the last attachment made, for the field to take; a new
/// one each time [attachSeq] goes up.
@override final  EditOutcome? attached;
@override@JsonKey() final  int attachSeq;
/// Why things could not be attached, one each, until it is dismissed.
 final  List<ApplicationError> _attachProblems;
/// Why things could not be attached, one each, until it is dismissed.
@override@JsonKey() List<ApplicationError> get attachProblems {
  if (_attachProblems is EqualUnmodifiableListView) return _attachProblems;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_attachProblems);
}

/// The note's own goal, the library default, and the resolved target.
@override final  core.GoalSetting? wordGoalSetting;
@override final  int? defaultWordGoal;
@override final  int? effectiveWordGoal;
@override final  ApplicationError? wordGoalError;
/// Why opening failed, while [status] is failed.
@override final  ApplicationError? error;

/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$EditorStateCopyWith<_EditorState> get copyWith => __$EditorStateCopyWithImpl<_EditorState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _EditorState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.document, document) || other.document == document)&&(identical(other.snapshot, snapshot) || other.snapshot == snapshot)&&(identical(other.save, save) || other.save == save)&&const DeepCollectionEquality().equals(other._properties, _properties)&&(identical(other.propertiesOpen, propertiesOpen) || other.propertiesOpen == propertiesOpen)&&(identical(other.propertyError, propertyError) || other.propertyError == propertyError)&&(identical(other.context, context) || other.context == context)&&(identical(other.completion, completion) || other.completion == completion)&&(identical(other.destination, destination) || other.destination == destination)&&(identical(other.linkError, linkError) || other.linkError == linkError)&&(identical(other.reloadSeq, reloadSeq) || other.reloadSeq == reloadSeq)&&(identical(other.attached, attached) || other.attached == attached)&&(identical(other.attachSeq, attachSeq) || other.attachSeq == attachSeq)&&const DeepCollectionEquality().equals(other._attachProblems, _attachProblems)&&(identical(other.wordGoalSetting, wordGoalSetting) || other.wordGoalSetting == wordGoalSetting)&&(identical(other.defaultWordGoal, defaultWordGoal) || other.defaultWordGoal == defaultWordGoal)&&(identical(other.effectiveWordGoal, effectiveWordGoal) || other.effectiveWordGoal == effectiveWordGoal)&&(identical(other.wordGoalError, wordGoalError) || other.wordGoalError == wordGoalError)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hashAll([runtimeType,noteId,status,document,snapshot,save,const DeepCollectionEquality().hash(_properties),propertiesOpen,propertyError,context,completion,destination,linkError,reloadSeq,attached,attachSeq,const DeepCollectionEquality().hash(_attachProblems),wordGoalSetting,defaultWordGoal,effectiveWordGoal,wordGoalError,error]);

@override
String toString() {
  return 'EditorState(noteId: $noteId, status: $status, document: $document, snapshot: $snapshot, save: $save, properties: $properties, propertiesOpen: $propertiesOpen, propertyError: $propertyError, context: $context, completion: $completion, destination: $destination, linkError: $linkError, reloadSeq: $reloadSeq, attached: $attached, attachSeq: $attachSeq, attachProblems: $attachProblems, wordGoalSetting: $wordGoalSetting, defaultWordGoal: $defaultWordGoal, effectiveWordGoal: $effectiveWordGoal, wordGoalError: $wordGoalError, error: $error)';
}


}

/// @nodoc
abstract mixin class _$EditorStateCopyWith<$Res> implements $EditorStateCopyWith<$Res> {
  factory _$EditorStateCopyWith(_EditorState value, $Res Function(_EditorState) _then) = __$EditorStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, EditorStatus status, EditorDocument? document, EditorSnapshot? snapshot, SaveState? save, List<Property> properties, bool propertiesOpen, ApplicationError? propertyError, EditorContext? context, Completion? completion, EditorDestination? destination, ApplicationError? linkError, int reloadSeq, EditOutcome? attached, int attachSeq, List<ApplicationError> attachProblems, core.GoalSetting? wordGoalSetting, int? defaultWordGoal, int? effectiveWordGoal, ApplicationError? wordGoalError, ApplicationError? error
});


@override $SaveStateCopyWith<$Res>? get save;@override $ApplicationErrorCopyWith<$Res>? get propertyError;@override $EditorDestinationCopyWith<$Res>? get destination;@override $ApplicationErrorCopyWith<$Res>? get linkError;@override $GoalSettingCopyWith<$Res>? get wordGoalSetting;@override $ApplicationErrorCopyWith<$Res>? get wordGoalError;@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$EditorStateCopyWithImpl<$Res>
    implements _$EditorStateCopyWith<$Res> {
  __$EditorStateCopyWithImpl(this._self, this._then);

  final _EditorState _self;
  final $Res Function(_EditorState) _then;

/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? status = null,Object? document = freezed,Object? snapshot = freezed,Object? save = freezed,Object? properties = null,Object? propertiesOpen = null,Object? propertyError = freezed,Object? context = freezed,Object? completion = freezed,Object? destination = freezed,Object? linkError = freezed,Object? reloadSeq = null,Object? attached = freezed,Object? attachSeq = null,Object? attachProblems = null,Object? wordGoalSetting = freezed,Object? defaultWordGoal = freezed,Object? effectiveWordGoal = freezed,Object? wordGoalError = freezed,Object? error = freezed,}) {
  return _then(_EditorState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as EditorStatus,document: freezed == document ? _self.document : document // ignore: cast_nullable_to_non_nullable
as EditorDocument?,snapshot: freezed == snapshot ? _self.snapshot : snapshot // ignore: cast_nullable_to_non_nullable
as EditorSnapshot?,save: freezed == save ? _self.save : save // ignore: cast_nullable_to_non_nullable
as SaveState?,properties: null == properties ? _self._properties : properties // ignore: cast_nullable_to_non_nullable
as List<Property>,propertiesOpen: null == propertiesOpen ? _self.propertiesOpen : propertiesOpen // ignore: cast_nullable_to_non_nullable
as bool,propertyError: freezed == propertyError ? _self.propertyError : propertyError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,context: freezed == context ? _self.context : context // ignore: cast_nullable_to_non_nullable
as EditorContext?,completion: freezed == completion ? _self.completion : completion // ignore: cast_nullable_to_non_nullable
as Completion?,destination: freezed == destination ? _self.destination : destination // ignore: cast_nullable_to_non_nullable
as EditorDestination?,linkError: freezed == linkError ? _self.linkError : linkError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,reloadSeq: null == reloadSeq ? _self.reloadSeq : reloadSeq // ignore: cast_nullable_to_non_nullable
as int,attached: freezed == attached ? _self.attached : attached // ignore: cast_nullable_to_non_nullable
as EditOutcome?,attachSeq: null == attachSeq ? _self.attachSeq : attachSeq // ignore: cast_nullable_to_non_nullable
as int,attachProblems: null == attachProblems ? _self._attachProblems : attachProblems // ignore: cast_nullable_to_non_nullable
as List<ApplicationError>,wordGoalSetting: freezed == wordGoalSetting ? _self.wordGoalSetting : wordGoalSetting // ignore: cast_nullable_to_non_nullable
as core.GoalSetting?,defaultWordGoal: freezed == defaultWordGoal ? _self.defaultWordGoal : defaultWordGoal // ignore: cast_nullable_to_non_nullable
as int?,effectiveWordGoal: freezed == effectiveWordGoal ? _self.effectiveWordGoal : effectiveWordGoal // ignore: cast_nullable_to_non_nullable
as int?,wordGoalError: freezed == wordGoalError ? _self.wordGoalError : wordGoalError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$SaveStateCopyWith<$Res>? get save {
    if (_self.save == null) {
    return null;
  }

  return $SaveStateCopyWith<$Res>(_self.save!, (value) {
    return _then(_self.copyWith(save: value));
  });
}/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get propertyError {
    if (_self.propertyError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.propertyError!, (value) {
    return _then(_self.copyWith(propertyError: value));
  });
}/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$EditorDestinationCopyWith<$Res>? get destination {
    if (_self.destination == null) {
    return null;
  }

  return $EditorDestinationCopyWith<$Res>(_self.destination!, (value) {
    return _then(_self.copyWith(destination: value));
  });
}/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get linkError {
    if (_self.linkError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.linkError!, (value) {
    return _then(_self.copyWith(linkError: value));
  });
}/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$GoalSettingCopyWith<$Res>? get wordGoalSetting {
    if (_self.wordGoalSetting == null) {
    return null;
  }

  return $GoalSettingCopyWith<$Res>(_self.wordGoalSetting!, (value) {
    return _then(_self.copyWith(wordGoalSetting: value));
  });
}/// Create a copy of EditorState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get wordGoalError {
    if (_self.wordGoalError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.wordGoalError!, (value) {
    return _then(_self.copyWith(wordGoalError: value));
  });
}/// Create a copy of EditorState
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
}
}

// dart format on
