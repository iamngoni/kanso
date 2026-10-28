// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'theme_gallery_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ThemeGalleryEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeGalleryEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ThemeGalleryEvent()';
}


}

/// @nodoc
class $ThemeGalleryEventCopyWith<$Res>  {
$ThemeGalleryEventCopyWith(ThemeGalleryEvent _, $Res Function(ThemeGalleryEvent) __);
}


/// Adds pattern-matching-related methods to [ThemeGalleryEvent].
extension ThemeGalleryEventPatterns on ThemeGalleryEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ThemeGalleryStartedEvent value)?  started,TResult Function( ThemeGalleryRefreshedEvent value)?  refreshed,TResult Function( ThemeGallerySearchedEvent value)?  searched,TResult Function( ThemeGalleryFilterChosenEvent value)?  filterChosen,TResult Function( ThemeGalleryInstalledEvent value)?  installed,TResult Function( ThemeGalleryUninstalledEvent value)?  uninstalled,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ThemeGalleryStartedEvent() when started != null:
return started(_that);case ThemeGalleryRefreshedEvent() when refreshed != null:
return refreshed(_that);case ThemeGallerySearchedEvent() when searched != null:
return searched(_that);case ThemeGalleryFilterChosenEvent() when filterChosen != null:
return filterChosen(_that);case ThemeGalleryInstalledEvent() when installed != null:
return installed(_that);case ThemeGalleryUninstalledEvent() when uninstalled != null:
return uninstalled(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ThemeGalleryStartedEvent value)  started,required TResult Function( ThemeGalleryRefreshedEvent value)  refreshed,required TResult Function( ThemeGallerySearchedEvent value)  searched,required TResult Function( ThemeGalleryFilterChosenEvent value)  filterChosen,required TResult Function( ThemeGalleryInstalledEvent value)  installed,required TResult Function( ThemeGalleryUninstalledEvent value)  uninstalled,}){
final _that = this;
switch (_that) {
case ThemeGalleryStartedEvent():
return started(_that);case ThemeGalleryRefreshedEvent():
return refreshed(_that);case ThemeGallerySearchedEvent():
return searched(_that);case ThemeGalleryFilterChosenEvent():
return filterChosen(_that);case ThemeGalleryInstalledEvent():
return installed(_that);case ThemeGalleryUninstalledEvent():
return uninstalled(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ThemeGalleryStartedEvent value)?  started,TResult? Function( ThemeGalleryRefreshedEvent value)?  refreshed,TResult? Function( ThemeGallerySearchedEvent value)?  searched,TResult? Function( ThemeGalleryFilterChosenEvent value)?  filterChosen,TResult? Function( ThemeGalleryInstalledEvent value)?  installed,TResult? Function( ThemeGalleryUninstalledEvent value)?  uninstalled,}){
final _that = this;
switch (_that) {
case ThemeGalleryStartedEvent() when started != null:
return started(_that);case ThemeGalleryRefreshedEvent() when refreshed != null:
return refreshed(_that);case ThemeGallerySearchedEvent() when searched != null:
return searched(_that);case ThemeGalleryFilterChosenEvent() when filterChosen != null:
return filterChosen(_that);case ThemeGalleryInstalledEvent() when installed != null:
return installed(_that);case ThemeGalleryUninstalledEvent() when uninstalled != null:
return uninstalled(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  refreshed,TResult Function( String query)?  searched,TResult Function( GalleryFilter filter)?  filterChosen,TResult Function( String id)?  installed,TResult Function( String id)?  uninstalled,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ThemeGalleryStartedEvent() when started != null:
return started();case ThemeGalleryRefreshedEvent() when refreshed != null:
return refreshed();case ThemeGallerySearchedEvent() when searched != null:
return searched(_that.query);case ThemeGalleryFilterChosenEvent() when filterChosen != null:
return filterChosen(_that.filter);case ThemeGalleryInstalledEvent() when installed != null:
return installed(_that.id);case ThemeGalleryUninstalledEvent() when uninstalled != null:
return uninstalled(_that.id);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  refreshed,required TResult Function( String query)  searched,required TResult Function( GalleryFilter filter)  filterChosen,required TResult Function( String id)  installed,required TResult Function( String id)  uninstalled,}) {final _that = this;
switch (_that) {
case ThemeGalleryStartedEvent():
return started();case ThemeGalleryRefreshedEvent():
return refreshed();case ThemeGallerySearchedEvent():
return searched(_that.query);case ThemeGalleryFilterChosenEvent():
return filterChosen(_that.filter);case ThemeGalleryInstalledEvent():
return installed(_that.id);case ThemeGalleryUninstalledEvent():
return uninstalled(_that.id);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  refreshed,TResult? Function( String query)?  searched,TResult? Function( GalleryFilter filter)?  filterChosen,TResult? Function( String id)?  installed,TResult? Function( String id)?  uninstalled,}) {final _that = this;
switch (_that) {
case ThemeGalleryStartedEvent() when started != null:
return started();case ThemeGalleryRefreshedEvent() when refreshed != null:
return refreshed();case ThemeGallerySearchedEvent() when searched != null:
return searched(_that.query);case ThemeGalleryFilterChosenEvent() when filterChosen != null:
return filterChosen(_that.filter);case ThemeGalleryInstalledEvent() when installed != null:
return installed(_that.id);case ThemeGalleryUninstalledEvent() when uninstalled != null:
return uninstalled(_that.id);case _:
  return null;

}
}

}

