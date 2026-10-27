// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'browse_location.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$BrowseLocation {

 String? get noteId;
/// Create a copy of BrowseLocation
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BrowseLocationCopyWith<BrowseLocation> get copyWith => _$BrowseLocationCopyWithImpl<BrowseLocation>(this as BrowseLocation, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BrowseLocation&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'BrowseLocation(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $BrowseLocationCopyWith<$Res>  {
  factory $BrowseLocationCopyWith(BrowseLocation value, $Res Function(BrowseLocation) _then) = _$BrowseLocationCopyWithImpl;
@useResult
$Res call({
 String? noteId
});




}
/// @nodoc
class _$BrowseLocationCopyWithImpl<$Res>
    implements $BrowseLocationCopyWith<$Res> {
  _$BrowseLocationCopyWithImpl(this._self, this._then);

  final BrowseLocation _self;
  final $Res Function(BrowseLocation) _then;

/// Create a copy of BrowseLocation
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = freezed,}) {
  return _then(_self.copyWith(
noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [BrowseLocation].
extension BrowseLocationPatterns on BrowseLocation {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PlaceLocation value)?  place,TResult Function( SearchLocation value)?  search,TResult Function( TasksLocation value)?  tasks,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PlaceLocation() when place != null:
return place(_that);case SearchLocation() when search != null:
return search(_that);case TasksLocation() when tasks != null:
return tasks(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PlaceLocation value)  place,required TResult Function( SearchLocation value)  search,required TResult Function( TasksLocation value)  tasks,}){
final _that = this;
switch (_that) {
case PlaceLocation():
return place(_that);case SearchLocation():
return search(_that);case TasksLocation():
return tasks(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PlaceLocation value)?  place,TResult? Function( SearchLocation value)?  search,TResult? Function( TasksLocation value)?  tasks,}){
final _that = this;
switch (_that) {
case PlaceLocation() when place != null:
return place(_that);case SearchLocation() when search != null:
return search(_that);case TasksLocation() when tasks != null:
return tasks(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( ListPlace place,  String? noteId)?  place,TResult Function( String query,  SearchScopeKind scope,  String? notebookId,  String? noteId)?  search,TResult Function( String? noteId)?  tasks,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PlaceLocation() when place != null:
return place(_that.place,_that.noteId);case SearchLocation() when search != null:
return search(_that.query,_that.scope,_that.notebookId,_that.noteId);case TasksLocation() when tasks != null:
return tasks(_that.noteId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( ListPlace place,  String? noteId)  place,required TResult Function( String query,  SearchScopeKind scope,  String? notebookId,  String? noteId)  search,required TResult Function( String? noteId)  tasks,}) {final _that = this;
switch (_that) {
case PlaceLocation():
return place(_that.place,_that.noteId);case SearchLocation():
return search(_that.query,_that.scope,_that.notebookId,_that.noteId);case TasksLocation():
return tasks(_that.noteId);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( ListPlace place,  String? noteId)?  place,TResult? Function( String query,  SearchScopeKind scope,  String? notebookId,  String? noteId)?  search,TResult? Function( String? noteId)?  tasks,}) {final _that = this;
switch (_that) {
case PlaceLocation() when place != null:
return place(_that.place,_that.noteId);case SearchLocation() when search != null:
return search(_that.query,_that.scope,_that.notebookId,_that.noteId);case TasksLocation() when tasks != null:
return tasks(_that.noteId);case _:
  return null;

}
}

}

/// @nodoc


class PlaceLocation extends BrowseLocation {
  const PlaceLocation(this.place, {this.noteId}): super._();
  

 final  ListPlace place;
@override final  String? noteId;

/// Create a copy of BrowseLocation
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlaceLocationCopyWith<PlaceLocation> get copyWith => _$PlaceLocationCopyWithImpl<PlaceLocation>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlaceLocation&&(identical(other.place, place) || other.place == place)&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,place,noteId);

@override
String toString() {
  return 'BrowseLocation.place(place: $place, noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $PlaceLocationCopyWith<$Res> implements $BrowseLocationCopyWith<$Res> {
  factory $PlaceLocationCopyWith(PlaceLocation value, $Res Function(PlaceLocation) _then) = _$PlaceLocationCopyWithImpl;
@override @useResult
$Res call({
 ListPlace place, String? noteId
});


$ListPlaceCopyWith<$Res> get place;

}
/// @nodoc
class _$PlaceLocationCopyWithImpl<$Res>
    implements $PlaceLocationCopyWith<$Res> {
  _$PlaceLocationCopyWithImpl(this._self, this._then);

  final PlaceLocation _self;
  final $Res Function(PlaceLocation) _then;

/// Create a copy of BrowseLocation
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? place = null,Object? noteId = freezed,}) {
  return _then(PlaceLocation(
null == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as ListPlace,noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of BrowseLocation
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListPlaceCopyWith<$Res> get place {
  
  return $ListPlaceCopyWith<$Res>(_self.place, (value) {
    return _then(_self.copyWith(place: value));
  });
}
}

/// @nodoc


class SearchLocation extends BrowseLocation {
  const SearchLocation({this.query = '', this.scope = SearchScopeKind.all, this.notebookId, this.noteId}): super._();
  

@JsonKey() final  String query;
@JsonKey() final  SearchScopeKind scope;
 final  String? notebookId;
@override final  String? noteId;

/// Create a copy of BrowseLocation
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SearchLocationCopyWith<SearchLocation> get copyWith => _$SearchLocationCopyWithImpl<SearchLocation>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchLocation&&(identical(other.query, query) || other.query == query)&&(identical(other.scope, scope) || other.scope == scope)&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId)&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,query,scope,notebookId,noteId);

@override
String toString() {
  return 'BrowseLocation.search(query: $query, scope: $scope, notebookId: $notebookId, noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $SearchLocationCopyWith<$Res> implements $BrowseLocationCopyWith<$Res> {
  factory $SearchLocationCopyWith(SearchLocation value, $Res Function(SearchLocation) _then) = _$SearchLocationCopyWithImpl;
@override @useResult
$Res call({
 String query, SearchScopeKind scope, String? notebookId, String? noteId
});




}
/// @nodoc
class _$SearchLocationCopyWithImpl<$Res>
    implements $SearchLocationCopyWith<$Res> {
  _$SearchLocationCopyWithImpl(this._self, this._then);

  final SearchLocation _self;
  final $Res Function(SearchLocation) _then;

/// Create a copy of BrowseLocation
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? query = null,Object? scope = null,Object? notebookId = freezed,Object? noteId = freezed,}) {
  return _then(SearchLocation(
query: null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,scope: null == scope ? _self.scope : scope // ignore: cast_nullable_to_non_nullable
as SearchScopeKind,notebookId: freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class TasksLocation extends BrowseLocation {
  const TasksLocation({this.noteId}): super._();
  

@override final  String? noteId;

/// Create a copy of BrowseLocation
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TasksLocationCopyWith<TasksLocation> get copyWith => _$TasksLocationCopyWithImpl<TasksLocation>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TasksLocation&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'BrowseLocation.tasks(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $TasksLocationCopyWith<$Res> implements $BrowseLocationCopyWith<$Res> {
  factory $TasksLocationCopyWith(TasksLocation value, $Res Function(TasksLocation) _then) = _$TasksLocationCopyWithImpl;
@override @useResult
$Res call({
 String? noteId
});




}
/// @nodoc
class _$TasksLocationCopyWithImpl<$Res>
    implements $TasksLocationCopyWith<$Res> {
  _$TasksLocationCopyWithImpl(this._self, this._then);

  final TasksLocation _self;
  final $Res Function(TasksLocation) _then;

/// Create a copy of BrowseLocation
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = freezed,}) {
  return _then(TasksLocation(
noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
