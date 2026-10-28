// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_history_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteHistoryEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteHistoryEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteHistoryEvent()';
}


}

/// @nodoc
class $NoteHistoryEventCopyWith<$Res>  {
$NoteHistoryEventCopyWith(NoteHistoryEvent _, $Res Function(NoteHistoryEvent) __);
}


/// Adds pattern-matching-related methods to [NoteHistoryEvent].
extension NoteHistoryEventPatterns on NoteHistoryEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteHistoryStartedEvent value)?  started,TResult Function( NoteHistoryRetriedEvent value)?  retried,TResult Function( NoteHistoryVersionSelectedEvent value)?  versionSelected,TResult Function( NoteHistoryPaneChangedEvent value)?  paneChanged,TResult Function( NoteHistorySectionCopiedEvent value)?  sectionCopied,TResult Function( NoteHistoryRestoreRequestedEvent value)?  restoreRequested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteHistoryStartedEvent() when started != null:
return started(_that);case NoteHistoryRetriedEvent() when retried != null:
return retried(_that);case NoteHistoryVersionSelectedEvent() when versionSelected != null:
return versionSelected(_that);case NoteHistoryPaneChangedEvent() when paneChanged != null:
return paneChanged(_that);case NoteHistorySectionCopiedEvent() when sectionCopied != null:
return sectionCopied(_that);case NoteHistoryRestoreRequestedEvent() when restoreRequested != null:
return restoreRequested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteHistoryStartedEvent value)  started,required TResult Function( NoteHistoryRetriedEvent value)  retried,required TResult Function( NoteHistoryVersionSelectedEvent value)  versionSelected,required TResult Function( NoteHistoryPaneChangedEvent value)  paneChanged,required TResult Function( NoteHistorySectionCopiedEvent value)  sectionCopied,required TResult Function( NoteHistoryRestoreRequestedEvent value)  restoreRequested,}){
final _that = this;
switch (_that) {
case NoteHistoryStartedEvent():
return started(_that);case NoteHistoryRetriedEvent():
return retried(_that);case NoteHistoryVersionSelectedEvent():
return versionSelected(_that);case NoteHistoryPaneChangedEvent():
return paneChanged(_that);case NoteHistorySectionCopiedEvent():
return sectionCopied(_that);case NoteHistoryRestoreRequestedEvent():
return restoreRequested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteHistoryStartedEvent value)?  started,TResult? Function( NoteHistoryRetriedEvent value)?  retried,TResult? Function( NoteHistoryVersionSelectedEvent value)?  versionSelected,TResult? Function( NoteHistoryPaneChangedEvent value)?  paneChanged,TResult? Function( NoteHistorySectionCopiedEvent value)?  sectionCopied,TResult? Function( NoteHistoryRestoreRequestedEvent value)?  restoreRequested,}){
final _that = this;
switch (_that) {
case NoteHistoryStartedEvent() when started != null:
return started(_that);case NoteHistoryRetriedEvent() when retried != null:
return retried(_that);case NoteHistoryVersionSelectedEvent() when versionSelected != null:
return versionSelected(_that);case NoteHistoryPaneChangedEvent() when paneChanged != null:
return paneChanged(_that);case NoteHistorySectionCopiedEvent() when sectionCopied != null:
return sectionCopied(_that);case NoteHistoryRestoreRequestedEvent() when restoreRequested != null:
return restoreRequested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  started,TResult Function()?  retried,TResult Function( String rev)?  versionSelected,TResult Function( HistoryPane pane)?  paneChanged,TResult Function( int index)?  sectionCopied,TResult Function()?  restoreRequested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteHistoryStartedEvent() when started != null:
return started(_that.noteId);case NoteHistoryRetriedEvent() when retried != null:
return retried();case NoteHistoryVersionSelectedEvent() when versionSelected != null:
return versionSelected(_that.rev);case NoteHistoryPaneChangedEvent() when paneChanged != null:
return paneChanged(_that.pane);case NoteHistorySectionCopiedEvent() when sectionCopied != null:
return sectionCopied(_that.index);case NoteHistoryRestoreRequestedEvent() when restoreRequested != null:
return restoreRequested();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  started,required TResult Function()  retried,required TResult Function( String rev)  versionSelected,required TResult Function( HistoryPane pane)  paneChanged,required TResult Function( int index)  sectionCopied,required TResult Function()  restoreRequested,}) {final _that = this;
switch (_that) {
case NoteHistoryStartedEvent():
return started(_that.noteId);case NoteHistoryRetriedEvent():
return retried();case NoteHistoryVersionSelectedEvent():
return versionSelected(_that.rev);case NoteHistoryPaneChangedEvent():
return paneChanged(_that.pane);case NoteHistorySectionCopiedEvent():
return sectionCopied(_that.index);case NoteHistoryRestoreRequestedEvent():
return restoreRequested();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  started,TResult? Function()?  retried,TResult? Function( String rev)?  versionSelected,TResult? Function( HistoryPane pane)?  paneChanged,TResult? Function( int index)?  sectionCopied,TResult? Function()?  restoreRequested,}) {final _that = this;
switch (_that) {
case NoteHistoryStartedEvent() when started != null:
return started(_that.noteId);case NoteHistoryRetriedEvent() when retried != null:
return retried();case NoteHistoryVersionSelectedEvent() when versionSelected != null:
return versionSelected(_that.rev);case NoteHistoryPaneChangedEvent() when paneChanged != null:
return paneChanged(_that.pane);case NoteHistorySectionCopiedEvent() when sectionCopied != null:
return sectionCopied(_that.index);case NoteHistoryRestoreRequestedEvent() when restoreRequested != null:
return restoreRequested();case _:
  return null;

}
}

}

