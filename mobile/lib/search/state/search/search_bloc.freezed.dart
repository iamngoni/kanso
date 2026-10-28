// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'search_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SearchEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchEvent()';
}


}

/// @nodoc
class $SearchEventCopyWith<$Res>  {
$SearchEventCopyWith(SearchEvent _, $Res Function(SearchEvent) __);
}


/// Adds pattern-matching-related methods to [SearchEvent].
extension SearchEventPatterns on SearchEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SearchStartedEvent value)?  started,TResult Function( SearchQueryChangedEvent value)?  queryChanged,TResult Function( SearchScopeChangedEvent value)?  scopeChanged,TResult Function( SearchClearedEvent value)?  cleared,TResult Function( SearchChangesReceivedEvent value)?  changesReceived,TResult Function( SearchRetriedEvent value)?  retried,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SearchStartedEvent() when started != null:
return started(_that);case SearchQueryChangedEvent() when queryChanged != null:
return queryChanged(_that);case SearchScopeChangedEvent() when scopeChanged != null:
return scopeChanged(_that);case SearchClearedEvent() when cleared != null:
return cleared(_that);case SearchChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case SearchRetriedEvent() when retried != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SearchStartedEvent value)  started,required TResult Function( SearchQueryChangedEvent value)  queryChanged,required TResult Function( SearchScopeChangedEvent value)  scopeChanged,required TResult Function( SearchClearedEvent value)  cleared,required TResult Function( SearchChangesReceivedEvent value)  changesReceived,required TResult Function( SearchRetriedEvent value)  retried,}){
final _that = this;
switch (_that) {
case SearchStartedEvent():
return started(_that);case SearchQueryChangedEvent():
return queryChanged(_that);case SearchScopeChangedEvent():
return scopeChanged(_that);case SearchClearedEvent():
return cleared(_that);case SearchChangesReceivedEvent():
return changesReceived(_that);case SearchRetriedEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SearchStartedEvent value)?  started,TResult? Function( SearchQueryChangedEvent value)?  queryChanged,TResult? Function( SearchScopeChangedEvent value)?  scopeChanged,TResult? Function( SearchClearedEvent value)?  cleared,TResult? Function( SearchChangesReceivedEvent value)?  changesReceived,TResult? Function( SearchRetriedEvent value)?  retried,}){
final _that = this;
switch (_that) {
case SearchStartedEvent() when started != null:
return started(_that);case SearchQueryChangedEvent() when queryChanged != null:
return queryChanged(_that);case SearchScopeChangedEvent() when scopeChanged != null:
return scopeChanged(_that);case SearchClearedEvent() when cleared != null:
return cleared(_that);case SearchChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case SearchRetriedEvent() when retried != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String query,  SearchScopeKind scope,  String? notebookId)?  started,TResult Function( String text)?  queryChanged,TResult Function( SearchScopeKind scope)?  scopeChanged,TResult Function()?  cleared,TResult Function( List<String> ids)?  changesReceived,TResult Function()?  retried,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SearchStartedEvent() when started != null:
return started(_that.query,_that.scope,_that.notebookId);case SearchQueryChangedEvent() when queryChanged != null:
return queryChanged(_that.text);case SearchScopeChangedEvent() when scopeChanged != null:
return scopeChanged(_that.scope);case SearchClearedEvent() when cleared != null:
return cleared();case SearchChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case SearchRetriedEvent() when retried != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String query,  SearchScopeKind scope,  String? notebookId)  started,required TResult Function( String text)  queryChanged,required TResult Function( SearchScopeKind scope)  scopeChanged,required TResult Function()  cleared,required TResult Function( List<String> ids)  changesReceived,required TResult Function()  retried,}) {final _that = this;
switch (_that) {
case SearchStartedEvent():
return started(_that.query,_that.scope,_that.notebookId);case SearchQueryChangedEvent():
return queryChanged(_that.text);case SearchScopeChangedEvent():
return scopeChanged(_that.scope);case SearchClearedEvent():
return cleared();case SearchChangesReceivedEvent():
return changesReceived(_that.ids);case SearchRetriedEvent():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String query,  SearchScopeKind scope,  String? notebookId)?  started,TResult? Function( String text)?  queryChanged,TResult? Function( SearchScopeKind scope)?  scopeChanged,TResult? Function()?  cleared,TResult? Function( List<String> ids)?  changesReceived,TResult? Function()?  retried,}) {final _that = this;
switch (_that) {
case SearchStartedEvent() when started != null:
return started(_that.query,_that.scope,_that.notebookId);case SearchQueryChangedEvent() when queryChanged != null:
return queryChanged(_that.text);case SearchScopeChangedEvent() when scopeChanged != null:
return scopeChanged(_that.scope);case SearchClearedEvent() when cleared != null:
return cleared();case SearchChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case SearchRetriedEvent() when retried != null:
return retried();case _:
  return null;

}
}

}

