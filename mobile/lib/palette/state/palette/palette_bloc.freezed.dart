// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'palette_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PaletteEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PaletteEvent()';
}


}

/// @nodoc
class $PaletteEventCopyWith<$Res>  {
$PaletteEventCopyWith(PaletteEvent _, $Res Function(PaletteEvent) __);
}


/// Adds pattern-matching-related methods to [PaletteEvent].
extension PaletteEventPatterns on PaletteEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PaletteStartedEvent value)?  started,TResult Function( PaletteQueryChangedEvent value)?  queryChanged,TResult Function( PaletteSearchDueEvent value)?  searchDue,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PaletteStartedEvent() when started != null:
return started(_that);case PaletteQueryChangedEvent() when queryChanged != null:
return queryChanged(_that);case PaletteSearchDueEvent() when searchDue != null:
return searchDue(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PaletteStartedEvent value)  started,required TResult Function( PaletteQueryChangedEvent value)  queryChanged,required TResult Function( PaletteSearchDueEvent value)  searchDue,}){
final _that = this;
switch (_that) {
case PaletteStartedEvent():
return started(_that);case PaletteQueryChangedEvent():
return queryChanged(_that);case PaletteSearchDueEvent():
return searchDue(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PaletteStartedEvent value)?  started,TResult? Function( PaletteQueryChangedEvent value)?  queryChanged,TResult? Function( PaletteSearchDueEvent value)?  searchDue,}){
final _that = this;
switch (_that) {
case PaletteStartedEvent() when started != null:
return started(_that);case PaletteQueryChangedEvent() when queryChanged != null:
return queryChanged(_that);case PaletteSearchDueEvent() when searchDue != null:
return searchDue(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( PaletteContext context)?  started,TResult Function( String query)?  queryChanged,TResult Function( String query)?  searchDue,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PaletteStartedEvent() when started != null:
return started(_that.context);case PaletteQueryChangedEvent() when queryChanged != null:
return queryChanged(_that.query);case PaletteSearchDueEvent() when searchDue != null:
return searchDue(_that.query);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( PaletteContext context)  started,required TResult Function( String query)  queryChanged,required TResult Function( String query)  searchDue,}) {final _that = this;
switch (_that) {
case PaletteStartedEvent():
return started(_that.context);case PaletteQueryChangedEvent():
return queryChanged(_that.query);case PaletteSearchDueEvent():
return searchDue(_that.query);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( PaletteContext context)?  started,TResult? Function( String query)?  queryChanged,TResult? Function( String query)?  searchDue,}) {final _that = this;
switch (_that) {
case PaletteStartedEvent() when started != null:
return started(_that.context);case PaletteQueryChangedEvent() when queryChanged != null:
return queryChanged(_that.query);case PaletteSearchDueEvent() when searchDue != null:
return searchDue(_that.query);case _:
  return null;

}
}

}

/// @nodoc


class PaletteStartedEvent implements PaletteEvent {
  const PaletteStartedEvent(this.context);
  

 final  PaletteContext context;

/// Create a copy of PaletteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PaletteStartedEventCopyWith<PaletteStartedEvent> get copyWith => _$PaletteStartedEventCopyWithImpl<PaletteStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteStartedEvent&&(identical(other.context, context) || other.context == context));
}


@override
int get hashCode => Object.hash(runtimeType,context);

@override
String toString() {
  return 'PaletteEvent.started(context: $context)';
}


}

/// @nodoc
abstract mixin class $PaletteStartedEventCopyWith<$Res> implements $PaletteEventCopyWith<$Res> {
  factory $PaletteStartedEventCopyWith(PaletteStartedEvent value, $Res Function(PaletteStartedEvent) _then) = _$PaletteStartedEventCopyWithImpl;
@useResult
$Res call({
 PaletteContext context
});




}
/// @nodoc
class _$PaletteStartedEventCopyWithImpl<$Res>
    implements $PaletteStartedEventCopyWith<$Res> {
  _$PaletteStartedEventCopyWithImpl(this._self, this._then);

  final PaletteStartedEvent _self;
  final $Res Function(PaletteStartedEvent) _then;

/// Create a copy of PaletteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? context = null,}) {
  return _then(PaletteStartedEvent(
null == context ? _self.context : context // ignore: cast_nullable_to_non_nullable
as PaletteContext,
  ));
}


}