/// @nodoc


class NoteHistoryStartedEvent implements NoteHistoryEvent {
  const NoteHistoryStartedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of NoteHistoryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteHistoryStartedEventCopyWith<NoteHistoryStartedEvent> get copyWith => _$NoteHistoryStartedEventCopyWithImpl<NoteHistoryStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteHistoryStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'NoteHistoryEvent.started(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $NoteHistoryStartedEventCopyWith<$Res> implements $NoteHistoryEventCopyWith<$Res> {
  factory $NoteHistoryStartedEventCopyWith(NoteHistoryStartedEvent value, $Res Function(NoteHistoryStartedEvent) _then) = _$NoteHistoryStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$NoteHistoryStartedEventCopyWithImpl<$Res>
    implements $NoteHistoryStartedEventCopyWith<$Res> {
  _$NoteHistoryStartedEventCopyWithImpl(this._self, this._then);

  final NoteHistoryStartedEvent _self;
  final $Res Function(NoteHistoryStartedEvent) _then;

/// Create a copy of NoteHistoryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(NoteHistoryStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteHistoryRetriedEvent implements NoteHistoryEvent {
  const NoteHistoryRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteHistoryRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteHistoryEvent.retried()';
}


}




/// @nodoc


class NoteHistoryVersionSelectedEvent implements NoteHistoryEvent {
  const NoteHistoryVersionSelectedEvent(this.rev);
  

 final  String rev;

/// Create a copy of NoteHistoryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteHistoryVersionSelectedEventCopyWith<NoteHistoryVersionSelectedEvent> get copyWith => _$NoteHistoryVersionSelectedEventCopyWithImpl<NoteHistoryVersionSelectedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteHistoryVersionSelectedEvent&&(identical(other.rev, rev) || other.rev == rev));
}


@override
int get hashCode => Object.hash(runtimeType,rev);

@override
String toString() {
  return 'NoteHistoryEvent.versionSelected(rev: $rev)';
}


}