/// @nodoc


class ThemeGalleryStartedEvent implements ThemeGalleryEvent {
  const ThemeGalleryStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeGalleryStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ThemeGalleryEvent.started()';
}


}




/// @nodoc


class ThemeGalleryRefreshedEvent implements ThemeGalleryEvent {
  const ThemeGalleryRefreshedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeGalleryRefreshedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ThemeGalleryEvent.refreshed()';
}


}




/// @nodoc


class ThemeGallerySearchedEvent implements ThemeGalleryEvent {
  const ThemeGallerySearchedEvent(this.query);
  

 final  String query;

/// Create a copy of ThemeGalleryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemeGallerySearchedEventCopyWith<ThemeGallerySearchedEvent> get copyWith => _$ThemeGallerySearchedEventCopyWithImpl<ThemeGallerySearchedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeGallerySearchedEvent&&(identical(other.query, query) || other.query == query));
}


@override
int get hashCode => Object.hash(runtimeType,query);

@override
String toString() {
  return 'ThemeGalleryEvent.searched(query: $query)';
}


}

/// @nodoc
abstract mixin class $ThemeGallerySearchedEventCopyWith<$Res> implements $ThemeGalleryEventCopyWith<$Res> {
  factory $ThemeGallerySearchedEventCopyWith(ThemeGallerySearchedEvent value, $Res Function(ThemeGallerySearchedEvent) _then) = _$ThemeGallerySearchedEventCopyWithImpl;
@useResult
$Res call({
 String query
});




}
/// @nodoc
class _$ThemeGallerySearchedEventCopyWithImpl<$Res>
    implements $ThemeGallerySearchedEventCopyWith<$Res> {
  _$ThemeGallerySearchedEventCopyWithImpl(this._self, this._then);

  final ThemeGallerySearchedEvent _self;
  final $Res Function(ThemeGallerySearchedEvent) _then;

/// Create a copy of ThemeGalleryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? query = null,}) {
  return _then(ThemeGallerySearchedEvent(
null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ThemeGalleryFilterChosenEvent implements ThemeGalleryEvent {
  const ThemeGalleryFilterChosenEvent(this.filter);
  

 final  GalleryFilter filter;

/// Create a copy of ThemeGalleryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemeGalleryFilterChosenEventCopyWith<ThemeGalleryFilterChosenEvent> get copyWith => _$ThemeGalleryFilterChosenEventCopyWithImpl<ThemeGalleryFilterChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeGalleryFilterChosenEvent&&(identical(other.filter, filter) || other.filter == filter));
}


@override
int get hashCode => Object.hash(runtimeType,filter);

@override
String toString() {
  return 'ThemeGalleryEvent.filterChosen(filter: $filter)';
}


}

/// @nodoc
abstract mixin class $ThemeGalleryFilterChosenEventCopyWith<$Res> implements $ThemeGalleryEventCopyWith<$Res> {
  factory $ThemeGalleryFilterChosenEventCopyWith(ThemeGalleryFilterChosenEvent value, $Res Function(ThemeGalleryFilterChosenEvent) _then) = _$ThemeGalleryFilterChosenEventCopyWithImpl;
@useResult
$Res call({
 GalleryFilter filter
});




}
/// @nodoc
class _$ThemeGalleryFilterChosenEventCopyWithImpl<$Res>
    implements $ThemeGalleryFilterChosenEventCopyWith<$Res> {
  _$ThemeGalleryFilterChosenEventCopyWithImpl(this._self, this._then);

  final ThemeGalleryFilterChosenEvent _self;
  final $Res Function(ThemeGalleryFilterChosenEvent) _then;

/// Create a copy of ThemeGalleryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? filter = null,}) {
  return _then(ThemeGalleryFilterChosenEvent(
null == filter ? _self.filter : filter // ignore: cast_nullable_to_non_nullable
as GalleryFilter,
  ));
}


}

/// @nodoc


class ThemeGalleryInstalledEvent implements ThemeGalleryEvent {
  const ThemeGalleryInstalledEvent(this.id);
  

 final  String id;

/// Create a copy of ThemeGalleryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemeGalleryInstalledEventCopyWith<ThemeGalleryInstalledEvent> get copyWith => _$ThemeGalleryInstalledEventCopyWithImpl<ThemeGalleryInstalledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeGalleryInstalledEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'ThemeGalleryEvent.installed(id: $id)';
}


}

