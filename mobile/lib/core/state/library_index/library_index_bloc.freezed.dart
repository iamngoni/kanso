// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'library_index_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$LibraryIndexEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LibraryIndexEvent()';
}


}

/// @nodoc
class $LibraryIndexEventCopyWith<$Res>  {
$LibraryIndexEventCopyWith(LibraryIndexEvent _, $Res Function(LibraryIndexEvent) __);
}


/// Adds pattern-matching-related methods to [LibraryIndexEvent].
extension LibraryIndexEventPatterns on LibraryIndexEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LibraryIndexStartedEvent value)?  started,TResult Function( LibraryIndexChangesReceivedEvent value)?  changesReceived,TResult Function( LibraryIndexRetriedEvent value)?  retried,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LibraryIndexStartedEvent() when started != null:
return started(_that);case LibraryIndexChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case LibraryIndexRetriedEvent() when retried != null:
return retried(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LibraryIndexStartedEvent value)  started,required TResult Function( LibraryIndexChangesReceivedEvent value)  changesReceived,required TResult Function( LibraryIndexRetriedEvent value)  retried,}){
final _that = this;
switch (_that) {
case LibraryIndexStartedEvent():
return started(_that);case LibraryIndexChangesReceivedEvent():
return changesReceived(_that);case LibraryIndexRetriedEvent():
return retried(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LibraryIndexStartedEvent value)?  started,TResult? Function( LibraryIndexChangesReceivedEvent value)?  changesReceived,TResult? Function( LibraryIndexRetriedEvent value)?  retried,}){
final _that = this;
switch (_that) {
case LibraryIndexStartedEvent() when started != null:
return started(_that);case LibraryIndexChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case LibraryIndexRetriedEvent() when retried != null:
return retried(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( List<String> ids)?  changesReceived,TResult Function()?  retried,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LibraryIndexStartedEvent() when started != null:
return started();case LibraryIndexChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case LibraryIndexRetriedEvent() when retried != null:
return retried();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( List<String> ids)  changesReceived,required TResult Function()  retried,}) {final _that = this;
switch (_that) {
case LibraryIndexStartedEvent():
return started();case LibraryIndexChangesReceivedEvent():
return changesReceived(_that.ids);case LibraryIndexRetriedEvent():
return retried();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( List<String> ids)?  changesReceived,TResult? Function()?  retried,}) {final _that = this;
switch (_that) {
case LibraryIndexStartedEvent() when started != null:
return started();case LibraryIndexChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case LibraryIndexRetriedEvent() when retried != null:
return retried();case _:
  return null;

}
}

}

/// @nodoc


class LibraryIndexStartedEvent implements LibraryIndexEvent {
  const LibraryIndexStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LibraryIndexEvent.started()';
}


}




/// @nodoc


class LibraryIndexChangesReceivedEvent implements LibraryIndexEvent {
  const LibraryIndexChangesReceivedEvent(final  List<String> ids): _ids = ids;
  

 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of LibraryIndexEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LibraryIndexChangesReceivedEventCopyWith<LibraryIndexChangesReceivedEvent> get copyWith => _$LibraryIndexChangesReceivedEventCopyWithImpl<LibraryIndexChangesReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexChangesReceivedEvent&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'LibraryIndexEvent.changesReceived(ids: $ids)';
}


}

