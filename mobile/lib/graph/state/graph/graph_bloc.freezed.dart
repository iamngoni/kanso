// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'graph_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$GraphEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GraphEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'GraphEvent()';
}


}

/// @nodoc
class $GraphEventCopyWith<$Res>  {
$GraphEventCopyWith(GraphEvent _, $Res Function(GraphEvent) __);
}


/// Adds pattern-matching-related methods to [GraphEvent].
extension GraphEventPatterns on GraphEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( GraphStartedEvent value)?  started,TResult Function( GraphFilterSelectedEvent value)?  filterSelected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case GraphStartedEvent() when started != null:
return started(_that);case GraphFilterSelectedEvent() when filterSelected != null:
return filterSelected(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( GraphStartedEvent value)  started,required TResult Function( GraphFilterSelectedEvent value)  filterSelected,}){
final _that = this;
switch (_that) {
case GraphStartedEvent():
return started(_that);case GraphFilterSelectedEvent():
return filterSelected(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( GraphStartedEvent value)?  started,TResult? Function( GraphFilterSelectedEvent value)?  filterSelected,}){
final _that = this;
switch (_that) {
case GraphStartedEvent() when started != null:
return started(_that);case GraphFilterSelectedEvent() when filterSelected != null:
return filterSelected(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( GraphRequest request)?  started,TResult Function( GraphFilter filter)?  filterSelected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case GraphStartedEvent() when started != null:
return started(_that.request);case GraphFilterSelectedEvent() when filterSelected != null:
return filterSelected(_that.filter);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( GraphRequest request)  started,required TResult Function( GraphFilter filter)  filterSelected,}) {final _that = this;
switch (_that) {
case GraphStartedEvent():
return started(_that.request);case GraphFilterSelectedEvent():
return filterSelected(_that.filter);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( GraphRequest request)?  started,TResult? Function( GraphFilter filter)?  filterSelected,}) {final _that = this;
switch (_that) {
case GraphStartedEvent() when started != null:
return started(_that.request);case GraphFilterSelectedEvent() when filterSelected != null:
return filterSelected(_that.filter);case _:
  return null;

}
}

}

/// @nodoc


class GraphStartedEvent implements GraphEvent {
  const GraphStartedEvent(this.request);
  

 final  GraphRequest request;

/// Create a copy of GraphEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$GraphStartedEventCopyWith<GraphStartedEvent> get copyWith => _$GraphStartedEventCopyWithImpl<GraphStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GraphStartedEvent&&(identical(other.request, request) || other.request == request));
}


@override
int get hashCode => Object.hash(runtimeType,request);

@override
String toString() {
  return 'GraphEvent.started(request: $request)';
}


}

/// @nodoc
abstract mixin class $GraphStartedEventCopyWith<$Res> implements $GraphEventCopyWith<$Res> {
  factory $GraphStartedEventCopyWith(GraphStartedEvent value, $Res Function(GraphStartedEvent) _then) = _$GraphStartedEventCopyWithImpl;
@useResult
$Res call({
 GraphRequest request
});


$GraphRequestCopyWith<$Res> get request;

}
/// @nodoc
class _$GraphStartedEventCopyWithImpl<$Res>
    implements $GraphStartedEventCopyWith<$Res> {
  _$GraphStartedEventCopyWithImpl(this._self, this._then);

  final GraphStartedEvent _self;
  final $Res Function(GraphStartedEvent) _then;

/// Create a copy of GraphEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? request = null,}) {
  return _then(GraphStartedEvent(
null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as GraphRequest,
  ));
}

/// Create a copy of GraphEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$GraphRequestCopyWith<$Res> get request {
  
  return $GraphRequestCopyWith<$Res>(_self.request, (value) {
    return _then(_self.copyWith(request: value));
  });
}
}

/// @nodoc


class GraphFilterSelectedEvent implements GraphEvent {
  const GraphFilterSelectedEvent(this.filter);
  

 final  GraphFilter filter;

/// Create a copy of GraphEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$GraphFilterSelectedEventCopyWith<GraphFilterSelectedEvent> get copyWith => _$GraphFilterSelectedEventCopyWithImpl<GraphFilterSelectedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GraphFilterSelectedEvent&&(identical(other.filter, filter) || other.filter == filter));
}


@override
int get hashCode => Object.hash(runtimeType,filter);

@override
String toString() {
  return 'GraphEvent.filterSelected(filter: $filter)';
}


}

/// @nodoc
abstract mixin class $GraphFilterSelectedEventCopyWith<$Res> implements $GraphEventCopyWith<$Res> {
  factory $GraphFilterSelectedEventCopyWith(GraphFilterSelectedEvent value, $Res Function(GraphFilterSelectedEvent) _then) = _$GraphFilterSelectedEventCopyWithImpl;
@useResult
$Res call({
 GraphFilter filter
});


$GraphFilterCopyWith<$Res> get filter;

}
/// @nodoc
class _$GraphFilterSelectedEventCopyWithImpl<$Res>
    implements $GraphFilterSelectedEventCopyWith<$Res> {
  _$GraphFilterSelectedEventCopyWithImpl(this._self, this._then);

  final GraphFilterSelectedEvent _self;
  final $Res Function(GraphFilterSelectedEvent) _then;

/// Create a copy of GraphEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? filter = null,}) {
  return _then(GraphFilterSelectedEvent(
null == filter ? _self.filter : filter // ignore: cast_nullable_to_non_nullable
as GraphFilter,
  ));
}

/// Create a copy of GraphEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$GraphFilterCopyWith<$Res> get filter {
  
  return $GraphFilterCopyWith<$Res>(_self.filter, (value) {
    return _then(_self.copyWith(filter: value));
  });
}
}

