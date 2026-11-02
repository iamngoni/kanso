// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'triage_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$TriageEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TriageEvent()';
}


}

/// @nodoc
class $TriageEventCopyWith<$Res>  {
$TriageEventCopyWith(TriageEvent _, $Res Function(TriageEvent) __);
}


/// Adds pattern-matching-related methods to [TriageEvent].
extension TriageEventPatterns on TriageEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( TriageStartedEvent value)?  started,TResult Function( TriageChangesReceivedEvent value)?  changesReceived,TResult Function( TriageReloadedEvent value)?  reloaded,TResult Function( TriageChipAcceptedEvent value)?  chipAccepted,TResult Function( TriageChipRejectedEvent value)?  chipRejected,TResult Function( TriageChipEditedEvent value)?  chipEdited,TResult Function( TriageCardAcceptedAllEvent value)?  cardAcceptedAll,TResult Function( TriageFieldAcceptedEverywhereEvent value)?  fieldAcceptedEverywhere,TResult Function( TriageCardSkippedEvent value)?  cardSkipped,TResult Function( TriageCardFiledEvent value)?  cardFiled,TResult Function( TriageFileAllRequestedEvent value)?  fileAllRequested,TResult Function( TriageFileAllConfirmedEvent value)?  fileAllConfirmed,TResult Function( TriagePlanDismissedEvent value)?  planDismissed,TResult Function( TriageRefineRequestedEvent value)?  refineRequested,TResult Function( TriageRelatedTurnedOnEvent value)?  relatedTurnedOn,TResult Function( TriageNoticeDismissedEvent value)?  noticeDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case TriageStartedEvent() when started != null:
return started(_that);case TriageChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case TriageReloadedEvent() when reloaded != null:
return reloaded(_that);case TriageChipAcceptedEvent() when chipAccepted != null:
return chipAccepted(_that);case TriageChipRejectedEvent() when chipRejected != null:
return chipRejected(_that);case TriageChipEditedEvent() when chipEdited != null:
return chipEdited(_that);case TriageCardAcceptedAllEvent() when cardAcceptedAll != null:
return cardAcceptedAll(_that);case TriageFieldAcceptedEverywhereEvent() when fieldAcceptedEverywhere != null:
return fieldAcceptedEverywhere(_that);case TriageCardSkippedEvent() when cardSkipped != null:
return cardSkipped(_that);case TriageCardFiledEvent() when cardFiled != null:
return cardFiled(_that);case TriageFileAllRequestedEvent() when fileAllRequested != null:
return fileAllRequested(_that);case TriageFileAllConfirmedEvent() when fileAllConfirmed != null:
return fileAllConfirmed(_that);case TriagePlanDismissedEvent() when planDismissed != null:
return planDismissed(_that);case TriageRefineRequestedEvent() when refineRequested != null:
return refineRequested(_that);case TriageRelatedTurnedOnEvent() when relatedTurnedOn != null:
return relatedTurnedOn(_that);case TriageNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( TriageStartedEvent value)  started,required TResult Function( TriageChangesReceivedEvent value)  changesReceived,required TResult Function( TriageReloadedEvent value)  reloaded,required TResult Function( TriageChipAcceptedEvent value)  chipAccepted,required TResult Function( TriageChipRejectedEvent value)  chipRejected,required TResult Function( TriageChipEditedEvent value)  chipEdited,required TResult Function( TriageCardAcceptedAllEvent value)  cardAcceptedAll,required TResult Function( TriageFieldAcceptedEverywhereEvent value)  fieldAcceptedEverywhere,required TResult Function( TriageCardSkippedEvent value)  cardSkipped,required TResult Function( TriageCardFiledEvent value)  cardFiled,required TResult Function( TriageFileAllRequestedEvent value)  fileAllRequested,required TResult Function( TriageFileAllConfirmedEvent value)  fileAllConfirmed,required TResult Function( TriagePlanDismissedEvent value)  planDismissed,required TResult Function( TriageRefineRequestedEvent value)  refineRequested,required TResult Function( TriageRelatedTurnedOnEvent value)  relatedTurnedOn,required TResult Function( TriageNoticeDismissedEvent value)  noticeDismissed,}){
final _that = this;
switch (_that) {
case TriageStartedEvent():
return started(_that);case TriageChangesReceivedEvent():
return changesReceived(_that);case TriageReloadedEvent():
return reloaded(_that);case TriageChipAcceptedEvent():
return chipAccepted(_that);case TriageChipRejectedEvent():
return chipRejected(_that);case TriageChipEditedEvent():
return chipEdited(_that);case TriageCardAcceptedAllEvent():
return cardAcceptedAll(_that);case TriageFieldAcceptedEverywhereEvent():
return fieldAcceptedEverywhere(_that);case TriageCardSkippedEvent():
return cardSkipped(_that);case TriageCardFiledEvent():
return cardFiled(_that);case TriageFileAllRequestedEvent():
return fileAllRequested(_that);case TriageFileAllConfirmedEvent():
return fileAllConfirmed(_that);case TriagePlanDismissedEvent():
return planDismissed(_that);case TriageRefineRequestedEvent():
return refineRequested(_that);case TriageRelatedTurnedOnEvent():
return relatedTurnedOn(_that);case TriageNoticeDismissedEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( TriageStartedEvent value)?  started,TResult? Function( TriageChangesReceivedEvent value)?  changesReceived,TResult? Function( TriageReloadedEvent value)?  reloaded,TResult? Function( TriageChipAcceptedEvent value)?  chipAccepted,TResult? Function( TriageChipRejectedEvent value)?  chipRejected,TResult? Function( TriageChipEditedEvent value)?  chipEdited,TResult? Function( TriageCardAcceptedAllEvent value)?  cardAcceptedAll,TResult? Function( TriageFieldAcceptedEverywhereEvent value)?  fieldAcceptedEverywhere,TResult? Function( TriageCardSkippedEvent value)?  cardSkipped,TResult? Function( TriageCardFiledEvent value)?  cardFiled,TResult? Function( TriageFileAllRequestedEvent value)?  fileAllRequested,TResult? Function( TriageFileAllConfirmedEvent value)?  fileAllConfirmed,TResult? Function( TriagePlanDismissedEvent value)?  planDismissed,TResult? Function( TriageRefineRequestedEvent value)?  refineRequested,TResult? Function( TriageRelatedTurnedOnEvent value)?  relatedTurnedOn,TResult? Function( TriageNoticeDismissedEvent value)?  noticeDismissed,}){
final _that = this;
switch (_that) {
case TriageStartedEvent() when started != null:
return started(_that);case TriageChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case TriageReloadedEvent() when reloaded != null:
return reloaded(_that);case TriageChipAcceptedEvent() when chipAccepted != null:
return chipAccepted(_that);case TriageChipRejectedEvent() when chipRejected != null:
return chipRejected(_that);case TriageChipEditedEvent() when chipEdited != null:
return chipEdited(_that);case TriageCardAcceptedAllEvent() when cardAcceptedAll != null:
return cardAcceptedAll(_that);case TriageFieldAcceptedEverywhereEvent() when fieldAcceptedEverywhere != null:
return fieldAcceptedEverywhere(_that);case TriageCardSkippedEvent() when cardSkipped != null:
return cardSkipped(_that);case TriageCardFiledEvent() when cardFiled != null:
return cardFiled(_that);case TriageFileAllRequestedEvent() when fileAllRequested != null:
return fileAllRequested(_that);case TriageFileAllConfirmedEvent() when fileAllConfirmed != null:
return fileAllConfirmed(_that);case TriagePlanDismissedEvent() when planDismissed != null:
return planDismissed(_that);case TriageRefineRequestedEvent() when refineRequested != null:
return refineRequested(_that);case TriageRelatedTurnedOnEvent() when relatedTurnedOn != null:
return relatedTurnedOn(_that);case TriageNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( List<String> ids)?  changesReceived,TResult Function()?  reloaded,TResult Function( String noteId,  int index)?  chipAccepted,TResult Function( String noteId,  int index)?  chipRejected,TResult Function( String noteId,  int index,  String text)?  chipEdited,TResult Function( String noteId)?  cardAcceptedAll,TResult Function( TriageField field)?  fieldAcceptedEverywhere,TResult Function( String noteId)?  cardSkipped,TResult Function( String noteId)?  cardFiled,TResult Function()?  fileAllRequested,TResult Function()?  fileAllConfirmed,TResult Function()?  planDismissed,TResult Function( String noteId)?  refineRequested,TResult Function()?  relatedTurnedOn,TResult Function()?  noticeDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case TriageStartedEvent() when started != null:
return started();case TriageChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case TriageReloadedEvent() when reloaded != null:
return reloaded();case TriageChipAcceptedEvent() when chipAccepted != null:
return chipAccepted(_that.noteId,_that.index);case TriageChipRejectedEvent() when chipRejected != null:
return chipRejected(_that.noteId,_that.index);case TriageChipEditedEvent() when chipEdited != null:
return chipEdited(_that.noteId,_that.index,_that.text);case TriageCardAcceptedAllEvent() when cardAcceptedAll != null:
return cardAcceptedAll(_that.noteId);case TriageFieldAcceptedEverywhereEvent() when fieldAcceptedEverywhere != null:
return fieldAcceptedEverywhere(_that.field);case TriageCardSkippedEvent() when cardSkipped != null:
return cardSkipped(_that.noteId);case TriageCardFiledEvent() when cardFiled != null:
return cardFiled(_that.noteId);case TriageFileAllRequestedEvent() when fileAllRequested != null:
return fileAllRequested();case TriageFileAllConfirmedEvent() when fileAllConfirmed != null:
return fileAllConfirmed();case TriagePlanDismissedEvent() when planDismissed != null:
return planDismissed();case TriageRefineRequestedEvent() when refineRequested != null:
return refineRequested(_that.noteId);case TriageRelatedTurnedOnEvent() when relatedTurnedOn != null:
return relatedTurnedOn();case TriageNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( List<String> ids)  changesReceived,required TResult Function()  reloaded,required TResult Function( String noteId,  int index)  chipAccepted,required TResult Function( String noteId,  int index)  chipRejected,required TResult Function( String noteId,  int index,  String text)  chipEdited,required TResult Function( String noteId)  cardAcceptedAll,required TResult Function( TriageField field)  fieldAcceptedEverywhere,required TResult Function( String noteId)  cardSkipped,required TResult Function( String noteId)  cardFiled,required TResult Function()  fileAllRequested,required TResult Function()  fileAllConfirmed,required TResult Function()  planDismissed,required TResult Function( String noteId)  refineRequested,required TResult Function()  relatedTurnedOn,required TResult Function()  noticeDismissed,}) {final _that = this;
switch (_that) {
case TriageStartedEvent():
return started();case TriageChangesReceivedEvent():
return changesReceived(_that.ids);case TriageReloadedEvent():
return reloaded();case TriageChipAcceptedEvent():
return chipAccepted(_that.noteId,_that.index);case TriageChipRejectedEvent():
return chipRejected(_that.noteId,_that.index);case TriageChipEditedEvent():
return chipEdited(_that.noteId,_that.index,_that.text);case TriageCardAcceptedAllEvent():
return cardAcceptedAll(_that.noteId);case TriageFieldAcceptedEverywhereEvent():
return fieldAcceptedEverywhere(_that.field);case TriageCardSkippedEvent():
return cardSkipped(_that.noteId);case TriageCardFiledEvent():
return cardFiled(_that.noteId);case TriageFileAllRequestedEvent():
return fileAllRequested();case TriageFileAllConfirmedEvent():
return fileAllConfirmed();case TriagePlanDismissedEvent():
return planDismissed();case TriageRefineRequestedEvent():
return refineRequested(_that.noteId);case TriageRelatedTurnedOnEvent():
return relatedTurnedOn();case TriageNoticeDismissedEvent():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( List<String> ids)?  changesReceived,TResult? Function()?  reloaded,TResult? Function( String noteId,  int index)?  chipAccepted,TResult? Function( String noteId,  int index)?  chipRejected,TResult? Function( String noteId,  int index,  String text)?  chipEdited,TResult? Function( String noteId)?  cardAcceptedAll,TResult? Function( TriageField field)?  fieldAcceptedEverywhere,TResult? Function( String noteId)?  cardSkipped,TResult? Function( String noteId)?  cardFiled,TResult? Function()?  fileAllRequested,TResult? Function()?  fileAllConfirmed,TResult? Function()?  planDismissed,TResult? Function( String noteId)?  refineRequested,TResult? Function()?  relatedTurnedOn,TResult? Function()?  noticeDismissed,}) {final _that = this;
switch (_that) {
case TriageStartedEvent() when started != null:
return started();case TriageChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case TriageReloadedEvent() when reloaded != null:
return reloaded();case TriageChipAcceptedEvent() when chipAccepted != null:
return chipAccepted(_that.noteId,_that.index);case TriageChipRejectedEvent() when chipRejected != null:
return chipRejected(_that.noteId,_that.index);case TriageChipEditedEvent() when chipEdited != null:
return chipEdited(_that.noteId,_that.index,_that.text);case TriageCardAcceptedAllEvent() when cardAcceptedAll != null:
return cardAcceptedAll(_that.noteId);case TriageFieldAcceptedEverywhereEvent() when fieldAcceptedEverywhere != null:
return fieldAcceptedEverywhere(_that.field);case TriageCardSkippedEvent() when cardSkipped != null:
return cardSkipped(_that.noteId);case TriageCardFiledEvent() when cardFiled != null:
return cardFiled(_that.noteId);case TriageFileAllRequestedEvent() when fileAllRequested != null:
return fileAllRequested();case TriageFileAllConfirmedEvent() when fileAllConfirmed != null:
return fileAllConfirmed();case TriagePlanDismissedEvent() when planDismissed != null:
return planDismissed();case TriageRefineRequestedEvent() when refineRequested != null:
return refineRequested(_that.noteId);case TriageRelatedTurnedOnEvent() when relatedTurnedOn != null:
return relatedTurnedOn();case TriageNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case _:
  return null;

}
}

}