/// @nodoc


class PaletteQueryChangedEvent implements PaletteEvent {
  const PaletteQueryChangedEvent(this.query);
  

 final  String query;

/// Create a copy of PaletteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PaletteQueryChangedEventCopyWith<PaletteQueryChangedEvent> get copyWith => _$PaletteQueryChangedEventCopyWithImpl<PaletteQueryChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteQueryChangedEvent&&(identical(other.query, query) || other.query == query));
}


@override
int get hashCode => Object.hash(runtimeType,query);

@override
String toString() {
  return 'PaletteEvent.queryChanged(query: $query)';
}


}

/// @nodoc
abstract mixin class $PaletteQueryChangedEventCopyWith<$Res> implements $PaletteEventCopyWith<$Res> {
  factory $PaletteQueryChangedEventCopyWith(PaletteQueryChangedEvent value, $Res Function(PaletteQueryChangedEvent) _then) = _$PaletteQueryChangedEventCopyWithImpl;
@useResult
$Res call({
 String query
});




}
/// @nodoc
class _$PaletteQueryChangedEventCopyWithImpl<$Res>
    implements $PaletteQueryChangedEventCopyWith<$Res> {
  _$PaletteQueryChangedEventCopyWithImpl(this._self, this._then);

  final PaletteQueryChangedEvent _self;
  final $Res Function(PaletteQueryChangedEvent) _then;

/// Create a copy of PaletteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? query = null,}) {
  return _then(PaletteQueryChangedEvent(
null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PaletteSearchDueEvent implements PaletteEvent {
  const PaletteSearchDueEvent(this.query);
  

 final  String query;

/// Create a copy of PaletteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PaletteSearchDueEventCopyWith<PaletteSearchDueEvent> get copyWith => _$PaletteSearchDueEventCopyWithImpl<PaletteSearchDueEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteSearchDueEvent&&(identical(other.query, query) || other.query == query));
}


@override
int get hashCode => Object.hash(runtimeType,query);

@override
String toString() {
  return 'PaletteEvent.searchDue(query: $query)';
}


}

/// @nodoc
abstract mixin class $PaletteSearchDueEventCopyWith<$Res> implements $PaletteEventCopyWith<$Res> {
  factory $PaletteSearchDueEventCopyWith(PaletteSearchDueEvent value, $Res Function(PaletteSearchDueEvent) _then) = _$PaletteSearchDueEventCopyWithImpl;
@useResult
$Res call({
 String query
});




}
/// @nodoc
class _$PaletteSearchDueEventCopyWithImpl<$Res>
    implements $PaletteSearchDueEventCopyWith<$Res> {
  _$PaletteSearchDueEventCopyWithImpl(this._self, this._then);

  final PaletteSearchDueEvent _self;
  final $Res Function(PaletteSearchDueEvent) _then;

/// Create a copy of PaletteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? query = null,}) {
  return _then(PaletteSearchDueEvent(
null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$PaletteState {

/// What is typed.
 String get query;/// Where the results are.
 PaletteStatus get status;/// What matches, in the order the palette shows it.
 List<PaletteSection> get sections;/// Why the last search failed, to say instead of the list.
 String? get problem;
/// Create a copy of PaletteState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PaletteStateCopyWith<PaletteState> get copyWith => _$PaletteStateCopyWithImpl<PaletteState>(this as PaletteState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteState&&(identical(other.query, query) || other.query == query)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.sections, sections)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,query,status,const DeepCollectionEquality().hash(sections),problem);

@override
String toString() {
  return 'PaletteState(query: $query, status: $status, sections: $sections, problem: $problem)';
}


}

/// @nodoc
abstract mixin class $PaletteStateCopyWith<$Res>  {
  factory $PaletteStateCopyWith(PaletteState value, $Res Function(PaletteState) _then) = _$PaletteStateCopyWithImpl;
@useResult
$Res call({
 String query, PaletteStatus status, List<PaletteSection> sections, String? problem
});




}
/// @nodoc
class _$PaletteStateCopyWithImpl<$Res>
    implements $PaletteStateCopyWith<$Res> {
  _$PaletteStateCopyWithImpl(this._self, this._then);

  final PaletteState _self;
  final $Res Function(PaletteState) _then;

/// Create a copy of PaletteState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? query = null,Object? status = null,Object? sections = null,Object? problem = freezed,}) {
  return _then(_self.copyWith(
query: null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as PaletteStatus,sections: null == sections ? _self.sections : sections // ignore: cast_nullable_to_non_nullable
as List<PaletteSection>,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [PaletteState].
extension PaletteStatePatterns on PaletteState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PaletteState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PaletteState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PaletteState value)  $default,){
final _that = this;
switch (_that) {
case _PaletteState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PaletteState value)?  $default,){
final _that = this;
switch (_that) {
case _PaletteState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String query,  PaletteStatus status,  List<PaletteSection> sections,  String? problem)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PaletteState() when $default != null:
return $default(_that.query,_that.status,_that.sections,_that.problem);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String query,  PaletteStatus status,  List<PaletteSection> sections,  String? problem)  $default,) {final _that = this;
switch (_that) {
case _PaletteState():
return $default(_that.query,_that.status,_that.sections,_that.problem);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String query,  PaletteStatus status,  List<PaletteSection> sections,  String? problem)?  $default,) {final _that = this;
switch (_that) {
case _PaletteState() when $default != null:
return $default(_that.query,_that.status,_that.sections,_that.problem);case _:
  return null;

}
}

}

/// @nodoc


class _PaletteState extends PaletteState {
  const _PaletteState({this.query = '', this.status = PaletteStatus.idle, final  List<PaletteSection> sections = const <PaletteSection>[], this.problem}): _sections = sections,super._();
  

/// What is typed.
@override@JsonKey() final  String query;
/// Where the results are.
@override@JsonKey() final  PaletteStatus status;
/// What matches, in the order the palette shows it.
 final  List<PaletteSection> _sections;
/// What matches, in the order the palette shows it.
@override@JsonKey() List<PaletteSection> get sections {
  if (_sections is EqualUnmodifiableListView) return _sections;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_sections);
}

/// Why the last search failed, to say instead of the list.
@override final  String? problem;

/// Create a copy of PaletteState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PaletteStateCopyWith<_PaletteState> get copyWith => __$PaletteStateCopyWithImpl<_PaletteState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PaletteState&&(identical(other.query, query) || other.query == query)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._sections, _sections)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,query,status,const DeepCollectionEquality().hash(_sections),problem);

@override
String toString() {
  return 'PaletteState(query: $query, status: $status, sections: $sections, problem: $problem)';
}


}

/// @nodoc
abstract mixin class _$PaletteStateCopyWith<$Res> implements $PaletteStateCopyWith<$Res> {
  factory _$PaletteStateCopyWith(_PaletteState value, $Res Function(_PaletteState) _then) = __$PaletteStateCopyWithImpl;
@override @useResult
$Res call({
 String query, PaletteStatus status, List<PaletteSection> sections, String? problem
});




}
/// @nodoc
class __$PaletteStateCopyWithImpl<$Res>
    implements _$PaletteStateCopyWith<$Res> {
  __$PaletteStateCopyWithImpl(this._self, this._then);

  final _PaletteState _self;
  final $Res Function(_PaletteState) _then;

/// Create a copy of PaletteState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? query = null,Object? status = null,Object? sections = null,Object? problem = freezed,}) {
  return _then(_PaletteState(
query: null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as PaletteStatus,sections: null == sections ? _self._sections : sections // ignore: cast_nullable_to_non_nullable
as List<PaletteSection>,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