/// @nodoc
abstract mixin class $ThemeGalleryInstalledEventCopyWith<$Res> implements $ThemeGalleryEventCopyWith<$Res> {
  factory $ThemeGalleryInstalledEventCopyWith(ThemeGalleryInstalledEvent value, $Res Function(ThemeGalleryInstalledEvent) _then) = _$ThemeGalleryInstalledEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$ThemeGalleryInstalledEventCopyWithImpl<$Res>
    implements $ThemeGalleryInstalledEventCopyWith<$Res> {
  _$ThemeGalleryInstalledEventCopyWithImpl(this._self, this._then);

  final ThemeGalleryInstalledEvent _self;
  final $Res Function(ThemeGalleryInstalledEvent) _then;

/// Create a copy of ThemeGalleryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(ThemeGalleryInstalledEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ThemeGalleryUninstalledEvent implements ThemeGalleryEvent {
  const ThemeGalleryUninstalledEvent(this.id);
  

 final  String id;

/// Create a copy of ThemeGalleryEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemeGalleryUninstalledEventCopyWith<ThemeGalleryUninstalledEvent> get copyWith => _$ThemeGalleryUninstalledEventCopyWithImpl<ThemeGalleryUninstalledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeGalleryUninstalledEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'ThemeGalleryEvent.uninstalled(id: $id)';
}


}

