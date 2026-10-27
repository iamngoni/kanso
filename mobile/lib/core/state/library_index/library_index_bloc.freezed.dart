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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LibraryIndexStartedEvent value)?  started,TResult Function( LibraryIndexChangesReceivedEvent value)?  changesReceived,TResult Function( LibraryIndexRetriedEvent value)?  retried,TResult Function( LibraryIndexPlaceChangedEvent value)?  placeChanged,TResult Function( LibraryIndexNotebookFoldedEvent value)?  notebookFolded,TResult Function( LibraryIndexTagToggledEvent value)?  tagToggled,TResult Function( LibraryIndexAllTagsToggledEvent value)?  allTagsToggled,TResult Function( LibraryIndexStatusGroupToggledEvent value)?  statusGroupToggled,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LibraryIndexStartedEvent() when started != null:
return started(_that);case LibraryIndexChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case LibraryIndexRetriedEvent() when retried != null:
return retried(_that);case LibraryIndexPlaceChangedEvent() when placeChanged != null:
return placeChanged(_that);case LibraryIndexNotebookFoldedEvent() when notebookFolded != null:
return notebookFolded(_that);case LibraryIndexTagToggledEvent() when tagToggled != null:
return tagToggled(_that);case LibraryIndexAllTagsToggledEvent() when allTagsToggled != null:
return allTagsToggled(_that);case LibraryIndexStatusGroupToggledEvent() when statusGroupToggled != null:
return statusGroupToggled(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LibraryIndexStartedEvent value)  started,required TResult Function( LibraryIndexChangesReceivedEvent value)  changesReceived,required TResult Function( LibraryIndexRetriedEvent value)  retried,required TResult Function( LibraryIndexPlaceChangedEvent value)  placeChanged,required TResult Function( LibraryIndexNotebookFoldedEvent value)  notebookFolded,required TResult Function( LibraryIndexTagToggledEvent value)  tagToggled,required TResult Function( LibraryIndexAllTagsToggledEvent value)  allTagsToggled,required TResult Function( LibraryIndexStatusGroupToggledEvent value)  statusGroupToggled,}){
final _that = this;
switch (_that) {
case LibraryIndexStartedEvent():
return started(_that);case LibraryIndexChangesReceivedEvent():
return changesReceived(_that);case LibraryIndexRetriedEvent():
return retried(_that);case LibraryIndexPlaceChangedEvent():
return placeChanged(_that);case LibraryIndexNotebookFoldedEvent():
return notebookFolded(_that);case LibraryIndexTagToggledEvent():
return tagToggled(_that);case LibraryIndexAllTagsToggledEvent():
return allTagsToggled(_that);case LibraryIndexStatusGroupToggledEvent():
return statusGroupToggled(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LibraryIndexStartedEvent value)?  started,TResult? Function( LibraryIndexChangesReceivedEvent value)?  changesReceived,TResult? Function( LibraryIndexRetriedEvent value)?  retried,TResult? Function( LibraryIndexPlaceChangedEvent value)?  placeChanged,TResult? Function( LibraryIndexNotebookFoldedEvent value)?  notebookFolded,TResult? Function( LibraryIndexTagToggledEvent value)?  tagToggled,TResult? Function( LibraryIndexAllTagsToggledEvent value)?  allTagsToggled,TResult? Function( LibraryIndexStatusGroupToggledEvent value)?  statusGroupToggled,}){
final _that = this;
switch (_that) {
case LibraryIndexStartedEvent() when started != null:
return started(_that);case LibraryIndexChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case LibraryIndexRetriedEvent() when retried != null:
return retried(_that);case LibraryIndexPlaceChangedEvent() when placeChanged != null:
return placeChanged(_that);case LibraryIndexNotebookFoldedEvent() when notebookFolded != null:
return notebookFolded(_that);case LibraryIndexTagToggledEvent() when tagToggled != null:
return tagToggled(_that);case LibraryIndexAllTagsToggledEvent() when allTagsToggled != null:
return allTagsToggled(_that);case LibraryIndexStatusGroupToggledEvent() when statusGroupToggled != null:
return statusGroupToggled(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( List<String> ids)?  changesReceived,TResult Function()?  retried,TResult Function( ListPlace? place)?  placeChanged,TResult Function( String id)?  notebookFolded,TResult Function( String path)?  tagToggled,TResult Function()?  allTagsToggled,TResult Function()?  statusGroupToggled,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LibraryIndexStartedEvent() when started != null:
return started();case LibraryIndexChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case LibraryIndexRetriedEvent() when retried != null:
return retried();case LibraryIndexPlaceChangedEvent() when placeChanged != null:
return placeChanged(_that.place);case LibraryIndexNotebookFoldedEvent() when notebookFolded != null:
return notebookFolded(_that.id);case LibraryIndexTagToggledEvent() when tagToggled != null:
return tagToggled(_that.path);case LibraryIndexAllTagsToggledEvent() when allTagsToggled != null:
return allTagsToggled();case LibraryIndexStatusGroupToggledEvent() when statusGroupToggled != null:
return statusGroupToggled();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( List<String> ids)  changesReceived,required TResult Function()  retried,required TResult Function( ListPlace? place)  placeChanged,required TResult Function( String id)  notebookFolded,required TResult Function( String path)  tagToggled,required TResult Function()  allTagsToggled,required TResult Function()  statusGroupToggled,}) {final _that = this;
switch (_that) {
case LibraryIndexStartedEvent():
return started();case LibraryIndexChangesReceivedEvent():
return changesReceived(_that.ids);case LibraryIndexRetriedEvent():
return retried();case LibraryIndexPlaceChangedEvent():
return placeChanged(_that.place);case LibraryIndexNotebookFoldedEvent():
return notebookFolded(_that.id);case LibraryIndexTagToggledEvent():
return tagToggled(_that.path);case LibraryIndexAllTagsToggledEvent():
return allTagsToggled();case LibraryIndexStatusGroupToggledEvent():
return statusGroupToggled();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( List<String> ids)?  changesReceived,TResult? Function()?  retried,TResult? Function( ListPlace? place)?  placeChanged,TResult? Function( String id)?  notebookFolded,TResult? Function( String path)?  tagToggled,TResult? Function()?  allTagsToggled,TResult? Function()?  statusGroupToggled,}) {final _that = this;
switch (_that) {
case LibraryIndexStartedEvent() when started != null:
return started();case LibraryIndexChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case LibraryIndexRetriedEvent() when retried != null:
return retried();case LibraryIndexPlaceChangedEvent() when placeChanged != null:
return placeChanged(_that.place);case LibraryIndexNotebookFoldedEvent() when notebookFolded != null:
return notebookFolded(_that.id);case LibraryIndexTagToggledEvent() when tagToggled != null:
return tagToggled(_that.path);case LibraryIndexAllTagsToggledEvent() when allTagsToggled != null:
return allTagsToggled();case LibraryIndexStatusGroupToggledEvent() when statusGroupToggled != null:
return statusGroupToggled();case _:
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


class LibraryIndexPlaceChangedEvent implements LibraryIndexEvent {
  const LibraryIndexPlaceChangedEvent(this.place);
  

 final  ListPlace? place;

/// Create a copy of LibraryIndexEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LibraryIndexPlaceChangedEventCopyWith<LibraryIndexPlaceChangedEvent> get copyWith => _$LibraryIndexPlaceChangedEventCopyWithImpl<LibraryIndexPlaceChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexPlaceChangedEvent&&(identical(other.place, place) || other.place == place));
}


@override
int get hashCode => Object.hash(runtimeType,place);

@override
String toString() {
  return 'LibraryIndexEvent.placeChanged(place: $place)';
}


}

/// @nodoc
abstract mixin class $LibraryIndexPlaceChangedEventCopyWith<$Res> implements $LibraryIndexEventCopyWith<$Res> {
  factory $LibraryIndexPlaceChangedEventCopyWith(LibraryIndexPlaceChangedEvent value, $Res Function(LibraryIndexPlaceChangedEvent) _then) = _$LibraryIndexPlaceChangedEventCopyWithImpl;
@useResult
$Res call({
 ListPlace? place
});


$ListPlaceCopyWith<$Res>? get place;

}
/// @nodoc
class _$LibraryIndexPlaceChangedEventCopyWithImpl<$Res>
    implements $LibraryIndexPlaceChangedEventCopyWith<$Res> {
  _$LibraryIndexPlaceChangedEventCopyWithImpl(this._self, this._then);

  final LibraryIndexPlaceChangedEvent _self;
  final $Res Function(LibraryIndexPlaceChangedEvent) _then;

/// Create a copy of LibraryIndexEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? place = freezed,}) {
  return _then(LibraryIndexPlaceChangedEvent(
freezed == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as ListPlace?,
  ));
}

/// Create a copy of LibraryIndexEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListPlaceCopyWith<$Res>? get place {
    if (_self.place == null) {
    return null;
  }

  return $ListPlaceCopyWith<$Res>(_self.place!, (value) {
    return _then(_self.copyWith(place: value));
  });
}
}