/// @nodoc
mixin _$GraphState {

 GraphRequest get request; GraphLayout? get layout; bool get loading; bool get failed;
/// Create a copy of GraphState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$GraphStateCopyWith<GraphState> get copyWith => _$GraphStateCopyWithImpl<GraphState>(this as GraphState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GraphState&&(identical(other.request, request) || other.request == request)&&(identical(other.layout, layout) || other.layout == layout)&&(identical(other.loading, loading) || other.loading == loading)&&(identical(other.failed, failed) || other.failed == failed));
}


@override
int get hashCode => Object.hash(runtimeType,request,layout,loading,failed);

@override
String toString() {
  return 'GraphState(request: $request, layout: $layout, loading: $loading, failed: $failed)';
}


}

/// @nodoc
abstract mixin class $GraphStateCopyWith<$Res>  {
  factory $GraphStateCopyWith(GraphState value, $Res Function(GraphState) _then) = _$GraphStateCopyWithImpl;
@useResult
$Res call({
 GraphRequest request, GraphLayout? layout, bool loading, bool failed
});


$GraphRequestCopyWith<$Res> get request;

}
/// @nodoc
class _$GraphStateCopyWithImpl<$Res>
    implements $GraphStateCopyWith<$Res> {
  _$GraphStateCopyWithImpl(this._self, this._then);

  final GraphState _self;
  final $Res Function(GraphState) _then;

/// Create a copy of GraphState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? request = null,Object? layout = freezed,Object? loading = null,Object? failed = null,}) {
  return _then(_self.copyWith(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as GraphRequest,layout: freezed == layout ? _self.layout : layout // ignore: cast_nullable_to_non_nullable
as GraphLayout?,loading: null == loading ? _self.loading : loading // ignore: cast_nullable_to_non_nullable
as bool,failed: null == failed ? _self.failed : failed // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}
/// Create a copy of GraphState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$GraphRequestCopyWith<$Res> get request {
  
  return $GraphRequestCopyWith<$Res>(_self.request, (value) {
    return _then(_self.copyWith(request: value));
  });
}
}


/// Adds pattern-matching-related methods to [GraphState].
extension GraphStatePatterns on GraphState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _GraphState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _GraphState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _GraphState value)  $default,){
final _that = this;
switch (_that) {
case _GraphState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _GraphState value)?  $default,){
final _that = this;
switch (_that) {
case _GraphState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( GraphRequest request,  GraphLayout? layout,  bool loading,  bool failed)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _GraphState() when $default != null:
return $default(_that.request,_that.layout,_that.loading,_that.failed);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( GraphRequest request,  GraphLayout? layout,  bool loading,  bool failed)  $default,) {final _that = this;
switch (_that) {
case _GraphState():
return $default(_that.request,_that.layout,_that.loading,_that.failed);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( GraphRequest request,  GraphLayout? layout,  bool loading,  bool failed)?  $default,) {final _that = this;
switch (_that) {
case _GraphState() when $default != null:
return $default(_that.request,_that.layout,_that.loading,_that.failed);case _:
  return null;

}
}

}

/// @nodoc


class _GraphState implements GraphState {
  const _GraphState({this.request = const GraphRequest.library_(filter: GraphFilter.all()), this.layout, this.loading = false, this.failed = false});
  

@override@JsonKey() final  GraphRequest request;
@override final  GraphLayout? layout;
@override@JsonKey() final  bool loading;
@override@JsonKey() final  bool failed;

/// Create a copy of GraphState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$GraphStateCopyWith<_GraphState> get copyWith => __$GraphStateCopyWithImpl<_GraphState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _GraphState&&(identical(other.request, request) || other.request == request)&&(identical(other.layout, layout) || other.layout == layout)&&(identical(other.loading, loading) || other.loading == loading)&&(identical(other.failed, failed) || other.failed == failed));
}


@override
int get hashCode => Object.hash(runtimeType,request,layout,loading,failed);

@override
String toString() {
  return 'GraphState(request: $request, layout: $layout, loading: $loading, failed: $failed)';
}


}

/// @nodoc
abstract mixin class _$GraphStateCopyWith<$Res> implements $GraphStateCopyWith<$Res> {
  factory _$GraphStateCopyWith(_GraphState value, $Res Function(_GraphState) _then) = __$GraphStateCopyWithImpl;
@override @useResult
$Res call({
 GraphRequest request, GraphLayout? layout, bool loading, bool failed
});


@override $GraphRequestCopyWith<$Res> get request;

}
/// @nodoc
class __$GraphStateCopyWithImpl<$Res>
    implements _$GraphStateCopyWith<$Res> {
  __$GraphStateCopyWithImpl(this._self, this._then);

  final _GraphState _self;
  final $Res Function(_GraphState) _then;

/// Create a copy of GraphState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? request = null,Object? layout = freezed,Object? loading = null,Object? failed = null,}) {
  return _then(_GraphState(
request: null == request ? _self.request : request // ignore: cast_nullable_to_non_nullable
as GraphRequest,layout: freezed == layout ? _self.layout : layout // ignore: cast_nullable_to_non_nullable
as GraphLayout?,loading: null == loading ? _self.loading : loading // ignore: cast_nullable_to_non_nullable
as bool,failed: null == failed ? _self.failed : failed // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

/// Create a copy of GraphState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$GraphRequestCopyWith<$Res> get request {
  
  return $GraphRequestCopyWith<$Res>(_self.request, (value) {
    return _then(_self.copyWith(request: value));
  });
}
}

// dart format on
