// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'import_flow_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ImportFlowEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportFlowEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ImportFlowEvent()';
}


}

/// @nodoc
class $ImportFlowEventCopyWith<$Res>  {
$ImportFlowEventCopyWith(ImportFlowEvent _, $Res Function(ImportFlowEvent) __);
}


/// Adds pattern-matching-related methods to [ImportFlowEvent].
extension ImportFlowEventPatterns on ImportFlowEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ImportFlowStartedEvent value)?  started,TResult Function( ImportFlowSourceChosenEvent value)?  sourceChosen,TResult Function( ImportFlowConfirmedEvent value)?  confirmed,TResult Function( ImportFlowRetriedEvent value)?  retried,TResult Function( ImportFlowRestartedEvent value)?  restarted,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ImportFlowStartedEvent() when started != null:
return started(_that);case ImportFlowSourceChosenEvent() when sourceChosen != null:
return sourceChosen(_that);case ImportFlowConfirmedEvent() when confirmed != null:
return confirmed(_that);case ImportFlowRetriedEvent() when retried != null:
return retried(_that);case ImportFlowRestartedEvent() when restarted != null:
return restarted(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ImportFlowStartedEvent value)  started,required TResult Function( ImportFlowSourceChosenEvent value)  sourceChosen,required TResult Function( ImportFlowConfirmedEvent value)  confirmed,required TResult Function( ImportFlowRetriedEvent value)  retried,required TResult Function( ImportFlowRestartedEvent value)  restarted,}){
final _that = this;
switch (_that) {
case ImportFlowStartedEvent():
return started(_that);case ImportFlowSourceChosenEvent():
return sourceChosen(_that);case ImportFlowConfirmedEvent():
return confirmed(_that);case ImportFlowRetriedEvent():
return retried(_that);case ImportFlowRestartedEvent():
return restarted(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ImportFlowStartedEvent value)?  started,TResult? Function( ImportFlowSourceChosenEvent value)?  sourceChosen,TResult? Function( ImportFlowConfirmedEvent value)?  confirmed,TResult? Function( ImportFlowRetriedEvent value)?  retried,TResult? Function( ImportFlowRestartedEvent value)?  restarted,}){
final _that = this;
switch (_that) {
case ImportFlowStartedEvent() when started != null:
return started(_that);case ImportFlowSourceChosenEvent() when sourceChosen != null:
return sourceChosen(_that);case ImportFlowConfirmedEvent() when confirmed != null:
return confirmed(_that);case ImportFlowRetriedEvent() when retried != null:
return retried(_that);case ImportFlowRestartedEvent() when restarted != null:
return restarted(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String? path,  String? kind)?  started,TResult Function( String kind,  bool folder)?  sourceChosen,TResult Function()?  confirmed,TResult Function()?  retried,TResult Function()?  restarted,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ImportFlowStartedEvent() when started != null:
return started(_that.path,_that.kind);case ImportFlowSourceChosenEvent() when sourceChosen != null:
return sourceChosen(_that.kind,_that.folder);case ImportFlowConfirmedEvent() when confirmed != null:
return confirmed();case ImportFlowRetriedEvent() when retried != null:
return retried();case ImportFlowRestartedEvent() when restarted != null:
return restarted();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String? path,  String? kind)  started,required TResult Function( String kind,  bool folder)  sourceChosen,required TResult Function()  confirmed,required TResult Function()  retried,required TResult Function()  restarted,}) {final _that = this;
switch (_that) {
case ImportFlowStartedEvent():
return started(_that.path,_that.kind);case ImportFlowSourceChosenEvent():
return sourceChosen(_that.kind,_that.folder);case ImportFlowConfirmedEvent():
return confirmed();case ImportFlowRetriedEvent():
return retried();case ImportFlowRestartedEvent():
return restarted();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String? path,  String? kind)?  started,TResult? Function( String kind,  bool folder)?  sourceChosen,TResult? Function()?  confirmed,TResult? Function()?  retried,TResult? Function()?  restarted,}) {final _that = this;
switch (_that) {
case ImportFlowStartedEvent() when started != null:
return started(_that.path,_that.kind);case ImportFlowSourceChosenEvent() when sourceChosen != null:
return sourceChosen(_that.kind,_that.folder);case ImportFlowConfirmedEvent() when confirmed != null:
return confirmed();case ImportFlowRetriedEvent() when retried != null:
return retried();case ImportFlowRestartedEvent() when restarted != null:
return restarted();case _:
  return null;

}
}

}