/// @nodoc
abstract mixin class $LibraryIndexChangesReceivedEventCopyWith<$Res> implements $LibraryIndexEventCopyWith<$Res> {
  factory $LibraryIndexChangesReceivedEventCopyWith(LibraryIndexChangesReceivedEvent value, $Res Function(LibraryIndexChangesReceivedEvent) _then) = _$LibraryIndexChangesReceivedEventCopyWithImpl;
@useResult
$Res call({
 List<String> ids
});




}
/// @nodoc
class _$LibraryIndexChangesReceivedEventCopyWithImpl<$Res>
    implements $LibraryIndexChangesReceivedEventCopyWith<$Res> {
  _$LibraryIndexChangesReceivedEventCopyWithImpl(this._self, this._then);

  final LibraryIndexChangesReceivedEvent _self;
  final $Res Function(LibraryIndexChangesReceivedEvent) _then;

/// Create a copy of LibraryIndexEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ids = null,}) {
  return _then(LibraryIndexChangesReceivedEvent(
null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class LibraryIndexRetriedEvent implements LibraryIndexEvent {
  const LibraryIndexRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LibraryIndexEvent.retried()';
}


}




/// @nodoc
mixin _$LibraryIndexState {

 LibraryIndexStatus get status;/// The notebooks in tree order, each with its depth and role.
 List<NotebookRow> get notebooks; List<TagRow> get tags;/// How many notes each place lists.
 LibraryOverview get overview;/// How many open tasks are overdue: the badge of the Tasks row.
 int get overdueTasks;/// Why the last reading failed, while [status] is failed.
 ApplicationError? get error;
/// Create a copy of LibraryIndexState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LibraryIndexStateCopyWith<LibraryIndexState> get copyWith => _$LibraryIndexStateCopyWithImpl<LibraryIndexState>(this as LibraryIndexState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.notebooks, notebooks)&&const DeepCollectionEquality().equals(other.tags, tags)&&(identical(other.overview, overview) || other.overview == overview)&&(identical(other.overdueTasks, overdueTasks) || other.overdueTasks == overdueTasks)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(notebooks),const DeepCollectionEquality().hash(tags),overview,overdueTasks,error);

@override
String toString() {
  return 'LibraryIndexState(status: $status, notebooks: $notebooks, tags: $tags, overview: $overview, overdueTasks: $overdueTasks, error: $error)';
}


}

/// @nodoc
abstract mixin class $LibraryIndexStateCopyWith<$Res>  {
  factory $LibraryIndexStateCopyWith(LibraryIndexState value, $Res Function(LibraryIndexState) _then) = _$LibraryIndexStateCopyWithImpl;
@useResult
$Res call({
 LibraryIndexStatus status, List<NotebookRow> notebooks, List<TagRow> tags, LibraryOverview overview, int overdueTasks, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$LibraryIndexStateCopyWithImpl<$Res>
    implements $LibraryIndexStateCopyWith<$Res> {
  _$LibraryIndexStateCopyWithImpl(this._self, this._then);

  final LibraryIndexState _self;
  final $Res Function(LibraryIndexState) _then;

/// Create a copy of LibraryIndexState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? notebooks = null,Object? tags = null,Object? overview = null,Object? overdueTasks = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LibraryIndexStatus,notebooks: null == notebooks ? _self.notebooks : notebooks // ignore: cast_nullable_to_non_nullable
as List<NotebookRow>,tags: null == tags ? _self.tags : tags // ignore: cast_nullable_to_non_nullable
as List<TagRow>,overview: null == overview ? _self.overview : overview // ignore: cast_nullable_to_non_nullable
as LibraryOverview,overdueTasks: null == overdueTasks ? _self.overdueTasks : overdueTasks // ignore: cast_nullable_to_non_nullable
as int,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of LibraryIndexState
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


/// Adds pattern-matching-related methods to [LibraryIndexState].
extension LibraryIndexStatePatterns on LibraryIndexState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _LibraryIndexState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _LibraryIndexState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _LibraryIndexState value)  $default,){
final _that = this;
switch (_that) {
case _LibraryIndexState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _LibraryIndexState value)?  $default,){
final _that = this;
switch (_that) {
case _LibraryIndexState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LibraryIndexStatus status,  List<NotebookRow> notebooks,  List<TagRow> tags,  LibraryOverview overview,  int overdueTasks,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _LibraryIndexState() when $default != null:
return $default(_that.status,_that.notebooks,_that.tags,_that.overview,_that.overdueTasks,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LibraryIndexStatus status,  List<NotebookRow> notebooks,  List<TagRow> tags,  LibraryOverview overview,  int overdueTasks,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _LibraryIndexState():
return $default(_that.status,_that.notebooks,_that.tags,_that.overview,_that.overdueTasks,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LibraryIndexStatus status,  List<NotebookRow> notebooks,  List<TagRow> tags,  LibraryOverview overview,  int overdueTasks,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _LibraryIndexState() when $default != null:
return $default(_that.status,_that.notebooks,_that.tags,_that.overview,_that.overdueTasks,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _LibraryIndexState extends LibraryIndexState {
  const _LibraryIndexState({this.status = LibraryIndexStatus.loading, final  List<NotebookRow> notebooks = const <NotebookRow>[], final  List<TagRow> tags = const <TagRow>[], this.overview = const LibraryOverview(all: 0, pinned: 0, recent: 0, trash: 0), this.overdueTasks = 0, this.error}): _notebooks = notebooks,_tags = tags,super._();
  

@override@JsonKey() final  LibraryIndexStatus status;
/// The notebooks in tree order, each with its depth and role.
 final  List<NotebookRow> _notebooks;
/// The notebooks in tree order, each with its depth and role.
@override@JsonKey() List<NotebookRow> get notebooks {
  if (_notebooks is EqualUnmodifiableListView) return _notebooks;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_notebooks);
}

 final  List<TagRow> _tags;
@override@JsonKey() List<TagRow> get tags {
  if (_tags is EqualUnmodifiableListView) return _tags;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_tags);
}

/// How many notes each place lists.
@override@JsonKey() final  LibraryOverview overview;
/// How many open tasks are overdue: the badge of the Tasks row.
@override@JsonKey() final  int overdueTasks;
/// Why the last reading failed, while [status] is failed.
@override final  ApplicationError? error;

/// Create a copy of LibraryIndexState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$LibraryIndexStateCopyWith<_LibraryIndexState> get copyWith => __$LibraryIndexStateCopyWithImpl<_LibraryIndexState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _LibraryIndexState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._notebooks, _notebooks)&&const DeepCollectionEquality().equals(other._tags, _tags)&&(identical(other.overview, overview) || other.overview == overview)&&(identical(other.overdueTasks, overdueTasks) || other.overdueTasks == overdueTasks)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_notebooks),const DeepCollectionEquality().hash(_tags),overview,overdueTasks,error);

@override
String toString() {
  return 'LibraryIndexState(status: $status, notebooks: $notebooks, tags: $tags, overview: $overview, overdueTasks: $overdueTasks, error: $error)';
}


}

/// @nodoc
abstract mixin class _$LibraryIndexStateCopyWith<$Res> implements $LibraryIndexStateCopyWith<$Res> {
  factory _$LibraryIndexStateCopyWith(_LibraryIndexState value, $Res Function(_LibraryIndexState) _then) = __$LibraryIndexStateCopyWithImpl;
@override @useResult
$Res call({
 LibraryIndexStatus status, List<NotebookRow> notebooks, List<TagRow> tags, LibraryOverview overview, int overdueTasks, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$LibraryIndexStateCopyWithImpl<$Res>
    implements _$LibraryIndexStateCopyWith<$Res> {
  __$LibraryIndexStateCopyWithImpl(this._self, this._then);

  final _LibraryIndexState _self;
  final $Res Function(_LibraryIndexState) _then;

/// Create a copy of LibraryIndexState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? notebooks = null,Object? tags = null,Object? overview = null,Object? overdueTasks = null,Object? error = freezed,}) {
  return _then(_LibraryIndexState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LibraryIndexStatus,notebooks: null == notebooks ? _self._notebooks : notebooks // ignore: cast_nullable_to_non_nullable
as List<NotebookRow>,tags: null == tags ? _self._tags : tags // ignore: cast_nullable_to_non_nullable
as List<TagRow>,overview: null == overview ? _self.overview : overview // ignore: cast_nullable_to_non_nullable
as LibraryOverview,overdueTasks: null == overdueTasks ? _self.overdueTasks : overdueTasks // ignore: cast_nullable_to_non_nullable
as int,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of LibraryIndexState
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