/// @nodoc


class LibraryIndexNotebookFoldedEvent implements LibraryIndexEvent {
  const LibraryIndexNotebookFoldedEvent(this.id);
  

 final  String id;

/// Create a copy of LibraryIndexEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LibraryIndexNotebookFoldedEventCopyWith<LibraryIndexNotebookFoldedEvent> get copyWith => _$LibraryIndexNotebookFoldedEventCopyWithImpl<LibraryIndexNotebookFoldedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexNotebookFoldedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'LibraryIndexEvent.notebookFolded(id: $id)';
}


}

/// @nodoc
abstract mixin class $LibraryIndexNotebookFoldedEventCopyWith<$Res> implements $LibraryIndexEventCopyWith<$Res> {
  factory $LibraryIndexNotebookFoldedEventCopyWith(LibraryIndexNotebookFoldedEvent value, $Res Function(LibraryIndexNotebookFoldedEvent) _then) = _$LibraryIndexNotebookFoldedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$LibraryIndexNotebookFoldedEventCopyWithImpl<$Res>
    implements $LibraryIndexNotebookFoldedEventCopyWith<$Res> {
  _$LibraryIndexNotebookFoldedEventCopyWithImpl(this._self, this._then);

  final LibraryIndexNotebookFoldedEvent _self;
  final $Res Function(LibraryIndexNotebookFoldedEvent) _then;

/// Create a copy of LibraryIndexEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(LibraryIndexNotebookFoldedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class LibraryIndexTagToggledEvent implements LibraryIndexEvent {
  const LibraryIndexTagToggledEvent(this.path);
  

 final  String path;

/// Create a copy of LibraryIndexEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LibraryIndexTagToggledEventCopyWith<LibraryIndexTagToggledEvent> get copyWith => _$LibraryIndexTagToggledEventCopyWithImpl<LibraryIndexTagToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexTagToggledEvent&&(identical(other.path, path) || other.path == path));
}


@override
int get hashCode => Object.hash(runtimeType,path);

@override
String toString() {
  return 'LibraryIndexEvent.tagToggled(path: $path)';
}


}