/// @nodoc


class ImportFlowStartedEvent implements ImportFlowEvent {
  const ImportFlowStartedEvent({this.path, this.kind});
  

 final  String? path;
 final  String? kind;

/// Create a copy of ImportFlowEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ImportFlowStartedEventCopyWith<ImportFlowStartedEvent> get copyWith => _$ImportFlowStartedEventCopyWithImpl<ImportFlowStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportFlowStartedEvent&&(identical(other.path, path) || other.path == path)&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,path,kind);

@override
String toString() {
  return 'ImportFlowEvent.started(path: $path, kind: $kind)';
}


}

/// @nodoc
abstract mixin class $ImportFlowStartedEventCopyWith<$Res> implements $ImportFlowEventCopyWith<$Res> {
  factory $ImportFlowStartedEventCopyWith(ImportFlowStartedEvent value, $Res Function(ImportFlowStartedEvent) _then) = _$ImportFlowStartedEventCopyWithImpl;
@useResult
$Res call({
 String? path, String? kind
});




}
/// @nodoc
class _$ImportFlowStartedEventCopyWithImpl<$Res>
    implements $ImportFlowStartedEventCopyWith<$Res> {
  _$ImportFlowStartedEventCopyWithImpl(this._self, this._then);

  final ImportFlowStartedEvent _self;
  final $Res Function(ImportFlowStartedEvent) _then;

/// Create a copy of ImportFlowEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? path = freezed,Object? kind = freezed,}) {
  return _then(ImportFlowStartedEvent(
path: freezed == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String?,kind: freezed == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class ImportFlowSourceChosenEvent implements ImportFlowEvent {
  const ImportFlowSourceChosenEvent(this.kind, {this.folder = false});
  

 final  String kind;
@JsonKey() final  bool folder;

/// Create a copy of ImportFlowEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ImportFlowSourceChosenEventCopyWith<ImportFlowSourceChosenEvent> get copyWith => _$ImportFlowSourceChosenEventCopyWithImpl<ImportFlowSourceChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportFlowSourceChosenEvent&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.folder, folder) || other.folder == folder));
}


@override
int get hashCode => Object.hash(runtimeType,kind,folder);

@override
String toString() {
  return 'ImportFlowEvent.sourceChosen(kind: $kind, folder: $folder)';
}


}

/// @nodoc
abstract mixin class $ImportFlowSourceChosenEventCopyWith<$Res> implements $ImportFlowEventCopyWith<$Res> {
  factory $ImportFlowSourceChosenEventCopyWith(ImportFlowSourceChosenEvent value, $Res Function(ImportFlowSourceChosenEvent) _then) = _$ImportFlowSourceChosenEventCopyWithImpl;
@useResult
$Res call({
 String kind, bool folder
});




}
/// @nodoc
class _$ImportFlowSourceChosenEventCopyWithImpl<$Res>
    implements $ImportFlowSourceChosenEventCopyWith<$Res> {
  _$ImportFlowSourceChosenEventCopyWithImpl(this._self, this._then);

  final ImportFlowSourceChosenEvent _self;
  final $Res Function(ImportFlowSourceChosenEvent) _then;

/// Create a copy of ImportFlowEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? kind = null,Object? folder = null,}) {
  return _then(ImportFlowSourceChosenEvent(
null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as String,folder: null == folder ? _self.folder : folder // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class ImportFlowConfirmedEvent implements ImportFlowEvent {
  const ImportFlowConfirmedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportFlowConfirmedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ImportFlowEvent.confirmed()';
}


}




/// @nodoc


class ImportFlowRetriedEvent implements ImportFlowEvent {
  const ImportFlowRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportFlowRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ImportFlowEvent.retried()';
}


}




/// @nodoc


class ImportFlowRestartedEvent implements ImportFlowEvent {
  const ImportFlowRestartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportFlowRestartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ImportFlowEvent.restarted()';
}


}




/// @nodoc
mixin _$ImportFlowState {

 ImportStage get stage;/// Every source a phone can import from, in the order to list them.
 List<ImportSource> get sources;/// The kinds a given file could be, best first; empty when the list is
/// not narrowed.
 List<String> get only;/// The source being read or imported.
 ImportSource? get source;/// What is being read or imported.
 ImportPick? get pick;/// Whether [pick] is a copy the pickers made (and so is removed when the
/// flow ends) and not a file somebody else handed over.
 bool get picked;/// The system's picker is open.
 bool get picking;/// How far reading or importing has got.
 ImportProgress? get progress;/// What the export holds ([ImportStage.summary]) or what the import did
/// ([ImportStage.done]).
 ImportSummary? get summary;/// Why a step did not happen.
 ImportProblem? get problem;
/// Create a copy of ImportFlowState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ImportFlowStateCopyWith<ImportFlowState> get copyWith => _$ImportFlowStateCopyWithImpl<ImportFlowState>(this as ImportFlowState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportFlowState&&(identical(other.stage, stage) || other.stage == stage)&&const DeepCollectionEquality().equals(other.sources, sources)&&const DeepCollectionEquality().equals(other.only, only)&&(identical(other.source, source) || other.source == source)&&(identical(other.pick, pick) || other.pick == pick)&&(identical(other.picked, picked) || other.picked == picked)&&(identical(other.picking, picking) || other.picking == picking)&&(identical(other.progress, progress) || other.progress == progress)&&(identical(other.summary, summary) || other.summary == summary)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,stage,const DeepCollectionEquality().hash(sources),const DeepCollectionEquality().hash(only),source,pick,picked,picking,progress,summary,problem);

@override
String toString() {
  return 'ImportFlowState(stage: $stage, sources: $sources, only: $only, source: $source, pick: $pick, picked: $picked, picking: $picking, progress: $progress, summary: $summary, problem: $problem)';
}


}

/// @nodoc
abstract mixin class $ImportFlowStateCopyWith<$Res>  {
  factory $ImportFlowStateCopyWith(ImportFlowState value, $Res Function(ImportFlowState) _then) = _$ImportFlowStateCopyWithImpl;
@useResult
$Res call({
 ImportStage stage, List<ImportSource> sources, List<String> only, ImportSource? source, ImportPick? pick, bool picked, bool picking, ImportProgress? progress, ImportSummary? summary, ImportProblem? problem
});


$ImportPickCopyWith<$Res>? get pick;$ImportProblemCopyWith<$Res>? get problem;

}
/// @nodoc
class _$ImportFlowStateCopyWithImpl<$Res>
    implements $ImportFlowStateCopyWith<$Res> {
  _$ImportFlowStateCopyWithImpl(this._self, this._then);

  final ImportFlowState _self;
  final $Res Function(ImportFlowState) _then;

/// Create a copy of ImportFlowState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? stage = null,Object? sources = null,Object? only = null,Object? source = freezed,Object? pick = freezed,Object? picked = null,Object? picking = null,Object? progress = freezed,Object? summary = freezed,Object? problem = freezed,}) {
  return _then(_self.copyWith(
stage: null == stage ? _self.stage : stage // ignore: cast_nullable_to_non_nullable
as ImportStage,sources: null == sources ? _self.sources : sources // ignore: cast_nullable_to_non_nullable
as List<ImportSource>,only: null == only ? _self.only : only // ignore: cast_nullable_to_non_nullable
as List<String>,source: freezed == source ? _self.source : source // ignore: cast_nullable_to_non_nullable
as ImportSource?,pick: freezed == pick ? _self.pick : pick // ignore: cast_nullable_to_non_nullable
as ImportPick?,picked: null == picked ? _self.picked : picked // ignore: cast_nullable_to_non_nullable
as bool,picking: null == picking ? _self.picking : picking // ignore: cast_nullable_to_non_nullable
as bool,progress: freezed == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as ImportProgress?,summary: freezed == summary ? _self.summary : summary // ignore: cast_nullable_to_non_nullable
as ImportSummary?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ImportProblem?,
  ));
}
/// Create a copy of ImportFlowState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ImportPickCopyWith<$Res>? get pick {
    if (_self.pick == null) {
    return null;
  }

  return $ImportPickCopyWith<$Res>(_self.pick!, (value) {
    return _then(_self.copyWith(pick: value));
  });
}/// Create a copy of ImportFlowState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ImportProblemCopyWith<$Res>? get problem {
    if (_self.problem == null) {
    return null;
  }

  return $ImportProblemCopyWith<$Res>(_self.problem!, (value) {
    return _then(_self.copyWith(problem: value));
  });
}
}