/// @nodoc
abstract mixin class $NoteHistoryVersionSelectedEventCopyWith<$Res> implements $NoteHistoryEventCopyWith<$Res> {
  factory $NoteHistoryVersionSelectedEventCopyWith(NoteHistoryVersionSelectedEvent value, $Res Function(NoteHistoryVersionSelectedEvent) _then) = _$NoteHistoryVersionSelectedEventCopyWithImpl;
@useResult
$Res call({
 String rev
});




}
/// @nodoc
class _$NoteHistoryVersionSelectedEventCopyWithImpl<$Res>
    implements $NoteHistoryVersionSelectedEventCopyWith<$Res> {
  _$NoteHistoryVersionSelectedEventCopyWithImpl(this._self, this._then);

  final NoteHistoryVersionSelectedEvent _self;
  final $Res Function(NoteHistoryVersionSelectedEvent) _then;

/// Create a copy of NoteHistoryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? rev = null,}) {
  return _then(NoteHistoryVersionSelectedEvent(
null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteHistoryPaneChangedEvent implements NoteHistoryEvent {
  const NoteHistoryPaneChangedEvent(this.pane);
  

 final  HistoryPane pane;

/// Create a copy of NoteHistoryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteHistoryPaneChangedEventCopyWith<NoteHistoryPaneChangedEvent> get copyWith => _$NoteHistoryPaneChangedEventCopyWithImpl<NoteHistoryPaneChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteHistoryPaneChangedEvent&&(identical(other.pane, pane) || other.pane == pane));
}


@override
int get hashCode => Object.hash(runtimeType,pane);

@override
String toString() {
  return 'NoteHistoryEvent.paneChanged(pane: $pane)';
}


}

/// @nodoc
abstract mixin class $NoteHistoryPaneChangedEventCopyWith<$Res> implements $NoteHistoryEventCopyWith<$Res> {
  factory $NoteHistoryPaneChangedEventCopyWith(NoteHistoryPaneChangedEvent value, $Res Function(NoteHistoryPaneChangedEvent) _then) = _$NoteHistoryPaneChangedEventCopyWithImpl;
@useResult
$Res call({
 HistoryPane pane
});




}
/// @nodoc
class _$NoteHistoryPaneChangedEventCopyWithImpl<$Res>
    implements $NoteHistoryPaneChangedEventCopyWith<$Res> {
  _$NoteHistoryPaneChangedEventCopyWithImpl(this._self, this._then);

  final NoteHistoryPaneChangedEvent _self;
  final $Res Function(NoteHistoryPaneChangedEvent) _then;

/// Create a copy of NoteHistoryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? pane = null,}) {
  return _then(NoteHistoryPaneChangedEvent(
null == pane ? _self.pane : pane // ignore: cast_nullable_to_non_nullable
as HistoryPane,
  ));
}


}

/// @nodoc


class NoteHistorySectionCopiedEvent implements NoteHistoryEvent {
  const NoteHistorySectionCopiedEvent(this.index);
  

 final  int index;

/// Create a copy of NoteHistoryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteHistorySectionCopiedEventCopyWith<NoteHistorySectionCopiedEvent> get copyWith => _$NoteHistorySectionCopiedEventCopyWithImpl<NoteHistorySectionCopiedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteHistorySectionCopiedEvent&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,index);

@override
String toString() {
  return 'NoteHistoryEvent.sectionCopied(index: $index)';
}


}

/// @nodoc
abstract mixin class $NoteHistorySectionCopiedEventCopyWith<$Res> implements $NoteHistoryEventCopyWith<$Res> {
  factory $NoteHistorySectionCopiedEventCopyWith(NoteHistorySectionCopiedEvent value, $Res Function(NoteHistorySectionCopiedEvent) _then) = _$NoteHistorySectionCopiedEventCopyWithImpl;
@useResult
$Res call({
 int index
});




}
/// @nodoc
class _$NoteHistorySectionCopiedEventCopyWithImpl<$Res>
    implements $NoteHistorySectionCopiedEventCopyWith<$Res> {
  _$NoteHistorySectionCopiedEventCopyWithImpl(this._self, this._then);

  final NoteHistorySectionCopiedEvent _self;
  final $Res Function(NoteHistorySectionCopiedEvent) _then;

/// Create a copy of NoteHistoryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? index = null,}) {
  return _then(NoteHistorySectionCopiedEvent(
null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class NoteHistoryRestoreRequestedEvent implements NoteHistoryEvent {
  const NoteHistoryRestoreRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteHistoryRestoreRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteHistoryEvent.restoreRequested()';
}


}




/// @nodoc
mixin _$NoteHistoryState {

/// The note whose history this is.
 String get noteId; HistoryStatus get status;/// The versions, newest first, beside the note as it is now.
 NoteHistory? get history;/// The note's revision when the history was read: what a restore is
/// checked against.
 String? get currentRev;/// The version being looked at; the newest at first.
 String? get selectedRev; HistoryPane get pane;/// What changed from the selected version to the note now.
 List<DiffHunk> get hunks; LoadStatus get diffStatus;/// The section copied last, for its button to say so.
 int? get copiedSection;/// The selected version is being restored.
 bool get restoring;/// The version was restored: the screen is done.
 bool get restored;/// Why the last restore, or the last look at a version, did not happen.
 HistoryProblem? get problem;
/// Create a copy of NoteHistoryState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteHistoryStateCopyWith<NoteHistoryState> get copyWith => _$NoteHistoryStateCopyWithImpl<NoteHistoryState>(this as NoteHistoryState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteHistoryState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.history, history) || other.history == history)&&(identical(other.currentRev, currentRev) || other.currentRev == currentRev)&&(identical(other.selectedRev, selectedRev) || other.selectedRev == selectedRev)&&(identical(other.pane, pane) || other.pane == pane)&&const DeepCollectionEquality().equals(other.hunks, hunks)&&(identical(other.diffStatus, diffStatus) || other.diffStatus == diffStatus)&&(identical(other.copiedSection, copiedSection) || other.copiedSection == copiedSection)&&(identical(other.restoring, restoring) || other.restoring == restoring)&&(identical(other.restored, restored) || other.restored == restored)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,status,history,currentRev,selectedRev,pane,const DeepCollectionEquality().hash(hunks),diffStatus,copiedSection,restoring,restored,problem);

@override
String toString() {
  return 'NoteHistoryState(noteId: $noteId, status: $status, history: $history, currentRev: $currentRev, selectedRev: $selectedRev, pane: $pane, hunks: $hunks, diffStatus: $diffStatus, copiedSection: $copiedSection, restoring: $restoring, restored: $restored, problem: $problem)';
}


}

/// @nodoc
abstract mixin class $NoteHistoryStateCopyWith<$Res>  {
  factory $NoteHistoryStateCopyWith(NoteHistoryState value, $Res Function(NoteHistoryState) _then) = _$NoteHistoryStateCopyWithImpl;
@useResult
$Res call({
 String noteId, HistoryStatus status, NoteHistory? history, String? currentRev, String? selectedRev, HistoryPane pane, List<DiffHunk> hunks, LoadStatus diffStatus, int? copiedSection, bool restoring, bool restored, HistoryProblem? problem
});




}
/// @nodoc
class _$NoteHistoryStateCopyWithImpl<$Res>
    implements $NoteHistoryStateCopyWith<$Res> {
  _$NoteHistoryStateCopyWithImpl(this._self, this._then);

  final NoteHistoryState _self;
  final $Res Function(NoteHistoryState) _then;

/// Create a copy of NoteHistoryState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? status = null,Object? history = freezed,Object? currentRev = freezed,Object? selectedRev = freezed,Object? pane = null,Object? hunks = null,Object? diffStatus = null,Object? copiedSection = freezed,Object? restoring = null,Object? restored = null,Object? problem = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as HistoryStatus,history: freezed == history ? _self.history : history // ignore: cast_nullable_to_non_nullable
as NoteHistory?,currentRev: freezed == currentRev ? _self.currentRev : currentRev // ignore: cast_nullable_to_non_nullable
as String?,selectedRev: freezed == selectedRev ? _self.selectedRev : selectedRev // ignore: cast_nullable_to_non_nullable
as String?,pane: null == pane ? _self.pane : pane // ignore: cast_nullable_to_non_nullable
as HistoryPane,hunks: null == hunks ? _self.hunks : hunks // ignore: cast_nullable_to_non_nullable
as List<DiffHunk>,diffStatus: null == diffStatus ? _self.diffStatus : diffStatus // ignore: cast_nullable_to_non_nullable
as LoadStatus,copiedSection: freezed == copiedSection ? _self.copiedSection : copiedSection // ignore: cast_nullable_to_non_nullable
as int?,restoring: null == restoring ? _self.restoring : restoring // ignore: cast_nullable_to_non_nullable
as bool,restored: null == restored ? _self.restored : restored // ignore: cast_nullable_to_non_nullable
as bool,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as HistoryProblem?,
  ));
}

}