/// @nodoc


class SearchStartedEvent implements SearchEvent {
  const SearchStartedEvent(this.query, this.scope, this.notebookId);
  

 final  String query;
 final  SearchScopeKind scope;
 final  String? notebookId;

/// Create a copy of SearchEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SearchStartedEventCopyWith<SearchStartedEvent> get copyWith => _$SearchStartedEventCopyWithImpl<SearchStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchStartedEvent&&(identical(other.query, query) || other.query == query)&&(identical(other.scope, scope) || other.scope == scope)&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,query,scope,notebookId);

@override
String toString() {
  return 'SearchEvent.started(query: $query, scope: $scope, notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $SearchStartedEventCopyWith<$Res> implements $SearchEventCopyWith<$Res> {
  factory $SearchStartedEventCopyWith(SearchStartedEvent value, $Res Function(SearchStartedEvent) _then) = _$SearchStartedEventCopyWithImpl;
@useResult
$Res call({
 String query, SearchScopeKind scope, String? notebookId
});




}
/// @nodoc
class _$SearchStartedEventCopyWithImpl<$Res>
    implements $SearchStartedEventCopyWith<$Res> {
  _$SearchStartedEventCopyWithImpl(this._self, this._then);

  final SearchStartedEvent _self;
  final $Res Function(SearchStartedEvent) _then;

/// Create a copy of SearchEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? query = null,Object? scope = null,Object? notebookId = freezed,}) {
  return _then(SearchStartedEvent(
null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,null == scope ? _self.scope : scope // ignore: cast_nullable_to_non_nullable
as SearchScopeKind,freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class SearchQueryChangedEvent implements SearchEvent {
  const SearchQueryChangedEvent(this.text);
  

 final  String text;

/// Create a copy of SearchEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SearchQueryChangedEventCopyWith<SearchQueryChangedEvent> get copyWith => _$SearchQueryChangedEventCopyWithImpl<SearchQueryChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchQueryChangedEvent&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'SearchEvent.queryChanged(text: $text)';
}


}

/// @nodoc
abstract mixin class $SearchQueryChangedEventCopyWith<$Res> implements $SearchEventCopyWith<$Res> {
  factory $SearchQueryChangedEventCopyWith(SearchQueryChangedEvent value, $Res Function(SearchQueryChangedEvent) _then) = _$SearchQueryChangedEventCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$SearchQueryChangedEventCopyWithImpl<$Res>
    implements $SearchQueryChangedEventCopyWith<$Res> {
  _$SearchQueryChangedEventCopyWithImpl(this._self, this._then);

  final SearchQueryChangedEvent _self;
  final $Res Function(SearchQueryChangedEvent) _then;

/// Create a copy of SearchEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(SearchQueryChangedEvent(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SearchScopeChangedEvent implements SearchEvent {
  const SearchScopeChangedEvent(this.scope);
  

 final  SearchScopeKind scope;

/// Create a copy of SearchEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SearchScopeChangedEventCopyWith<SearchScopeChangedEvent> get copyWith => _$SearchScopeChangedEventCopyWithImpl<SearchScopeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScopeChangedEvent&&(identical(other.scope, scope) || other.scope == scope));
}


@override
int get hashCode => Object.hash(runtimeType,scope);

@override
String toString() {
  return 'SearchEvent.scopeChanged(scope: $scope)';
}


}

/// @nodoc
abstract mixin class $SearchScopeChangedEventCopyWith<$Res> implements $SearchEventCopyWith<$Res> {
  factory $SearchScopeChangedEventCopyWith(SearchScopeChangedEvent value, $Res Function(SearchScopeChangedEvent) _then) = _$SearchScopeChangedEventCopyWithImpl;
@useResult
$Res call({
 SearchScopeKind scope
});




}
/// @nodoc
class _$SearchScopeChangedEventCopyWithImpl<$Res>
    implements $SearchScopeChangedEventCopyWith<$Res> {
  _$SearchScopeChangedEventCopyWithImpl(this._self, this._then);

  final SearchScopeChangedEvent _self;
  final $Res Function(SearchScopeChangedEvent) _then;

/// Create a copy of SearchEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? scope = null,}) {
  return _then(SearchScopeChangedEvent(
null == scope ? _self.scope : scope // ignore: cast_nullable_to_non_nullable
as SearchScopeKind,
  ));
}


}

/// @nodoc


class SearchClearedEvent implements SearchEvent {
  const SearchClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchEvent.cleared()';
}


}




/// @nodoc


class SearchChangesReceivedEvent implements SearchEvent {
  const SearchChangesReceivedEvent(final  List<String> ids): _ids = ids;
  

 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of SearchEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SearchChangesReceivedEventCopyWith<SearchChangesReceivedEvent> get copyWith => _$SearchChangesReceivedEventCopyWithImpl<SearchChangesReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchChangesReceivedEvent&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'SearchEvent.changesReceived(ids: $ids)';
}


}