/// Adds pattern-matching-related methods to [ImportFlowState].
extension ImportFlowStatePatterns on ImportFlowState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ImportFlowState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ImportFlowState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ImportFlowState value)  $default,){
final _that = this;
switch (_that) {
case _ImportFlowState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ImportFlowState value)?  $default,){
final _that = this;
switch (_that) {
case _ImportFlowState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( ImportStage stage,  List<ImportSource> sources,  List<String> only,  ImportSource? source,  ImportPick? pick,  bool picked,  bool picking,  ImportProgress? progress,  ImportSummary? summary,  ImportProblem? problem)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ImportFlowState() when $default != null:
return $default(_that.stage,_that.sources,_that.only,_that.source,_that.pick,_that.picked,_that.picking,_that.progress,_that.summary,_that.problem);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( ImportStage stage,  List<ImportSource> sources,  List<String> only,  ImportSource? source,  ImportPick? pick,  bool picked,  bool picking,  ImportProgress? progress,  ImportSummary? summary,  ImportProblem? problem)  $default,) {final _that = this;
switch (_that) {
case _ImportFlowState():
return $default(_that.stage,_that.sources,_that.only,_that.source,_that.pick,_that.picked,_that.picking,_that.progress,_that.summary,_that.problem);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( ImportStage stage,  List<ImportSource> sources,  List<String> only,  ImportSource? source,  ImportPick? pick,  bool picked,  bool picking,  ImportProgress? progress,  ImportSummary? summary,  ImportProblem? problem)?  $default,) {final _that = this;
switch (_that) {
case _ImportFlowState() when $default != null:
return $default(_that.stage,_that.sources,_that.only,_that.source,_that.pick,_that.picked,_that.picking,_that.progress,_that.summary,_that.problem);case _:
  return null;

}
}

}

/// @nodoc


class _ImportFlowState extends ImportFlowState {
  const _ImportFlowState({this.stage = ImportStage.sources, final  List<ImportSource> sources = const <ImportSource>[], final  List<String> only = const <String>[], this.source, this.pick, this.picked = false, this.picking = false, this.progress, this.summary, this.problem}): _sources = sources,_only = only,super._();
  

@override@JsonKey() final  ImportStage stage;
/// Every source a phone can import from, in the order to list them.
 final  List<ImportSource> _sources;
/// Every source a phone can import from, in the order to list them.
@override@JsonKey() List<ImportSource> get sources {
  if (_sources is EqualUnmodifiableListView) return _sources;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_sources);
}

/// The kinds a given file could be, best first; empty when the list is
/// not narrowed.
 final  List<String> _only;
/// The kinds a given file could be, best first; empty when the list is
/// not narrowed.
@override@JsonKey() List<String> get only {
  if (_only is EqualUnmodifiableListView) return _only;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_only);
}