/// Adds pattern-matching-related methods to [NoteHistoryState].
extension NoteHistoryStatePatterns on NoteHistoryState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteHistoryState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteHistoryState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteHistoryState value)  $default,){
final _that = this;
switch (_that) {
case _NoteHistoryState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteHistoryState value)?  $default,){
final _that = this;
switch (_that) {
case _NoteHistoryState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  HistoryStatus status,  NoteHistory? history,  String? currentRev,  String? selectedRev,  HistoryPane pane,  List<DiffHunk> hunks,  LoadStatus diffStatus,  int? copiedSection,  bool restoring,  bool restored,  HistoryProblem? problem)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteHistoryState() when $default != null:
return $default(_that.noteId,_that.status,_that.history,_that.currentRev,_that.selectedRev,_that.pane,_that.hunks,_that.diffStatus,_that.copiedSection,_that.restoring,_that.restored,_that.problem);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  HistoryStatus status,  NoteHistory? history,  String? currentRev,  String? selectedRev,  HistoryPane pane,  List<DiffHunk> hunks,  LoadStatus diffStatus,  int? copiedSection,  bool restoring,  bool restored,  HistoryProblem? problem)  $default,) {final _that = this;
switch (_that) {
case _NoteHistoryState():
return $default(_that.noteId,_that.status,_that.history,_that.currentRev,_that.selectedRev,_that.pane,_that.hunks,_that.diffStatus,_that.copiedSection,_that.restoring,_that.restored,_that.problem);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  HistoryStatus status,  NoteHistory? history,  String? currentRev,  String? selectedRev,  HistoryPane pane,  List<DiffHunk> hunks,  LoadStatus diffStatus,  int? copiedSection,  bool restoring,  bool restored,  HistoryProblem? problem)?  $default,) {final _that = this;
switch (_that) {
case _NoteHistoryState() when $default != null:
return $default(_that.noteId,_that.status,_that.history,_that.currentRev,_that.selectedRev,_that.pane,_that.hunks,_that.diffStatus,_that.copiedSection,_that.restoring,_that.restored,_that.problem);case _:
  return null;

}
}

}