/// @nodoc


class TriageStartedEvent implements TriageEvent {
  const TriageStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TriageEvent.started()';
}


}




/// @nodoc


class TriageChangesReceivedEvent implements TriageEvent {
  const TriageChangesReceivedEvent(final  List<String> ids): _ids = ids;
  

 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageChangesReceivedEventCopyWith<TriageChangesReceivedEvent> get copyWith => _$TriageChangesReceivedEventCopyWithImpl<TriageChangesReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageChangesReceivedEvent&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'TriageEvent.changesReceived(ids: $ids)';
}


}

/// @nodoc
abstract mixin class $TriageChangesReceivedEventCopyWith<$Res> implements $TriageEventCopyWith<$Res> {
  factory $TriageChangesReceivedEventCopyWith(TriageChangesReceivedEvent value, $Res Function(TriageChangesReceivedEvent) _then) = _$TriageChangesReceivedEventCopyWithImpl;
@useResult
$Res call({
 List<String> ids
});




}
/// @nodoc
class _$TriageChangesReceivedEventCopyWithImpl<$Res>
    implements $TriageChangesReceivedEventCopyWith<$Res> {
  _$TriageChangesReceivedEventCopyWithImpl(this._self, this._then);

  final TriageChangesReceivedEvent _self;
  final $Res Function(TriageChangesReceivedEvent) _then;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ids = null,}) {
  return _then(TriageChangesReceivedEvent(
null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class TriageReloadedEvent implements TriageEvent {
  const TriageReloadedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageReloadedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TriageEvent.reloaded()';
}


}




/// @nodoc


class TriageChipAcceptedEvent implements TriageEvent {
  const TriageChipAcceptedEvent(this.noteId, this.index);
  

 final  String noteId;
 final  int index;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageChipAcceptedEventCopyWith<TriageChipAcceptedEvent> get copyWith => _$TriageChipAcceptedEventCopyWithImpl<TriageChipAcceptedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageChipAcceptedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,index);

@override
String toString() {
  return 'TriageEvent.chipAccepted(noteId: $noteId, index: $index)';
}


}

/// @nodoc
abstract mixin class $TriageChipAcceptedEventCopyWith<$Res> implements $TriageEventCopyWith<$Res> {
  factory $TriageChipAcceptedEventCopyWith(TriageChipAcceptedEvent value, $Res Function(TriageChipAcceptedEvent) _then) = _$TriageChipAcceptedEventCopyWithImpl;
@useResult
$Res call({
 String noteId, int index
});




}
/// @nodoc
class _$TriageChipAcceptedEventCopyWithImpl<$Res>
    implements $TriageChipAcceptedEventCopyWith<$Res> {
  _$TriageChipAcceptedEventCopyWithImpl(this._self, this._then);

  final TriageChipAcceptedEvent _self;
  final $Res Function(TriageChipAcceptedEvent) _then;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? index = null,}) {
  return _then(TriageChipAcceptedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class TriageChipRejectedEvent implements TriageEvent {
  const TriageChipRejectedEvent(this.noteId, this.index);
  

 final  String noteId;
 final  int index;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageChipRejectedEventCopyWith<TriageChipRejectedEvent> get copyWith => _$TriageChipRejectedEventCopyWithImpl<TriageChipRejectedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageChipRejectedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,index);

@override
String toString() {
  return 'TriageEvent.chipRejected(noteId: $noteId, index: $index)';
}


}

/// @nodoc
abstract mixin class $TriageChipRejectedEventCopyWith<$Res> implements $TriageEventCopyWith<$Res> {
  factory $TriageChipRejectedEventCopyWith(TriageChipRejectedEvent value, $Res Function(TriageChipRejectedEvent) _then) = _$TriageChipRejectedEventCopyWithImpl;
@useResult
$Res call({
 String noteId, int index
});




}
/// @nodoc
class _$TriageChipRejectedEventCopyWithImpl<$Res>
    implements $TriageChipRejectedEventCopyWith<$Res> {
  _$TriageChipRejectedEventCopyWithImpl(this._self, this._then);

  final TriageChipRejectedEvent _self;
  final $Res Function(TriageChipRejectedEvent) _then;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? index = null,}) {
  return _then(TriageChipRejectedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class TriageChipEditedEvent implements TriageEvent {
  const TriageChipEditedEvent(this.noteId, this.index, this.text);
  

 final  String noteId;
 final  int index;
 final  String text;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageChipEditedEventCopyWith<TriageChipEditedEvent> get copyWith => _$TriageChipEditedEventCopyWithImpl<TriageChipEditedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageChipEditedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.index, index) || other.index == index)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,index,text);