/// The source being read or imported.
@override final  ImportSource? source;
/// What is being read or imported.
@override final  ImportPick? pick;
/// Whether [pick] is a copy the pickers made (and so is removed when the
/// flow ends) and not a file somebody else handed over.
@override@JsonKey() final  bool picked;
/// The system's picker is open.
@override@JsonKey() final  bool picking;
/// How far reading or importing has got.
@override final  ImportProgress? progress;
/// What the export holds ([ImportStage.summary]) or what the import did
/// ([ImportStage.done]).
@override final  ImportSummary? summary;
/// Why a step did not happen.
@override final  ImportProblem? problem;

/// Create a copy of ImportFlowState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ImportFlowStateCopyWith<_ImportFlowState> get copyWith => __$ImportFlowStateCopyWithImpl<_ImportFlowState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ImportFlowState&&(identical(other.stage, stage) || other.stage == stage)&&const DeepCollectionEquality().equals(other._sources, _sources)&&const DeepCollectionEquality().equals(other._only, _only)&&(identical(other.source, source) || other.source == source)&&(identical(other.pick, pick) || other.pick == pick)&&(identical(other.picked, picked) || other.picked == picked)&&(identical(other.picking, picking) || other.picking == picking)&&(identical(other.progress, progress) || other.progress == progress)&&(identical(other.summary, summary) || other.summary == summary)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,stage,const DeepCollectionEquality().hash(_sources),const DeepCollectionEquality().hash(_only),source,pick,picked,picking,progress,summary,problem);