/// @nodoc


class _NoteHistoryState implements NoteHistoryState {
  const _NoteHistoryState({this.noteId = '', this.status = HistoryStatus.loading, this.history, this.currentRev, this.selectedRev, this.pane = HistoryPane.changes, final  List<DiffHunk> hunks = const <DiffHunk>[], this.diffStatus = LoadStatus.loading, this.copiedSection, this.restoring = false, this.restored = false, this.problem}): _hunks = hunks;
  

/// The note whose history this is.
@override@JsonKey() final  String noteId;
@override@JsonKey() final  HistoryStatus status;
/// The versions, newest first, beside the note as it is now.
@override final  NoteHistory? history;
/// The note's revision when the history was read: what a restore is
/// checked against.
@override final  String? currentRev;
/// The version being looked at; the newest at first.
@override final  String? selectedRev;
@override@JsonKey() final  HistoryPane pane;
/// What changed from the selected version to the note now.
 final  List<DiffHunk> _hunks;
/// What changed from the selected version to the note now.
@override@JsonKey() List<DiffHunk> get hunks {
  if (_hunks is EqualUnmodifiableListView) return _hunks;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_hunks);
}

@override@JsonKey() final  LoadStatus diffStatus;
/// The section copied last, for its button to say so.
@override final  int? copiedSection;
/// The selected version is being restored.
@override@JsonKey() final  bool restoring;
/// The version was restored: the screen is done.
@override@JsonKey() final  bool restored;
/// Why the last restore, or the last look at a version, did not happen.
@override final  HistoryProblem? problem;