@override
String toString() {
  return 'TriageEvent.chipEdited(noteId: $noteId, index: $index, text: $text)';
}


}

/// @nodoc
abstract mixin class $TriageChipEditedEventCopyWith<$Res> implements $TriageEventCopyWith<$Res> {
  factory $TriageChipEditedEventCopyWith(TriageChipEditedEvent value, $Res Function(TriageChipEditedEvent) _then) = _$TriageChipEditedEventCopyWithImpl;
@useResult
$Res call({
 String noteId, int index, String text
});




}
/// @nodoc
class _$TriageChipEditedEventCopyWithImpl<$Res>
    implements $TriageChipEditedEventCopyWith<$Res> {
  _$TriageChipEditedEventCopyWithImpl(this._self, this._then);

  final TriageChipEditedEvent _self;
  final $Res Function(TriageChipEditedEvent) _then;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? index = null,Object? text = null,}) {
  return _then(TriageChipEditedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class TriageCardAcceptedAllEvent implements TriageEvent {
  const TriageCardAcceptedAllEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageCardAcceptedAllEventCopyWith<TriageCardAcceptedAllEvent> get copyWith => _$TriageCardAcceptedAllEventCopyWithImpl<TriageCardAcceptedAllEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageCardAcceptedAllEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'TriageEvent.cardAcceptedAll(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $TriageCardAcceptedAllEventCopyWith<$Res> implements $TriageEventCopyWith<$Res> {
  factory $TriageCardAcceptedAllEventCopyWith(TriageCardAcceptedAllEvent value, $Res Function(TriageCardAcceptedAllEvent) _then) = _$TriageCardAcceptedAllEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$TriageCardAcceptedAllEventCopyWithImpl<$Res>
    implements $TriageCardAcceptedAllEventCopyWith<$Res> {
  _$TriageCardAcceptedAllEventCopyWithImpl(this._self, this._then);

  final TriageCardAcceptedAllEvent _self;
  final $Res Function(TriageCardAcceptedAllEvent) _then;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(TriageCardAcceptedAllEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class TriageFieldAcceptedEverywhereEvent implements TriageEvent {
  const TriageFieldAcceptedEverywhereEvent(this.field);
  

 final  TriageField field;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageFieldAcceptedEverywhereEventCopyWith<TriageFieldAcceptedEverywhereEvent> get copyWith => _$TriageFieldAcceptedEverywhereEventCopyWithImpl<TriageFieldAcceptedEverywhereEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageFieldAcceptedEverywhereEvent&&(identical(other.field, field) || other.field == field));
}


@override
int get hashCode => Object.hash(runtimeType,field);

@override
String toString() {
  return 'TriageEvent.fieldAcceptedEverywhere(field: $field)';
}


}

/// @nodoc
abstract mixin class $TriageFieldAcceptedEverywhereEventCopyWith<$Res> implements $TriageEventCopyWith<$Res> {
  factory $TriageFieldAcceptedEverywhereEventCopyWith(TriageFieldAcceptedEverywhereEvent value, $Res Function(TriageFieldAcceptedEverywhereEvent) _then) = _$TriageFieldAcceptedEverywhereEventCopyWithImpl;
@useResult
$Res call({
 TriageField field
});




}
/// @nodoc
class _$TriageFieldAcceptedEverywhereEventCopyWithImpl<$Res>
    implements $TriageFieldAcceptedEverywhereEventCopyWith<$Res> {
  _$TriageFieldAcceptedEverywhereEventCopyWithImpl(this._self, this._then);

  final TriageFieldAcceptedEverywhereEvent _self;
  final $Res Function(TriageFieldAcceptedEverywhereEvent) _then;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field = null,}) {
  return _then(TriageFieldAcceptedEverywhereEvent(
null == field ? _self.field : field // ignore: cast_nullable_to_non_nullable
as TriageField,
  ));
}


}

/// @nodoc


class TriageCardSkippedEvent implements TriageEvent {
  const TriageCardSkippedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageCardSkippedEventCopyWith<TriageCardSkippedEvent> get copyWith => _$TriageCardSkippedEventCopyWithImpl<TriageCardSkippedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageCardSkippedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'TriageEvent.cardSkipped(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $TriageCardSkippedEventCopyWith<$Res> implements $TriageEventCopyWith<$Res> {
  factory $TriageCardSkippedEventCopyWith(TriageCardSkippedEvent value, $Res Function(TriageCardSkippedEvent) _then) = _$TriageCardSkippedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$TriageCardSkippedEventCopyWithImpl<$Res>
    implements $TriageCardSkippedEventCopyWith<$Res> {
  _$TriageCardSkippedEventCopyWithImpl(this._self, this._then);

  final TriageCardSkippedEvent _self;
  final $Res Function(TriageCardSkippedEvent) _then;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(TriageCardSkippedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class TriageCardFiledEvent implements TriageEvent {
  const TriageCardFiledEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageCardFiledEventCopyWith<TriageCardFiledEvent> get copyWith => _$TriageCardFiledEventCopyWithImpl<TriageCardFiledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageCardFiledEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'TriageEvent.cardFiled(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $TriageCardFiledEventCopyWith<$Res> implements $TriageEventCopyWith<$Res> {
  factory $TriageCardFiledEventCopyWith(TriageCardFiledEvent value, $Res Function(TriageCardFiledEvent) _then) = _$TriageCardFiledEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$TriageCardFiledEventCopyWithImpl<$Res>
    implements $TriageCardFiledEventCopyWith<$Res> {
  _$TriageCardFiledEventCopyWithImpl(this._self, this._then);

  final TriageCardFiledEvent _self;
  final $Res Function(TriageCardFiledEvent) _then;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(TriageCardFiledEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class TriageFileAllRequestedEvent implements TriageEvent {
  const TriageFileAllRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageFileAllRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TriageEvent.fileAllRequested()';
}


}




/// @nodoc


class TriageFileAllConfirmedEvent implements TriageEvent {
  const TriageFileAllConfirmedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageFileAllConfirmedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TriageEvent.fileAllConfirmed()';
}


}




/// @nodoc


class TriagePlanDismissedEvent implements TriageEvent {
  const TriagePlanDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriagePlanDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TriageEvent.planDismissed()';
}


}




/// @nodoc


class TriageRefineRequestedEvent implements TriageEvent {
  const TriageRefineRequestedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageRefineRequestedEventCopyWith<TriageRefineRequestedEvent> get copyWith => _$TriageRefineRequestedEventCopyWithImpl<TriageRefineRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageRefineRequestedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'TriageEvent.refineRequested(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $TriageRefineRequestedEventCopyWith<$Res> implements $TriageEventCopyWith<$Res> {
  factory $TriageRefineRequestedEventCopyWith(TriageRefineRequestedEvent value, $Res Function(TriageRefineRequestedEvent) _then) = _$TriageRefineRequestedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$TriageRefineRequestedEventCopyWithImpl<$Res>
    implements $TriageRefineRequestedEventCopyWith<$Res> {
  _$TriageRefineRequestedEventCopyWithImpl(this._self, this._then);

  final TriageRefineRequestedEvent _self;
  final $Res Function(TriageRefineRequestedEvent) _then;

/// Create a copy of TriageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(TriageRefineRequestedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class TriageRelatedTurnedOnEvent implements TriageEvent {
  const TriageRelatedTurnedOnEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageRelatedTurnedOnEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TriageEvent.relatedTurnedOn()';
}


}




/// @nodoc


class TriageNoticeDismissedEvent implements TriageEvent {
  const TriageNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TriageEvent.noticeDismissed()';
}


}




/// @nodoc
mixin _$TriageState {

 LoadStatus get status;/// Why reading failed, while [status] is failed.
 ApplicationError? get error;/// Every card, skipped ones included.
 List<TriageCardState> get cards;/// How many notes wait and what is switched on, as last read.
 TriageStatus? get info;/// What "File all accepted" would do, until the person answers.
 TriageFiled? get plan;/// Why the last edit or filing did not happen, until it is dismissed.
 String? get notice;
/// Create a copy of TriageState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TriageStateCopyWith<TriageState> get copyWith => _$TriageStateCopyWithImpl<TriageState>(this as TriageState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TriageState&&(identical(other.status, status) || other.status == status)&&(identical(other.error, error) || other.error == error)&&const DeepCollectionEquality().equals(other.cards, cards)&&(identical(other.info, info) || other.info == info)&&(identical(other.plan, plan) || other.plan == plan)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,status,error,const DeepCollectionEquality().hash(cards),info,plan,notice);

@override
String toString() {
  return 'TriageState(status: $status, error: $error, cards: $cards, info: $info, plan: $plan, notice: $notice)';
}


}

/// @nodoc
abstract mixin class $TriageStateCopyWith<$Res>  {
  factory $TriageStateCopyWith(TriageState value, $Res Function(TriageState) _then) = _$TriageStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, ApplicationError? error, List<TriageCardState> cards, TriageStatus? info, TriageFiled? plan, String? notice
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$TriageStateCopyWithImpl<$Res>
    implements $TriageStateCopyWith<$Res> {
  _$TriageStateCopyWithImpl(this._self, this._then);

  final TriageState _self;
  final $Res Function(TriageState) _then;

/// Create a copy of TriageState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? error = freezed,Object? cards = null,Object? info = freezed,Object? plan = freezed,Object? notice = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,cards: null == cards ? _self.cards : cards // ignore: cast_nullable_to_non_nullable
as List<TriageCardState>,info: freezed == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as TriageStatus?,plan: freezed == plan ? _self.plan : plan // ignore: cast_nullable_to_non_nullable
as TriageFiled?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}
/// Create a copy of TriageState
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


/// Adds pattern-matching-related methods to [TriageState].
extension TriageStatePatterns on TriageState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TriageState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TriageState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TriageState value)  $default,){
final _that = this;
switch (_that) {
case _TriageState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TriageState value)?  $default,){
final _that = this;
switch (_that) {
case _TriageState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  ApplicationError? error,  List<TriageCardState> cards,  TriageStatus? info,  TriageFiled? plan,  String? notice)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TriageState() when $default != null:
return $default(_that.status,_that.error,_that.cards,_that.info,_that.plan,_that.notice);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  ApplicationError? error,  List<TriageCardState> cards,  TriageStatus? info,  TriageFiled? plan,  String? notice)  $default,) {final _that = this;
switch (_that) {
case _TriageState():
return $default(_that.status,_that.error,_that.cards,_that.info,_that.plan,_that.notice);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  ApplicationError? error,  List<TriageCardState> cards,  TriageStatus? info,  TriageFiled? plan,  String? notice)?  $default,) {final _that = this;
switch (_that) {
case _TriageState() when $default != null:
return $default(_that.status,_that.error,_that.cards,_that.info,_that.plan,_that.notice);case _:
  return null;

}
}

}