/// @nodoc
abstract mixin class $ThemeGalleryUninstalledEventCopyWith<$Res> implements $ThemeGalleryEventCopyWith<$Res> {
  factory $ThemeGalleryUninstalledEventCopyWith(ThemeGalleryUninstalledEvent value, $Res Function(ThemeGalleryUninstalledEvent) _then) = _$ThemeGalleryUninstalledEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$ThemeGalleryUninstalledEventCopyWithImpl<$Res>
    implements $ThemeGalleryUninstalledEventCopyWith<$Res> {
  _$ThemeGalleryUninstalledEventCopyWithImpl(this._self, this._then);

  final ThemeGalleryUninstalledEvent _self;
  final $Res Function(ThemeGalleryUninstalledEvent) _then;

/// Create a copy of ThemeGalleryEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(ThemeGalleryUninstalledEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$ThemeGalleryState {

 LoadStatus get status;/// Every item of the index, in the catalog's order.
 List<GalleryItem> get items;/// What the search field holds.
 String get query; GalleryFilter get filter;/// The current index is being fetched while the last one is shown.
 bool get refreshing;/// The current index could not be fetched; the last one is shown.
 bool get refreshFailed;/// The items being installed, updated or removed now, by id.
 Set<String> get working;/// Why the last install, update or removal did not happen.
 GalleryProblem? get problem;/// Counts the times the themes installed from the gallery changed, so a
/// listener can tell that the installed themes must be read again.
 int get installedRevision;/// The ids of the themes that changed the last time: installed, updated,
/// removed, or withdrawn by the catalog.
 Set<String> get changed;
/// Create a copy of ThemeGalleryState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemeGalleryStateCopyWith<ThemeGalleryState> get copyWith => _$ThemeGalleryStateCopyWithImpl<ThemeGalleryState>(this as ThemeGalleryState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeGalleryState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.items, items)&&(identical(other.query, query) || other.query == query)&&(identical(other.filter, filter) || other.filter == filter)&&(identical(other.refreshing, refreshing) || other.refreshing == refreshing)&&(identical(other.refreshFailed, refreshFailed) || other.refreshFailed == refreshFailed)&&const DeepCollectionEquality().equals(other.working, working)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.installedRevision, installedRevision) || other.installedRevision == installedRevision)&&const DeepCollectionEquality().equals(other.changed, changed));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(items),query,filter,refreshing,refreshFailed,const DeepCollectionEquality().hash(working),problem,installedRevision,const DeepCollectionEquality().hash(changed));

@override
String toString() {
  return 'ThemeGalleryState(status: $status, items: $items, query: $query, filter: $filter, refreshing: $refreshing, refreshFailed: $refreshFailed, working: $working, problem: $problem, installedRevision: $installedRevision, changed: $changed)';
}


}

/// @nodoc
abstract mixin class $ThemeGalleryStateCopyWith<$Res>  {
  factory $ThemeGalleryStateCopyWith(ThemeGalleryState value, $Res Function(ThemeGalleryState) _then) = _$ThemeGalleryStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<GalleryItem> items, String query, GalleryFilter filter, bool refreshing, bool refreshFailed, Set<String> working, GalleryProblem? problem, int installedRevision, Set<String> changed
});


$GalleryProblemCopyWith<$Res>? get problem;

}
/// @nodoc
class _$ThemeGalleryStateCopyWithImpl<$Res>
    implements $ThemeGalleryStateCopyWith<$Res> {
  _$ThemeGalleryStateCopyWithImpl(this._self, this._then);

  final ThemeGalleryState _self;
  final $Res Function(ThemeGalleryState) _then;

/// Create a copy of ThemeGalleryState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? items = null,Object? query = null,Object? filter = null,Object? refreshing = null,Object? refreshFailed = null,Object? working = null,Object? problem = freezed,Object? installedRevision = null,Object? changed = null,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,items: null == items ? _self.items : items // ignore: cast_nullable_to_non_nullable
as List<GalleryItem>,query: null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,filter: null == filter ? _self.filter : filter // ignore: cast_nullable_to_non_nullable
as GalleryFilter,refreshing: null == refreshing ? _self.refreshing : refreshing // ignore: cast_nullable_to_non_nullable
as bool,refreshFailed: null == refreshFailed ? _self.refreshFailed : refreshFailed // ignore: cast_nullable_to_non_nullable
as bool,working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as Set<String>,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as GalleryProblem?,installedRevision: null == installedRevision ? _self.installedRevision : installedRevision // ignore: cast_nullable_to_non_nullable
as int,changed: null == changed ? _self.changed : changed // ignore: cast_nullable_to_non_nullable
as Set<String>,
  ));
}
/// Create a copy of ThemeGalleryState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$GalleryProblemCopyWith<$Res>? get problem {
    if (_self.problem == null) {
    return null;
  }

  return $GalleryProblemCopyWith<$Res>(_self.problem!, (value) {
    return _then(_self.copyWith(problem: value));
  });
}
}