@override
String toString() {
  return 'ImportFlowState(stage: $stage, sources: $sources, only: $only, source: $source, pick: $pick, picked: $picked, picking: $picking, progress: $progress, summary: $summary, problem: $problem)';
}


}

/// @nodoc
abstract mixin class _$ImportFlowStateCopyWith<$Res> implements $ImportFlowStateCopyWith<$Res> {
  factory _$ImportFlowStateCopyWith(_ImportFlowState value, $Res Function(_ImportFlowState) _then) = __$ImportFlowStateCopyWithImpl;
@override @useResult
$Res call({
 ImportStage stage, List<ImportSource> sources, List<String> only, ImportSource? source, ImportPick? pick, bool picked, bool picking, ImportProgress? progress, ImportSummary? summary, ImportProblem? problem
});


@override $ImportPickCopyWith<$Res>? get pick;@override $ImportProblemCopyWith<$Res>? get problem;

}
/// @nodoc
class __$ImportFlowStateCopyWithImpl<$Res>
    implements _$ImportFlowStateCopyWith<$Res> {
  __$ImportFlowStateCopyWithImpl(this._self, this._then);

  final _ImportFlowState _self;
  final $Res Function(_ImportFlowState) _then;

/// Create a copy of ImportFlowState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? stage = null,Object? sources = null,Object? only = null,Object? source = freezed,Object? pick = freezed,Object? picked = null,Object? picking = null,Object? progress = freezed,Object? summary = freezed,Object? problem = freezed,}) {
  return _then(_ImportFlowState(
stage: null == stage ? _self.stage : stage // ignore: cast_nullable_to_non_nullable
as ImportStage,sources: null == sources ? _self._sources : sources // ignore: cast_nullable_to_non_nullable
as List<ImportSource>,only: null == only ? _self._only : only // ignore: cast_nullable_to_non_nullable
as List<String>,source: freezed == source ? _self.source : source // ignore: cast_nullable_to_non_nullable
as ImportSource?,pick: freezed == pick ? _self.pick : pick // ignore: cast_nullable_to_non_nullable
as ImportPick?,picked: null == picked ? _self.picked : picked // ignore: cast_nullable_to_non_nullable
as bool,picking: null == picking ? _self.picking : picking // ignore: cast_nullable_to_non_nullable
as bool,progress: freezed == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as ImportProgress?,summary: freezed == summary ? _self.summary : summary // ignore: cast_nullable_to_non_nullable
as ImportSummary?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ImportProblem?,
  ));
}

/// Create a copy of ImportFlowState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ImportPickCopyWith<$Res>? get pick {
    if (_self.pick == null) {
    return null;
  }

  return $ImportPickCopyWith<$Res>(_self.pick!, (value) {
    return _then(_self.copyWith(pick: value));
  });
}/// Create a copy of ImportFlowState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ImportProblemCopyWith<$Res>? get problem {
    if (_self.problem == null) {
    return null;
  }

  return $ImportProblemCopyWith<$Res>(_self.problem!, (value) {
    return _then(_self.copyWith(problem: value));
  });
}
}

// dart format on