/// Create a copy of NoteHistoryState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteHistoryStateCopyWith<_NoteHistoryState> get copyWith => __$NoteHistoryStateCopyWithImpl<_NoteHistoryState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteHistoryState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.history, history) || other.history == history)&&(identical(other.currentRev, currentRev) || other.currentRev == currentRev)&&(identical(other.selectedRev, selectedRev) || other.selectedRev == selectedRev)&&(identical(other.pane, pane) || other.pane == pane)&&const DeepCollectionEquality().equals(other._hunks, _hunks)&&(identical(other.diffStatus, diffStatus) || other.diffStatus == diffStatus)&&(identical(other.copiedSection, copiedSection) || other.copiedSection == copiedSection)&&(identical(other.restoring, restoring) || other.restoring == restoring)&&(identical(other.restored, restored) || other.restored == restored)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,status,history,currentRev,selectedRev,pane,const DeepCollectionEquality().hash(_hunks),diffStatus,copiedSection,restoring,restored,problem);

@override
String toString() {
  return 'NoteHistoryState(noteId: $noteId, status: $status, history: $history, currentRev: $currentRev, selectedRev: $selectedRev, pane: $pane, hunks: $hunks, diffStatus: $diffStatus, copiedSection: $copiedSection, restoring: $restoring, restored: $restored, problem: $problem)';
}


}

/// @nodoc
abstract mixin class _$NoteHistoryStateCopyWith<$Res> implements $NoteHistoryStateCopyWith<$Res> {
  factory _$NoteHistoryStateCopyWith(_NoteHistoryState value, $Res Function(_NoteHistoryState) _then) = __$NoteHistoryStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, HistoryStatus status, NoteHistory? history, String? currentRev, String? selectedRev, HistoryPane pane, List<DiffHunk> hunks, LoadStatus diffStatus, int? copiedSection, bool restoring, bool restored, HistoryProblem? problem
});




}
/// @nodoc
class __$NoteHistoryStateCopyWithImpl<$Res>
    implements _$NoteHistoryStateCopyWith<$Res> {
  __$NoteHistoryStateCopyWithImpl(this._self, this._then);

  final _NoteHistoryState _self;
  final $Res Function(_NoteHistoryState) _then;

/// Create a copy of NoteHistoryState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? status = null,Object? history = freezed,Object? currentRev = freezed,Object? selectedRev = freezed,Object? pane = null,Object? hunks = null,Object? diffStatus = null,Object? copiedSection = freezed,Object? restoring = null,Object? restored = null,Object? problem = freezed,}) {
  return _then(_NoteHistoryState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as HistoryStatus,history: freezed == history ? _self.history : history // ignore: cast_nullable_to_non_nullable
as NoteHistory?,currentRev: freezed == currentRev ? _self.currentRev : currentRev // ignore: cast_nullable_to_non_nullable
as String?,selectedRev: freezed == selectedRev ? _self.selectedRev : selectedRev // ignore: cast_nullable_to_non_nullable
as String?,pane: null == pane ? _self.pane : pane // ignore: cast_nullable_to_non_nullable
as HistoryPane,hunks: null == hunks ? _self._hunks : hunks // ignore: cast_nullable_to_non_nullable
as List<DiffHunk>,diffStatus: null == diffStatus ? _self.diffStatus : diffStatus // ignore: cast_nullable_to_non_nullable
as LoadStatus,copiedSection: freezed == copiedSection ? _self.copiedSection : copiedSection // ignore: cast_nullable_to_non_nullable
as int?,restoring: null == restoring ? _self.restoring : restoring // ignore: cast_nullable_to_non_nullable
as bool,restored: null == restored ? _self.restored : restored // ignore: cast_nullable_to_non_nullable
as bool,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as HistoryProblem?,
  ));
}


}

// dart format on