/// Adds pattern-matching-related methods to [ThemeGalleryState].
extension ThemeGalleryStatePatterns on ThemeGalleryState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ThemeGalleryState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ThemeGalleryState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ThemeGalleryState value)  $default,){
final _that = this;
switch (_that) {
case _ThemeGalleryState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ThemeGalleryState value)?  $default,){
final _that = this;
switch (_that) {
case _ThemeGalleryState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<GalleryItem> items,  String query,  GalleryFilter filter,  bool refreshing,  bool refreshFailed,  Set<String> working,  GalleryProblem? problem,  int installedRevision,  Set<String> changed)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ThemeGalleryState() when $default != null:
return $default(_that.status,_that.items,_that.query,_that.filter,_that.refreshing,_that.refreshFailed,_that.working,_that.problem,_that.installedRevision,_that.changed);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<GalleryItem> items,  String query,  GalleryFilter filter,  bool refreshing,  bool refreshFailed,  Set<String> working,  GalleryProblem? problem,  int installedRevision,  Set<String> changed)  $default,) {final _that = this;
switch (_that) {
case _ThemeGalleryState():
return $default(_that.status,_that.items,_that.query,_that.filter,_that.refreshing,_that.refreshFailed,_that.working,_that.problem,_that.installedRevision,_that.changed);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<GalleryItem> items,  String query,  GalleryFilter filter,  bool refreshing,  bool refreshFailed,  Set<String> working,  GalleryProblem? problem,  int installedRevision,  Set<String> changed)?  $default,) {final _that = this;
switch (_that) {
case _ThemeGalleryState() when $default != null:
return $default(_that.status,_that.items,_that.query,_that.filter,_that.refreshing,_that.refreshFailed,_that.working,_that.problem,_that.installedRevision,_that.changed);case _:
  return null;

}
}

}

/// @nodoc


class _ThemeGalleryState extends ThemeGalleryState {
  const _ThemeGalleryState({this.status = LoadStatus.loading, final  List<GalleryItem> items = const <GalleryItem>[], this.query = '', this.filter = GalleryFilter.popular, this.refreshing = false, this.refreshFailed = false, final  Set<String> working = const <String>{}, this.problem, this.installedRevision = 0, final  Set<String> changed = const <String>{}}): _items = items,_working = working,_changed = changed,super._();
  

@override@JsonKey() final  LoadStatus status;
/// Every item of the index, in the catalog's order.
 final  List<GalleryItem> _items;
/// Every item of the index, in the catalog's order.
@override@JsonKey() List<GalleryItem> get items {
  if (_items is EqualUnmodifiableListView) return _items;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_items);
}

/// What the search field holds.
@override@JsonKey() final  String query;
@override@JsonKey() final  GalleryFilter filter;
/// The current index is being fetched while the last one is shown.
@override@JsonKey() final  bool refreshing;
/// The current index could not be fetched; the last one is shown.
@override@JsonKey() final  bool refreshFailed;
/// The items being installed, updated or removed now, by id.
 final  Set<String> _working;
/// The items being installed, updated or removed now, by id.
@override@JsonKey() Set<String> get working {
  if (_working is EqualUnmodifiableSetView) return _working;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_working);
}

/// Why the last install, update or removal did not happen.
@override final  GalleryProblem? problem;
/// Counts the times the themes installed from the gallery changed, so a
/// listener can tell that the installed themes must be read again.
@override@JsonKey() final  int installedRevision;
/// The ids of the themes that changed the last time: installed, updated,
/// removed, or withdrawn by the catalog.
 final  Set<String> _changed;