/// @nodoc


class _TriageState extends TriageState {
  const _TriageState({this.status = LoadStatus.loading, this.error, final  List<TriageCardState> cards = const <TriageCardState>[], this.info, this.plan, this.notice}): _cards = cards,super._();
  

@override@JsonKey() final  LoadStatus status;
/// Why reading failed, while [status] is failed.
@override final  ApplicationError? error;
/// Every card, skipped ones included.
 final  List<TriageCardState> _cards;
/// Every card, skipped ones included.
@override@JsonKey() List<TriageCardState> get cards {
  if (_cards is EqualUnmodifiableListView) return _cards;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_cards);
}

/// How many notes wait and what is switched on, as last read.
@override final  TriageStatus? info;
/// What "File all accepted" would do, until the person answers.
@override final  TriageFiled? plan;
/// Why the last edit or filing did not happen, until it is dismissed.
@override final  String? notice;

/// Create a copy of TriageState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TriageStateCopyWith<_TriageState> get copyWith => __$TriageStateCopyWithImpl<_TriageState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TriageState&&(identical(other.status, status) || other.status == status)&&(identical(other.error, error) || other.error == error)&&const DeepCollectionEquality().equals(other._cards, _cards)&&(identical(other.info, info) || other.info == info)&&(identical(other.plan, plan) || other.plan == plan)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,status,error,const DeepCollectionEquality().hash(_cards),info,plan,notice);

@override
String toString() {
  return 'TriageState(status: $status, error: $error, cards: $cards, info: $info, plan: $plan, notice: $notice)';
}


}

/// @nodoc
abstract mixin class _$TriageStateCopyWith<$Res> implements $TriageStateCopyWith<$Res> {
  factory _$TriageStateCopyWith(_TriageState value, $Res Function(_TriageState) _then) = __$TriageStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, ApplicationError? error, List<TriageCardState> cards, TriageStatus? info, TriageFiled? plan, String? notice
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$TriageStateCopyWithImpl<$Res>
    implements _$TriageStateCopyWith<$Res> {
  __$TriageStateCopyWithImpl(this._self, this._then);

  final _TriageState _self;
  final $Res Function(_TriageState) _then;

/// Create a copy of TriageState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? error = freezed,Object? cards = null,Object? info = freezed,Object? plan = freezed,Object? notice = freezed,}) {
  return _then(_TriageState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,cards: null == cards ? _self._cards : cards // ignore: cast_nullable_to_non_nullable
as List<TriageCardState>,info: freezed == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as TriageStatus?,plan: freezed == plan ? _self.plan : plan // ignore: cast_nullable_to_non_nullable
as TriageFiled?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of TriageState
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