/// @nodoc
abstract mixin class $SearchChangesReceivedEventCopyWith<$Res> implements $SearchEventCopyWith<$Res> {
  factory $SearchChangesReceivedEventCopyWith(SearchChangesReceivedEvent value, $Res Function(SearchChangesReceivedEvent) _then) = _$SearchChangesReceivedEventCopyWithImpl;
@useResult
$Res call({
 List<String> ids
});




}
/// @nodoc
class _$SearchChangesReceivedEventCopyWithImpl<$Res>
    implements $SearchChangesReceivedEventCopyWith<$Res> {
  _$SearchChangesReceivedEventCopyWithImpl(this._self, this._then);

  final SearchChangesReceivedEvent _self;
  final $Res Function(SearchChangesReceivedEvent) _then;

/// Create a copy of SearchEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ids = null,}) {
  return _then(SearchChangesReceivedEvent(
null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class SearchRetriedEvent implements SearchEvent {
  const SearchRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchEvent.retried()';
}


}




/// @nodoc
mixin _$SearchState {

/// The words being looked for.
 String get query;/// Where to look.
 SearchScopeKind get scope;/// The notebook the search was opened from, if any: what the notebook
/// scope looks in.
 String? get notebookId; SearchStatus get status;/// What was found for [query] in [scope], best match first.
 List<Hit> get hits;/// Notes found with the local semantic model, when it is available.
 List<RelatedRow> get meaningHits;/// Why the last search failed, while [status] is failed.
 ApplicationError? get error;
/// Create a copy of SearchState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SearchStateCopyWith<SearchState> get copyWith => _$SearchStateCopyWithImpl<SearchState>(this as SearchState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchState&&(identical(other.query, query) || other.query == query)&&(identical(other.scope, scope) || other.scope == scope)&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.hits, hits)&&const DeepCollectionEquality().equals(other.meaningHits, meaningHits)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,query,scope,notebookId,status,const DeepCollectionEquality().hash(hits),const DeepCollectionEquality().hash(meaningHits),error);

@override
String toString() {
  return 'SearchState(query: $query, scope: $scope, notebookId: $notebookId, status: $status, hits: $hits, meaningHits: $meaningHits, error: $error)';
}


}

/// @nodoc
abstract mixin class $SearchStateCopyWith<$Res>  {
  factory $SearchStateCopyWith(SearchState value, $Res Function(SearchState) _then) = _$SearchStateCopyWithImpl;
@useResult
$Res call({
 String query, SearchScopeKind scope, String? notebookId, SearchStatus status, List<Hit> hits, List<RelatedRow> meaningHits, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$SearchStateCopyWithImpl<$Res>
    implements $SearchStateCopyWith<$Res> {
  _$SearchStateCopyWithImpl(this._self, this._then);

  final SearchState _self;
  final $Res Function(SearchState) _then;

/// Create a copy of SearchState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? query = null,Object? scope = null,Object? notebookId = freezed,Object? status = null,Object? hits = null,Object? meaningHits = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
query: null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,scope: null == scope ? _self.scope : scope // ignore: cast_nullable_to_non_nullable
as SearchScopeKind,notebookId: freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as SearchStatus,hits: null == hits ? _self.hits : hits // ignore: cast_nullable_to_non_nullable
as List<Hit>,meaningHits: null == meaningHits ? _self.meaningHits : meaningHits // ignore: cast_nullable_to_non_nullable
as List<RelatedRow>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of SearchState
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


/// Adds pattern-matching-related methods to [SearchState].
extension SearchStatePatterns on SearchState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SearchState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SearchState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SearchState value)  $default,){
final _that = this;
switch (_that) {
case _SearchState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SearchState value)?  $default,){
final _that = this;
switch (_that) {
case _SearchState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String query,  SearchScopeKind scope,  String? notebookId,  SearchStatus status,  List<Hit> hits,  List<RelatedRow> meaningHits,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SearchState() when $default != null:
return $default(_that.query,_that.scope,_that.notebookId,_that.status,_that.hits,_that.meaningHits,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String query,  SearchScopeKind scope,  String? notebookId,  SearchStatus status,  List<Hit> hits,  List<RelatedRow> meaningHits,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _SearchState():
return $default(_that.query,_that.scope,_that.notebookId,_that.status,_that.hits,_that.meaningHits,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String query,  SearchScopeKind scope,  String? notebookId,  SearchStatus status,  List<Hit> hits,  List<RelatedRow> meaningHits,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _SearchState() when $default != null:
return $default(_that.query,_that.scope,_that.notebookId,_that.status,_that.hits,_that.meaningHits,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _SearchState extends SearchState {
  const _SearchState({this.query = '', this.scope = SearchScopeKind.all, this.notebookId, this.status = SearchStatus.idle, final  List<Hit> hits = const <Hit>[], final  List<RelatedRow> meaningHits = const <RelatedRow>[], this.error}): _hits = hits,_meaningHits = meaningHits,super._();
  

/// The words being looked for.
@override@JsonKey() final  String query;
/// Where to look.
@override@JsonKey() final  SearchScopeKind scope;
/// The notebook the search was opened from, if any: what the notebook
/// scope looks in.
@override final  String? notebookId;
@override@JsonKey() final  SearchStatus status;
/// What was found for [query] in [scope], best match first.
 final  List<Hit> _hits;
/// What was found for [query] in [scope], best match first.
@override@JsonKey() List<Hit> get hits {
  if (_hits is EqualUnmodifiableListView) return _hits;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_hits);
}

/// Notes found with the local semantic model, when it is available.
 final  List<RelatedRow> _meaningHits;
/// Notes found with the local semantic model, when it is available.
@override@JsonKey() List<RelatedRow> get meaningHits {
  if (_meaningHits is EqualUnmodifiableListView) return _meaningHits;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_meaningHits);
}

/// Why the last search failed, while [status] is failed.
@override final  ApplicationError? error;

/// Create a copy of SearchState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SearchStateCopyWith<_SearchState> get copyWith => __$SearchStateCopyWithImpl<_SearchState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SearchState&&(identical(other.query, query) || other.query == query)&&(identical(other.scope, scope) || other.scope == scope)&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._hits, _hits)&&const DeepCollectionEquality().equals(other._meaningHits, _meaningHits)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,query,scope,notebookId,status,const DeepCollectionEquality().hash(_hits),const DeepCollectionEquality().hash(_meaningHits),error);

@override
String toString() {
  return 'SearchState(query: $query, scope: $scope, notebookId: $notebookId, status: $status, hits: $hits, meaningHits: $meaningHits, error: $error)';
}


}

/// @nodoc
abstract mixin class _$SearchStateCopyWith<$Res> implements $SearchStateCopyWith<$Res> {
  factory _$SearchStateCopyWith(_SearchState value, $Res Function(_SearchState) _then) = __$SearchStateCopyWithImpl;
@override @useResult
$Res call({
 String query, SearchScopeKind scope, String? notebookId, SearchStatus status, List<Hit> hits, List<RelatedRow> meaningHits, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$SearchStateCopyWithImpl<$Res>
    implements _$SearchStateCopyWith<$Res> {
  __$SearchStateCopyWithImpl(this._self, this._then);

  final _SearchState _self;
  final $Res Function(_SearchState) _then;

/// Create a copy of SearchState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? query = null,Object? scope = null,Object? notebookId = freezed,Object? status = null,Object? hits = null,Object? meaningHits = null,Object? error = freezed,}) {
  return _then(_SearchState(
query: null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,scope: null == scope ? _self.scope : scope // ignore: cast_nullable_to_non_nullable
as SearchScopeKind,notebookId: freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as SearchStatus,hits: null == hits ? _self._hits : hits // ignore: cast_nullable_to_non_nullable
as List<Hit>,meaningHits: null == meaningHits ? _self._meaningHits : meaningHits // ignore: cast_nullable_to_non_nullable
as List<RelatedRow>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of SearchState
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