/// The ids of the themes that changed the last time: installed, updated,
/// removed, or withdrawn by the catalog.
@override@JsonKey() Set<String> get changed {
  if (_changed is EqualUnmodifiableSetView) return _changed;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_changed);
}


/// Create a copy of ThemeGalleryState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ThemeGalleryStateCopyWith<_ThemeGalleryState> get copyWith => __$ThemeGalleryStateCopyWithImpl<_ThemeGalleryState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ThemeGalleryState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._items, _items)&&(identical(other.query, query) || other.query == query)&&(identical(other.filter, filter) || other.filter == filter)&&(identical(other.refreshing, refreshing) || other.refreshing == refreshing)&&(identical(other.refreshFailed, refreshFailed) || other.refreshFailed == refreshFailed)&&const DeepCollectionEquality().equals(other._working, _working)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.installedRevision, installedRevision) || other.installedRevision == installedRevision)&&const DeepCollectionEquality().equals(other._changed, _changed));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_items),query,filter,refreshing,refreshFailed,const DeepCollectionEquality().hash(_working),problem,installedRevision,const DeepCollectionEquality().hash(_changed));

@override
String toString() {
  return 'ThemeGalleryState(status: $status, items: $items, query: $query, filter: $filter, refreshing: $refreshing, refreshFailed: $refreshFailed, working: $working, problem: $problem, installedRevision: $installedRevision, changed: $changed)';
}


}

/// @nodoc
abstract mixin class _$ThemeGalleryStateCopyWith<$Res> implements $ThemeGalleryStateCopyWith<$Res> {
  factory _$ThemeGalleryStateCopyWith(_ThemeGalleryState value, $Res Function(_ThemeGalleryState) _then) = __$ThemeGalleryStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<GalleryItem> items, String query, GalleryFilter filter, bool refreshing, bool refreshFailed, Set<String> working, GalleryProblem? problem, int installedRevision, Set<String> changed
});


@override $GalleryProblemCopyWith<$Res>? get problem;

}
/// @nodoc
class __$ThemeGalleryStateCopyWithImpl<$Res>
    implements _$ThemeGalleryStateCopyWith<$Res> {
  __$ThemeGalleryStateCopyWithImpl(this._self, this._then);

  final _ThemeGalleryState _self;
  final $Res Function(_ThemeGalleryState) _then;

/// Create a copy of ThemeGalleryState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? items = null,Object? query = null,Object? filter = null,Object? refreshing = null,Object? refreshFailed = null,Object? working = null,Object? problem = freezed,Object? installedRevision = null,Object? changed = null,}) {
  return _then(_ThemeGalleryState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,items: null == items ? _self._items : items // ignore: cast_nullable_to_non_nullable
as List<GalleryItem>,query: null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,filter: null == filter ? _self.filter : filter // ignore: cast_nullable_to_non_nullable
as GalleryFilter,refreshing: null == refreshing ? _self.refreshing : refreshing // ignore: cast_nullable_to_non_nullable
as bool,refreshFailed: null == refreshFailed ? _self.refreshFailed : refreshFailed // ignore: cast_nullable_to_non_nullable
as bool,working: null == working ? _self._working : working // ignore: cast_nullable_to_non_nullable
as Set<String>,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as GalleryProblem?,installedRevision: null == installedRevision ? _self.installedRevision : installedRevision // ignore: cast_nullable_to_non_nullable
as int,changed: null == changed ? _self._changed : changed // ignore: cast_nullable_to_non_nullable
as Set<String>,
  ));
}

/// Create a copy of ThemeGalleryState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$GalleryProblemCopyWith<$Res>? get problem {
    if (_self.problem == null) {
    return null;
  }

  return $GalleryProblemCopyWith<$Res>(_self.problem!, (value) {
    return _then(_self.copyWith(problem: value));
  });
}
}

// dart format on