/// @nodoc
abstract mixin class $LibraryIndexTagToggledEventCopyWith<$Res> implements $LibraryIndexEventCopyWith<$Res> {
  factory $LibraryIndexTagToggledEventCopyWith(LibraryIndexTagToggledEvent value, $Res Function(LibraryIndexTagToggledEvent) _then) = _$LibraryIndexTagToggledEventCopyWithImpl;
@useResult
$Res call({
 String path
});




}
/// @nodoc
class _$LibraryIndexTagToggledEventCopyWithImpl<$Res>
    implements $LibraryIndexTagToggledEventCopyWith<$Res> {
  _$LibraryIndexTagToggledEventCopyWithImpl(this._self, this._then);

  final LibraryIndexTagToggledEvent _self;
  final $Res Function(LibraryIndexTagToggledEvent) _then;

/// Create a copy of LibraryIndexEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? path = null,}) {
  return _then(LibraryIndexTagToggledEvent(
null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class LibraryIndexAllTagsToggledEvent implements LibraryIndexEvent {
  const LibraryIndexAllTagsToggledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexAllTagsToggledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LibraryIndexEvent.allTagsToggled()';
}


}




/// @nodoc


class LibraryIndexStatusGroupToggledEvent implements LibraryIndexEvent {
  const LibraryIndexStatusGroupToggledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexStatusGroupToggledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LibraryIndexEvent.statusGroupToggled()';
}


}




/// @nodoc
mixin _$LibraryIndexState {

 LibraryIndexStatus get status;/// The notebooks in tree order, each with its depth and role.
 List<NotebookRow> get notebooks; List<TagRow> get tags;/// How many notes each place lists.
 LibraryOverview get overview;/// How many open tasks are overdue: the badge of the Tasks row.
 int get overdueTasks;/// The notebook rows that show, opened and folded as [folds] and the
/// place say.
 List<VisibleNotebook> get visibleNotebooks;/// The tag chips and the rows of the nested tag tree that show.
 TagPlaces get tagPlaces;/// The statuses a note can have, in the desktop's order.
 List<StatusChoice> get statuses;/// The notebooks opened or folded with their arrows.
 FoldState get folds;/// The tag levels opened with their arrows, by lower-case path.
 List<String> get openTags;/// Whether every tag shows as a chip, not only the most used few.
 bool get allTags;/// Whether the Status group shows its rows.
 bool get statusOpen;/// The place the library pane is drawing, which keeps the notebooks and
/// tag levels around it open.
 ListPlace? get place;/// Why the last reading failed, while [status] is failed.
 ApplicationError? get error;
/// Create a copy of LibraryIndexState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LibraryIndexStateCopyWith<LibraryIndexState> get copyWith => _$LibraryIndexStateCopyWithImpl<LibraryIndexState>(this as LibraryIndexState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryIndexState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.notebooks, notebooks)&&const DeepCollectionEquality().equals(other.tags, tags)&&(identical(other.overview, overview) || other.overview == overview)&&(identical(other.overdueTasks, overdueTasks) || other.overdueTasks == overdueTasks)&&const DeepCollectionEquality().equals(other.visibleNotebooks, visibleNotebooks)&&(identical(other.tagPlaces, tagPlaces) || other.tagPlaces == tagPlaces)&&const DeepCollectionEquality().equals(other.statuses, statuses)&&(identical(other.folds, folds) || other.folds == folds)&&const DeepCollectionEquality().equals(other.openTags, openTags)&&(identical(other.allTags, allTags) || other.allTags == allTags)&&(identical(other.statusOpen, statusOpen) || other.statusOpen == statusOpen)&&(identical(other.place, place) || other.place == place)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(notebooks),const DeepCollectionEquality().hash(tags),overview,overdueTasks,const DeepCollectionEquality().hash(visibleNotebooks),tagPlaces,const DeepCollectionEquality().hash(statuses),folds,const DeepCollectionEquality().hash(openTags),allTags,statusOpen,place,error);

@override
String toString() {
  return 'LibraryIndexState(status: $status, notebooks: $notebooks, tags: $tags, overview: $overview, overdueTasks: $overdueTasks, visibleNotebooks: $visibleNotebooks, tagPlaces: $tagPlaces, statuses: $statuses, folds: $folds, openTags: $openTags, allTags: $allTags, statusOpen: $statusOpen, place: $place, error: $error)';
}


}

/// @nodoc
abstract mixin class $LibraryIndexStateCopyWith<$Res>  {
  factory $LibraryIndexStateCopyWith(LibraryIndexState value, $Res Function(LibraryIndexState) _then) = _$LibraryIndexStateCopyWithImpl;
@useResult
$Res call({
 LibraryIndexStatus status, List<NotebookRow> notebooks, List<TagRow> tags, LibraryOverview overview, int overdueTasks, List<VisibleNotebook> visibleNotebooks, TagPlaces tagPlaces, List<StatusChoice> statuses, FoldState folds, List<String> openTags, bool allTags, bool statusOpen, ListPlace? place, ApplicationError? error
});


$ListPlaceCopyWith<$Res>? get place;$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$LibraryIndexStateCopyWithImpl<$Res>
    implements $LibraryIndexStateCopyWith<$Res> {
  _$LibraryIndexStateCopyWithImpl(this._self, this._then);

  final LibraryIndexState _self;
  final $Res Function(LibraryIndexState) _then;

/// Create a copy of LibraryIndexState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? notebooks = null,Object? tags = null,Object? overview = null,Object? overdueTasks = null,Object? visibleNotebooks = null,Object? tagPlaces = null,Object? statuses = null,Object? folds = null,Object? openTags = null,Object? allTags = null,Object? statusOpen = null,Object? place = freezed,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LibraryIndexStatus,notebooks: null == notebooks ? _self.notebooks : notebooks // ignore: cast_nullable_to_non_nullable
as List<NotebookRow>,tags: null == tags ? _self.tags : tags // ignore: cast_nullable_to_non_nullable
as List<TagRow>,overview: null == overview ? _self.overview : overview // ignore: cast_nullable_to_non_nullable
as LibraryOverview,overdueTasks: null == overdueTasks ? _self.overdueTasks : overdueTasks // ignore: cast_nullable_to_non_nullable
as int,visibleNotebooks: null == visibleNotebooks ? _self.visibleNotebooks : visibleNotebooks // ignore: cast_nullable_to_non_nullable
as List<VisibleNotebook>,tagPlaces: null == tagPlaces ? _self.tagPlaces : tagPlaces // ignore: cast_nullable_to_non_nullable
as TagPlaces,statuses: null == statuses ? _self.statuses : statuses // ignore: cast_nullable_to_non_nullable
as List<StatusChoice>,folds: null == folds ? _self.folds : folds // ignore: cast_nullable_to_non_nullable
as FoldState,openTags: null == openTags ? _self.openTags : openTags // ignore: cast_nullable_to_non_nullable
as List<String>,allTags: null == allTags ? _self.allTags : allTags // ignore: cast_nullable_to_non_nullable
as bool,statusOpen: null == statusOpen ? _self.statusOpen : statusOpen // ignore: cast_nullable_to_non_nullable
as bool,place: freezed == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as ListPlace?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of LibraryIndexState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListPlaceCopyWith<$Res>? get place {
    if (_self.place == null) {
    return null;
  }

  return $ListPlaceCopyWith<$Res>(_self.place!, (value) {
    return _then(_self.copyWith(place: value));
  });
}/// Create a copy of LibraryIndexState
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LibraryIndexStatus status,  List<NotebookRow> notebooks,  List<TagRow> tags,  LibraryOverview overview,  int overdueTasks,  List<VisibleNotebook> visibleNotebooks,  TagPlaces tagPlaces,  List<StatusChoice> statuses,  FoldState folds,  List<String> openTags,  bool allTags,  bool statusOpen,  ListPlace? place,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _LibraryIndexState() when $default != null:
return $default(_that.status,_that.notebooks,_that.tags,_that.overview,_that.overdueTasks,_that.visibleNotebooks,_that.tagPlaces,_that.statuses,_that.folds,_that.openTags,_that.allTags,_that.statusOpen,_that.place,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LibraryIndexStatus status,  List<NotebookRow> notebooks,  List<TagRow> tags,  LibraryOverview overview,  int overdueTasks,  List<VisibleNotebook> visibleNotebooks,  TagPlaces tagPlaces,  List<StatusChoice> statuses,  FoldState folds,  List<String> openTags,  bool allTags,  bool statusOpen,  ListPlace? place,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _LibraryIndexState():
return $default(_that.status,_that.notebooks,_that.tags,_that.overview,_that.overdueTasks,_that.visibleNotebooks,_that.tagPlaces,_that.statuses,_that.folds,_that.openTags,_that.allTags,_that.statusOpen,_that.place,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LibraryIndexStatus status,  List<NotebookRow> notebooks,  List<TagRow> tags,  LibraryOverview overview,  int overdueTasks,  List<VisibleNotebook> visibleNotebooks,  TagPlaces tagPlaces,  List<StatusChoice> statuses,  FoldState folds,  List<String> openTags,  bool allTags,  bool statusOpen,  ListPlace? place,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _LibraryIndexState() when $default != null:
return $default(_that.status,_that.notebooks,_that.tags,_that.overview,_that.overdueTasks,_that.visibleNotebooks,_that.tagPlaces,_that.statuses,_that.folds,_that.openTags,_that.allTags,_that.statusOpen,_that.place,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _LibraryIndexState extends LibraryIndexState {
  const _LibraryIndexState({this.status = LibraryIndexStatus.loading, final  List<NotebookRow> notebooks = const <NotebookRow>[], final  List<TagRow> tags = const <TagRow>[], this.overview = const LibraryOverview(all: 0, pinned: 0, recent: 0, trash: 0, byStatus: <StatusCount>[]), this.overdueTasks = 0, final  List<VisibleNotebook> visibleNotebooks = const <VisibleNotebook>[], this.tagPlaces = const TagPlaces(chips: <TagRow>[], more: false, tree: <TagTreeRow>[]), final  List<StatusChoice> statuses = const <StatusChoice>[], this.folds = const FoldState(open: <String>[], folded: <String>[]), final  List<String> openTags = const <String>[], this.allTags = false, this.statusOpen = true, this.place, this.error}): _notebooks = notebooks,_tags = tags,_visibleNotebooks = visibleNotebooks,_statuses = statuses,_openTags = openTags,super._();
  

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
/// The notebook rows that show, opened and folded as [folds] and the
/// place say.
 final  List<VisibleNotebook> _visibleNotebooks;
/// The notebook rows that show, opened and folded as [folds] and the
/// place say.
@override@JsonKey() List<VisibleNotebook> get visibleNotebooks {
  if (_visibleNotebooks is EqualUnmodifiableListView) return _visibleNotebooks;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_visibleNotebooks);
}

/// The tag chips and the rows of the nested tag tree that show.
@override@JsonKey() final  TagPlaces tagPlaces;
/// The statuses a note can have, in the desktop's order.
 final  List<StatusChoice> _statuses;
/// The statuses a note can have, in the desktop's order.
@override@JsonKey() List<StatusChoice> get statuses {
  if (_statuses is EqualUnmodifiableListView) return _statuses;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_statuses);
}

/// The notebooks opened or folded with their arrows.
@override@JsonKey() final  FoldState folds;
/// The tag levels opened with their arrows, by lower-case path.
 final  List<String> _openTags;
/// The tag levels opened with their arrows, by lower-case path.
@override@JsonKey() List<String> get openTags {
  if (_openTags is EqualUnmodifiableListView) return _openTags;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_openTags);
}

/// Whether every tag shows as a chip, not only the most used few.
@override@JsonKey() final  bool allTags;
/// Whether the Status group shows its rows.
@override@JsonKey() final  bool statusOpen;
/// The place the library pane is drawing, which keeps the notebooks and
/// tag levels around it open.
@override final  ListPlace? place;
/// Why the last reading failed, while [status] is failed.
@override final  ApplicationError? error;

/// Create a copy of LibraryIndexState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$LibraryIndexStateCopyWith<_LibraryIndexState> get copyWith => __$LibraryIndexStateCopyWithImpl<_LibraryIndexState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _LibraryIndexState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._notebooks, _notebooks)&&const DeepCollectionEquality().equals(other._tags, _tags)&&(identical(other.overview, overview) || other.overview == overview)&&(identical(other.overdueTasks, overdueTasks) || other.overdueTasks == overdueTasks)&&const DeepCollectionEquality().equals(other._visibleNotebooks, _visibleNotebooks)&&(identical(other.tagPlaces, tagPlaces) || other.tagPlaces == tagPlaces)&&const DeepCollectionEquality().equals(other._statuses, _statuses)&&(identical(other.folds, folds) || other.folds == folds)&&const DeepCollectionEquality().equals(other._openTags, _openTags)&&(identical(other.allTags, allTags) || other.allTags == allTags)&&(identical(other.statusOpen, statusOpen) || other.statusOpen == statusOpen)&&(identical(other.place, place) || other.place == place)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_notebooks),const DeepCollectionEquality().hash(_tags),overview,overdueTasks,const DeepCollectionEquality().hash(_visibleNotebooks),tagPlaces,const DeepCollectionEquality().hash(_statuses),folds,const DeepCollectionEquality().hash(_openTags),allTags,statusOpen,place,error);

@override
String toString() {
  return 'LibraryIndexState(status: $status, notebooks: $notebooks, tags: $tags, overview: $overview, overdueTasks: $overdueTasks, visibleNotebooks: $visibleNotebooks, tagPlaces: $tagPlaces, statuses: $statuses, folds: $folds, openTags: $openTags, allTags: $allTags, statusOpen: $statusOpen, place: $place, error: $error)';
}


}

/// @nodoc
abstract mixin class _$LibraryIndexStateCopyWith<$Res> implements $LibraryIndexStateCopyWith<$Res> {
  factory _$LibraryIndexStateCopyWith(_LibraryIndexState value, $Res Function(_LibraryIndexState) _then) = __$LibraryIndexStateCopyWithImpl;
@override @useResult
$Res call({
 LibraryIndexStatus status, List<NotebookRow> notebooks, List<TagRow> tags, LibraryOverview overview, int overdueTasks, List<VisibleNotebook> visibleNotebooks, TagPlaces tagPlaces, List<StatusChoice> statuses, FoldState folds, List<String> openTags, bool allTags, bool statusOpen, ListPlace? place, ApplicationError? error
});


@override $ListPlaceCopyWith<$Res>? get place;@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$LibraryIndexStateCopyWithImpl<$Res>
    implements _$LibraryIndexStateCopyWith<$Res> {
  __$LibraryIndexStateCopyWithImpl(this._self, this._then);

  final _LibraryIndexState _self;
  final $Res Function(_LibraryIndexState) _then;

/// Create a copy of LibraryIndexState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? notebooks = null,Object? tags = null,Object? overview = null,Object? overdueTasks = null,Object? visibleNotebooks = null,Object? tagPlaces = null,Object? statuses = null,Object? folds = null,Object? openTags = null,Object? allTags = null,Object? statusOpen = null,Object? place = freezed,Object? error = freezed,}) {
  return _then(_LibraryIndexState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LibraryIndexStatus,notebooks: null == notebooks ? _self._notebooks : notebooks // ignore: cast_nullable_to_non_nullable
as List<NotebookRow>,tags: null == tags ? _self._tags : tags // ignore: cast_nullable_to_non_nullable
as List<TagRow>,overview: null == overview ? _self.overview : overview // ignore: cast_nullable_to_non_nullable
as LibraryOverview,overdueTasks: null == overdueTasks ? _self.overdueTasks : overdueTasks // ignore: cast_nullable_to_non_nullable
as int,visibleNotebooks: null == visibleNotebooks ? _self._visibleNotebooks : visibleNotebooks // ignore: cast_nullable_to_non_nullable
as List<VisibleNotebook>,tagPlaces: null == tagPlaces ? _self.tagPlaces : tagPlaces // ignore: cast_nullable_to_non_nullable
as TagPlaces,statuses: null == statuses ? _self._statuses : statuses // ignore: cast_nullable_to_non_nullable
as List<StatusChoice>,folds: null == folds ? _self.folds : folds // ignore: cast_nullable_to_non_nullable
as FoldState,openTags: null == openTags ? _self._openTags : openTags // ignore: cast_nullable_to_non_nullable
as List<String>,allTags: null == allTags ? _self.allTags : allTags // ignore: cast_nullable_to_non_nullable
as bool,statusOpen: null == statusOpen ? _self.statusOpen : statusOpen // ignore: cast_nullable_to_non_nullable
as bool,place: freezed == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as ListPlace?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of LibraryIndexState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListPlaceCopyWith<$Res>? get place {
    if (_self.place == null) {
    return null;
  }

  return $ListPlaceCopyWith<$Res>(_self.place!, (value) {
    return _then(_self.copyWith(place: value));
  });
}/// Create a copy of LibraryIndexState
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
