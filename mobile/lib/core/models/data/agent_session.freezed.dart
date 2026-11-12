// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'agent_session.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AgentSession {

/// The task's id for this launch: Review, Stop and Apply take it.
 int get id;/// The session's own id, which a restart keeps.
 String get key;/// The goal cut to one line.
 String get title; AgentTaskPhase get phase; String get agent;/// The question, the error or the number of changes, by [phase].
 String get detail; AgentSessionDot get dot;/// New items since the session was last opened.
 bool get unread; int get lastActivityMs; int get notesRead; int get changes; bool get canStop; bool get canReview;/// Whether the composer can send now.
 bool get canFollowUp;
/// Create a copy of AgentSession
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentSessionCopyWith<AgentSession> get copyWith => _$AgentSessionCopyWithImpl<AgentSession>(this as AgentSession, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSession&&(identical(other.id, id) || other.id == id)&&(identical(other.key, key) || other.key == key)&&(identical(other.title, title) || other.title == title)&&(identical(other.phase, phase) || other.phase == phase)&&(identical(other.agent, agent) || other.agent == agent)&&(identical(other.detail, detail) || other.detail == detail)&&(identical(other.dot, dot) || other.dot == dot)&&(identical(other.unread, unread) || other.unread == unread)&&(identical(other.lastActivityMs, lastActivityMs) || other.lastActivityMs == lastActivityMs)&&(identical(other.notesRead, notesRead) || other.notesRead == notesRead)&&(identical(other.changes, changes) || other.changes == changes)&&(identical(other.canStop, canStop) || other.canStop == canStop)&&(identical(other.canReview, canReview) || other.canReview == canReview)&&(identical(other.canFollowUp, canFollowUp) || other.canFollowUp == canFollowUp));
}


@override
int get hashCode => Object.hash(runtimeType,id,key,title,phase,agent,detail,dot,unread,lastActivityMs,notesRead,changes,canStop,canReview,canFollowUp);

@override
String toString() {
  return 'AgentSession(id: $id, key: $key, title: $title, phase: $phase, agent: $agent, detail: $detail, dot: $dot, unread: $unread, lastActivityMs: $lastActivityMs, notesRead: $notesRead, changes: $changes, canStop: $canStop, canReview: $canReview, canFollowUp: $canFollowUp)';
}


}

/// @nodoc
abstract mixin class $AgentSessionCopyWith<$Res>  {
  factory $AgentSessionCopyWith(AgentSession value, $Res Function(AgentSession) _then) = _$AgentSessionCopyWithImpl;
@useResult
$Res call({
 int id, String key, String title, AgentTaskPhase phase, String agent, String detail, AgentSessionDot dot, bool unread, int lastActivityMs, int notesRead, int changes, bool canStop, bool canReview, bool canFollowUp
});




}
/// @nodoc
class _$AgentSessionCopyWithImpl<$Res>
    implements $AgentSessionCopyWith<$Res> {
  _$AgentSessionCopyWithImpl(this._self, this._then);

  final AgentSession _self;
  final $Res Function(AgentSession) _then;

/// Create a copy of AgentSession
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? key = null,Object? title = null,Object? phase = null,Object? agent = null,Object? detail = null,Object? dot = null,Object? unread = null,Object? lastActivityMs = null,Object? notesRead = null,Object? changes = null,Object? canStop = null,Object? canReview = null,Object? canFollowUp = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as int,key: null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as AgentTaskPhase,agent: null == agent ? _self.agent : agent // ignore: cast_nullable_to_non_nullable
as String,detail: null == detail ? _self.detail : detail // ignore: cast_nullable_to_non_nullable
as String,dot: null == dot ? _self.dot : dot // ignore: cast_nullable_to_non_nullable
as AgentSessionDot,unread: null == unread ? _self.unread : unread // ignore: cast_nullable_to_non_nullable
as bool,lastActivityMs: null == lastActivityMs ? _self.lastActivityMs : lastActivityMs // ignore: cast_nullable_to_non_nullable
as int,notesRead: null == notesRead ? _self.notesRead : notesRead // ignore: cast_nullable_to_non_nullable
as int,changes: null == changes ? _self.changes : changes // ignore: cast_nullable_to_non_nullable
as int,canStop: null == canStop ? _self.canStop : canStop // ignore: cast_nullable_to_non_nullable
as bool,canReview: null == canReview ? _self.canReview : canReview // ignore: cast_nullable_to_non_nullable
as bool,canFollowUp: null == canFollowUp ? _self.canFollowUp : canFollowUp // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [AgentSession].
extension AgentSessionPatterns on AgentSession {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AgentSession value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AgentSession() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AgentSession value)  $default,){
final _that = this;
switch (_that) {
case _AgentSession():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AgentSession value)?  $default,){
final _that = this;
switch (_that) {
case _AgentSession() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int id,  String key,  String title,  AgentTaskPhase phase,  String agent,  String detail,  AgentSessionDot dot,  bool unread,  int lastActivityMs,  int notesRead,  int changes,  bool canStop,  bool canReview,  bool canFollowUp)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AgentSession() when $default != null:
return $default(_that.id,_that.key,_that.title,_that.phase,_that.agent,_that.detail,_that.dot,_that.unread,_that.lastActivityMs,_that.notesRead,_that.changes,_that.canStop,_that.canReview,_that.canFollowUp);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int id,  String key,  String title,  AgentTaskPhase phase,  String agent,  String detail,  AgentSessionDot dot,  bool unread,  int lastActivityMs,  int notesRead,  int changes,  bool canStop,  bool canReview,  bool canFollowUp)  $default,) {final _that = this;
switch (_that) {
case _AgentSession():
return $default(_that.id,_that.key,_that.title,_that.phase,_that.agent,_that.detail,_that.dot,_that.unread,_that.lastActivityMs,_that.notesRead,_that.changes,_that.canStop,_that.canReview,_that.canFollowUp);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int id,  String key,  String title,  AgentTaskPhase phase,  String agent,  String detail,  AgentSessionDot dot,  bool unread,  int lastActivityMs,  int notesRead,  int changes,  bool canStop,  bool canReview,  bool canFollowUp)?  $default,) {final _that = this;
switch (_that) {
case _AgentSession() when $default != null:
return $default(_that.id,_that.key,_that.title,_that.phase,_that.agent,_that.detail,_that.dot,_that.unread,_that.lastActivityMs,_that.notesRead,_that.changes,_that.canStop,_that.canReview,_that.canFollowUp);case _:
  return null;

}
}

}

/// @nodoc


class _AgentSession extends AgentSession {
  const _AgentSession({required this.id, required this.key, required this.title, required this.phase, this.agent = 'Kanso AI', this.detail = '', this.dot = AgentSessionDot.idle, this.unread = false, this.lastActivityMs = 0, this.notesRead = 0, this.changes = 0, this.canStop = false, this.canReview = false, this.canFollowUp = false}): super._();
  

/// The task's id for this launch: Review, Stop and Apply take it.
@override final  int id;
/// The session's own id, which a restart keeps.
@override final  String key;
/// The goal cut to one line.
@override final  String title;
@override final  AgentTaskPhase phase;
@override@JsonKey() final  String agent;
/// The question, the error or the number of changes, by [phase].
@override@JsonKey() final  String detail;
@override@JsonKey() final  AgentSessionDot dot;
/// New items since the session was last opened.
@override@JsonKey() final  bool unread;
@override@JsonKey() final  int lastActivityMs;
@override@JsonKey() final  int notesRead;
@override@JsonKey() final  int changes;
@override@JsonKey() final  bool canStop;
@override@JsonKey() final  bool canReview;
/// Whether the composer can send now.
@override@JsonKey() final  bool canFollowUp;

/// Create a copy of AgentSession
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AgentSessionCopyWith<_AgentSession> get copyWith => __$AgentSessionCopyWithImpl<_AgentSession>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AgentSession&&(identical(other.id, id) || other.id == id)&&(identical(other.key, key) || other.key == key)&&(identical(other.title, title) || other.title == title)&&(identical(other.phase, phase) || other.phase == phase)&&(identical(other.agent, agent) || other.agent == agent)&&(identical(other.detail, detail) || other.detail == detail)&&(identical(other.dot, dot) || other.dot == dot)&&(identical(other.unread, unread) || other.unread == unread)&&(identical(other.lastActivityMs, lastActivityMs) || other.lastActivityMs == lastActivityMs)&&(identical(other.notesRead, notesRead) || other.notesRead == notesRead)&&(identical(other.changes, changes) || other.changes == changes)&&(identical(other.canStop, canStop) || other.canStop == canStop)&&(identical(other.canReview, canReview) || other.canReview == canReview)&&(identical(other.canFollowUp, canFollowUp) || other.canFollowUp == canFollowUp));
}


@override
int get hashCode => Object.hash(runtimeType,id,key,title,phase,agent,detail,dot,unread,lastActivityMs,notesRead,changes,canStop,canReview,canFollowUp);

@override
String toString() {
  return 'AgentSession(id: $id, key: $key, title: $title, phase: $phase, agent: $agent, detail: $detail, dot: $dot, unread: $unread, lastActivityMs: $lastActivityMs, notesRead: $notesRead, changes: $changes, canStop: $canStop, canReview: $canReview, canFollowUp: $canFollowUp)';
}


}

/// @nodoc
abstract mixin class _$AgentSessionCopyWith<$Res> implements $AgentSessionCopyWith<$Res> {
  factory _$AgentSessionCopyWith(_AgentSession value, $Res Function(_AgentSession) _then) = __$AgentSessionCopyWithImpl;
@override @useResult
$Res call({
 int id, String key, String title, AgentTaskPhase phase, String agent, String detail, AgentSessionDot dot, bool unread, int lastActivityMs, int notesRead, int changes, bool canStop, bool canReview, bool canFollowUp
});




}
/// @nodoc
class __$AgentSessionCopyWithImpl<$Res>
    implements _$AgentSessionCopyWith<$Res> {
  __$AgentSessionCopyWithImpl(this._self, this._then);

  final _AgentSession _self;
  final $Res Function(_AgentSession) _then;

/// Create a copy of AgentSession
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? key = null,Object? title = null,Object? phase = null,Object? agent = null,Object? detail = null,Object? dot = null,Object? unread = null,Object? lastActivityMs = null,Object? notesRead = null,Object? changes = null,Object? canStop = null,Object? canReview = null,Object? canFollowUp = null,}) {
  return _then(_AgentSession(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as int,key: null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as AgentTaskPhase,agent: null == agent ? _self.agent : agent // ignore: cast_nullable_to_non_nullable
as String,detail: null == detail ? _self.detail : detail // ignore: cast_nullable_to_non_nullable
as String,dot: null == dot ? _self.dot : dot // ignore: cast_nullable_to_non_nullable
as AgentSessionDot,unread: null == unread ? _self.unread : unread // ignore: cast_nullable_to_non_nullable
as bool,lastActivityMs: null == lastActivityMs ? _self.lastActivityMs : lastActivityMs // ignore: cast_nullable_to_non_nullable
as int,notesRead: null == notesRead ? _self.notesRead : notesRead // ignore: cast_nullable_to_non_nullable
as int,changes: null == changes ? _self.changes : changes // ignore: cast_nullable_to_non_nullable
as int,canStop: null == canStop ? _self.canStop : canStop // ignore: cast_nullable_to_non_nullable
as bool,canReview: null == canReview ? _self.canReview : canReview // ignore: cast_nullable_to_non_nullable
as bool,canFollowUp: null == canFollowUp ? _self.canFollowUp : canFollowUp // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc
mixin _$AgentTranscriptItem {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTranscriptItem);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentTranscriptItem()';
}


}

/// @nodoc
class $AgentTranscriptItemCopyWith<$Res>  {
$AgentTranscriptItemCopyWith(AgentTranscriptItem _, $Res Function(AgentTranscriptItem) __);
}


/// Adds pattern-matching-related methods to [AgentTranscriptItem].
extension AgentTranscriptItemPatterns on AgentTranscriptItem {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AgentUserItem value)?  user,TResult Function( AgentSaidItem value)?  agent,TResult Function( AgentThoughtItem value)?  thought,TResult Function( AgentToolItem value)?  tool,TResult Function( AgentPermissionItem value)?  permission,TResult Function( AgentProposedItem value)?  proposed,TResult Function( AgentNoticeItem value)?  notice,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AgentUserItem() when user != null:
return user(_that);case AgentSaidItem() when agent != null:
return agent(_that);case AgentThoughtItem() when thought != null:
return thought(_that);case AgentToolItem() when tool != null:
return tool(_that);case AgentPermissionItem() when permission != null:
return permission(_that);case AgentProposedItem() when proposed != null:
return proposed(_that);case AgentNoticeItem() when notice != null:
return notice(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AgentUserItem value)  user,required TResult Function( AgentSaidItem value)  agent,required TResult Function( AgentThoughtItem value)  thought,required TResult Function( AgentToolItem value)  tool,required TResult Function( AgentPermissionItem value)  permission,required TResult Function( AgentProposedItem value)  proposed,required TResult Function( AgentNoticeItem value)  notice,}){
final _that = this;
switch (_that) {
case AgentUserItem():
return user(_that);case AgentSaidItem():
return agent(_that);case AgentThoughtItem():
return thought(_that);case AgentToolItem():
return tool(_that);case AgentPermissionItem():
return permission(_that);case AgentProposedItem():
return proposed(_that);case AgentNoticeItem():
return notice(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AgentUserItem value)?  user,TResult? Function( AgentSaidItem value)?  agent,TResult? Function( AgentThoughtItem value)?  thought,TResult? Function( AgentToolItem value)?  tool,TResult? Function( AgentPermissionItem value)?  permission,TResult? Function( AgentProposedItem value)?  proposed,TResult? Function( AgentNoticeItem value)?  notice,}){
final _that = this;
switch (_that) {
case AgentUserItem() when user != null:
return user(_that);case AgentSaidItem() when agent != null:
return agent(_that);case AgentThoughtItem() when thought != null:
return thought(_that);case AgentToolItem() when tool != null:
return tool(_that);case AgentPermissionItem() when permission != null:
return permission(_that);case AgentProposedItem() when proposed != null:
return proposed(_that);case AgentNoticeItem() when notice != null:
return notice(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String text)?  user,TResult Function( String text)?  agent,TResult Function( String text,  int seconds)?  thought,TResult Function( String title,  AgentToolKind kind,  AgentToolStatus status)?  tool,TResult Function( String title,  AgentPermissionAnswer answer)?  permission,TResult Function( int round,  String counts,  AgentRoundState state)?  proposed,TResult Function( String text)?  notice,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AgentUserItem() when user != null:
return user(_that.text);case AgentSaidItem() when agent != null:
return agent(_that.text);case AgentThoughtItem() when thought != null:
return thought(_that.text,_that.seconds);case AgentToolItem() when tool != null:
return tool(_that.title,_that.kind,_that.status);case AgentPermissionItem() when permission != null:
return permission(_that.title,_that.answer);case AgentProposedItem() when proposed != null:
return proposed(_that.round,_that.counts,_that.state);case AgentNoticeItem() when notice != null:
return notice(_that.text);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String text)  user,required TResult Function( String text)  agent,required TResult Function( String text,  int seconds)  thought,required TResult Function( String title,  AgentToolKind kind,  AgentToolStatus status)  tool,required TResult Function( String title,  AgentPermissionAnswer answer)  permission,required TResult Function( int round,  String counts,  AgentRoundState state)  proposed,required TResult Function( String text)  notice,}) {final _that = this;
switch (_that) {
case AgentUserItem():
return user(_that.text);case AgentSaidItem():
return agent(_that.text);case AgentThoughtItem():
return thought(_that.text,_that.seconds);case AgentToolItem():
return tool(_that.title,_that.kind,_that.status);case AgentPermissionItem():
return permission(_that.title,_that.answer);case AgentProposedItem():
return proposed(_that.round,_that.counts,_that.state);case AgentNoticeItem():
return notice(_that.text);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String text)?  user,TResult? Function( String text)?  agent,TResult? Function( String text,  int seconds)?  thought,TResult? Function( String title,  AgentToolKind kind,  AgentToolStatus status)?  tool,TResult? Function( String title,  AgentPermissionAnswer answer)?  permission,TResult? Function( int round,  String counts,  AgentRoundState state)?  proposed,TResult? Function( String text)?  notice,}) {final _that = this;
switch (_that) {
case AgentUserItem() when user != null:
return user(_that.text);case AgentSaidItem() when agent != null:
return agent(_that.text);case AgentThoughtItem() when thought != null:
return thought(_that.text,_that.seconds);case AgentToolItem() when tool != null:
return tool(_that.title,_that.kind,_that.status);case AgentPermissionItem() when permission != null:
return permission(_that.title,_that.answer);case AgentProposedItem() when proposed != null:
return proposed(_that.round,_that.counts,_that.state);case AgentNoticeItem() when notice != null:
return notice(_that.text);case _:
  return null;

}
}

}

/// @nodoc


class AgentUserItem implements AgentTranscriptItem {
  const AgentUserItem(this.text);
  

 final  String text;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentUserItemCopyWith<AgentUserItem> get copyWith => _$AgentUserItemCopyWithImpl<AgentUserItem>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentUserItem&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AgentTranscriptItem.user(text: $text)';
}


}

/// @nodoc
abstract mixin class $AgentUserItemCopyWith<$Res> implements $AgentTranscriptItemCopyWith<$Res> {
  factory $AgentUserItemCopyWith(AgentUserItem value, $Res Function(AgentUserItem) _then) = _$AgentUserItemCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AgentUserItemCopyWithImpl<$Res>
    implements $AgentUserItemCopyWith<$Res> {
  _$AgentUserItemCopyWithImpl(this._self, this._then);

  final AgentUserItem _self;
  final $Res Function(AgentUserItem) _then;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AgentUserItem(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AgentSaidItem implements AgentTranscriptItem {
  const AgentSaidItem(this.text);
  

 final  String text;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentSaidItemCopyWith<AgentSaidItem> get copyWith => _$AgentSaidItemCopyWithImpl<AgentSaidItem>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentSaidItem&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AgentTranscriptItem.agent(text: $text)';
}


}

/// @nodoc
abstract mixin class $AgentSaidItemCopyWith<$Res> implements $AgentTranscriptItemCopyWith<$Res> {
  factory $AgentSaidItemCopyWith(AgentSaidItem value, $Res Function(AgentSaidItem) _then) = _$AgentSaidItemCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AgentSaidItemCopyWithImpl<$Res>
    implements $AgentSaidItemCopyWith<$Res> {
  _$AgentSaidItemCopyWithImpl(this._self, this._then);

  final AgentSaidItem _self;
  final $Res Function(AgentSaidItem) _then;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AgentSaidItem(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AgentThoughtItem implements AgentTranscriptItem {
  const AgentThoughtItem({required this.text, this.seconds = 0});
  

 final  String text;
@JsonKey() final  int seconds;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentThoughtItemCopyWith<AgentThoughtItem> get copyWith => _$AgentThoughtItemCopyWithImpl<AgentThoughtItem>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentThoughtItem&&(identical(other.text, text) || other.text == text)&&(identical(other.seconds, seconds) || other.seconds == seconds));
}


@override
int get hashCode => Object.hash(runtimeType,text,seconds);

@override
String toString() {
  return 'AgentTranscriptItem.thought(text: $text, seconds: $seconds)';
}


}

/// @nodoc
abstract mixin class $AgentThoughtItemCopyWith<$Res> implements $AgentTranscriptItemCopyWith<$Res> {
  factory $AgentThoughtItemCopyWith(AgentThoughtItem value, $Res Function(AgentThoughtItem) _then) = _$AgentThoughtItemCopyWithImpl;
@useResult
$Res call({
 String text, int seconds
});




}
/// @nodoc
class _$AgentThoughtItemCopyWithImpl<$Res>
    implements $AgentThoughtItemCopyWith<$Res> {
  _$AgentThoughtItemCopyWithImpl(this._self, this._then);

  final AgentThoughtItem _self;
  final $Res Function(AgentThoughtItem) _then;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,Object? seconds = null,}) {
  return _then(AgentThoughtItem(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,seconds: null == seconds ? _self.seconds : seconds // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class AgentToolItem implements AgentTranscriptItem {
  const AgentToolItem({required this.title, required this.kind, required this.status});
  

 final  String title;
 final  AgentToolKind kind;
 final  AgentToolStatus status;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentToolItemCopyWith<AgentToolItem> get copyWith => _$AgentToolItemCopyWithImpl<AgentToolItem>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentToolItem&&(identical(other.title, title) || other.title == title)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.status, status) || other.status == status));
}


@override
int get hashCode => Object.hash(runtimeType,title,kind,status);

@override
String toString() {
  return 'AgentTranscriptItem.tool(title: $title, kind: $kind, status: $status)';
}


}

/// @nodoc
abstract mixin class $AgentToolItemCopyWith<$Res> implements $AgentTranscriptItemCopyWith<$Res> {
  factory $AgentToolItemCopyWith(AgentToolItem value, $Res Function(AgentToolItem) _then) = _$AgentToolItemCopyWithImpl;
@useResult
$Res call({
 String title, AgentToolKind kind, AgentToolStatus status
});




}
/// @nodoc
class _$AgentToolItemCopyWithImpl<$Res>
    implements $AgentToolItemCopyWith<$Res> {
  _$AgentToolItemCopyWithImpl(this._self, this._then);

  final AgentToolItem _self;
  final $Res Function(AgentToolItem) _then;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? title = null,Object? kind = null,Object? status = null,}) {
  return _then(AgentToolItem(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as AgentToolKind,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as AgentToolStatus,
  ));
}


}

/// @nodoc


class AgentPermissionItem implements AgentTranscriptItem {
  const AgentPermissionItem({required this.title, required this.answer});
  

 final  String title;
 final  AgentPermissionAnswer answer;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentPermissionItemCopyWith<AgentPermissionItem> get copyWith => _$AgentPermissionItemCopyWithImpl<AgentPermissionItem>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentPermissionItem&&(identical(other.title, title) || other.title == title)&&(identical(other.answer, answer) || other.answer == answer));
}


@override
int get hashCode => Object.hash(runtimeType,title,answer);

@override
String toString() {
  return 'AgentTranscriptItem.permission(title: $title, answer: $answer)';
}


}

/// @nodoc
abstract mixin class $AgentPermissionItemCopyWith<$Res> implements $AgentTranscriptItemCopyWith<$Res> {
  factory $AgentPermissionItemCopyWith(AgentPermissionItem value, $Res Function(AgentPermissionItem) _then) = _$AgentPermissionItemCopyWithImpl;
@useResult
$Res call({
 String title, AgentPermissionAnswer answer
});




}
/// @nodoc
class _$AgentPermissionItemCopyWithImpl<$Res>
    implements $AgentPermissionItemCopyWith<$Res> {
  _$AgentPermissionItemCopyWithImpl(this._self, this._then);

  final AgentPermissionItem _self;
  final $Res Function(AgentPermissionItem) _then;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? title = null,Object? answer = null,}) {
  return _then(AgentPermissionItem(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,answer: null == answer ? _self.answer : answer // ignore: cast_nullable_to_non_nullable
as AgentPermissionAnswer,
  ));
}


}

/// @nodoc


class AgentProposedItem implements AgentTranscriptItem {
  const AgentProposedItem({required this.round, required this.counts, required this.state});
  

 final  int round;
 final  String counts;
 final  AgentRoundState state;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentProposedItemCopyWith<AgentProposedItem> get copyWith => _$AgentProposedItemCopyWithImpl<AgentProposedItem>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentProposedItem&&(identical(other.round, round) || other.round == round)&&(identical(other.counts, counts) || other.counts == counts)&&(identical(other.state, state) || other.state == state));
}


@override
int get hashCode => Object.hash(runtimeType,round,counts,state);

@override
String toString() {
  return 'AgentTranscriptItem.proposed(round: $round, counts: $counts, state: $state)';
}


}

/// @nodoc
abstract mixin class $AgentProposedItemCopyWith<$Res> implements $AgentTranscriptItemCopyWith<$Res> {
  factory $AgentProposedItemCopyWith(AgentProposedItem value, $Res Function(AgentProposedItem) _then) = _$AgentProposedItemCopyWithImpl;
@useResult
$Res call({
 int round, String counts, AgentRoundState state
});




}
/// @nodoc
class _$AgentProposedItemCopyWithImpl<$Res>
    implements $AgentProposedItemCopyWith<$Res> {
  _$AgentProposedItemCopyWithImpl(this._self, this._then);

  final AgentProposedItem _self;
  final $Res Function(AgentProposedItem) _then;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? round = null,Object? counts = null,Object? state = null,}) {
  return _then(AgentProposedItem(
round: null == round ? _self.round : round // ignore: cast_nullable_to_non_nullable
as int,counts: null == counts ? _self.counts : counts // ignore: cast_nullable_to_non_nullable
as String,state: null == state ? _self.state : state // ignore: cast_nullable_to_non_nullable
as AgentRoundState,
  ));
}


}

/// @nodoc


class AgentNoticeItem implements AgentTranscriptItem {
  const AgentNoticeItem(this.text);
  

 final  String text;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentNoticeItemCopyWith<AgentNoticeItem> get copyWith => _$AgentNoticeItemCopyWithImpl<AgentNoticeItem>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentNoticeItem&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AgentTranscriptItem.notice(text: $text)';
}


}

/// @nodoc
abstract mixin class $AgentNoticeItemCopyWith<$Res> implements $AgentTranscriptItemCopyWith<$Res> {
  factory $AgentNoticeItemCopyWith(AgentNoticeItem value, $Res Function(AgentNoticeItem) _then) = _$AgentNoticeItemCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AgentNoticeItemCopyWithImpl<$Res>
    implements $AgentNoticeItemCopyWith<$Res> {
  _$AgentNoticeItemCopyWithImpl(this._self, this._then);

  final AgentNoticeItem _self;
  final $Res Function(AgentNoticeItem) _then;

/// Create a copy of AgentTranscriptItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AgentNoticeItem(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$AgentTranscript {

 AgentSession get session;/// Moves with every change.
 int get version; List<AgentTranscriptItem> get items;
/// Create a copy of AgentTranscript
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentTranscriptCopyWith<AgentTranscript> get copyWith => _$AgentTranscriptCopyWithImpl<AgentTranscript>(this as AgentTranscript, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTranscript&&(identical(other.session, session) || other.session == session)&&(identical(other.version, version) || other.version == version)&&const DeepCollectionEquality().equals(other.items, items));
}


@override
int get hashCode => Object.hash(runtimeType,session,version,const DeepCollectionEquality().hash(items));

@override
String toString() {
  return 'AgentTranscript(session: $session, version: $version, items: $items)';
}


}

/// @nodoc
abstract mixin class $AgentTranscriptCopyWith<$Res>  {
  factory $AgentTranscriptCopyWith(AgentTranscript value, $Res Function(AgentTranscript) _then) = _$AgentTranscriptCopyWithImpl;
@useResult
$Res call({
 AgentSession session, int version, List<AgentTranscriptItem> items
});


$AgentSessionCopyWith<$Res> get session;

}
/// @nodoc
class _$AgentTranscriptCopyWithImpl<$Res>
    implements $AgentTranscriptCopyWith<$Res> {
  _$AgentTranscriptCopyWithImpl(this._self, this._then);

  final AgentTranscript _self;
  final $Res Function(AgentTranscript) _then;

/// Create a copy of AgentTranscript
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? session = null,Object? version = null,Object? items = null,}) {
  return _then(_self.copyWith(
session: null == session ? _self.session : session // ignore: cast_nullable_to_non_nullable
as AgentSession,version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as int,items: null == items ? _self.items : items // ignore: cast_nullable_to_non_nullable
as List<AgentTranscriptItem>,
  ));
}
/// Create a copy of AgentTranscript
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AgentSessionCopyWith<$Res> get session {
  
  return $AgentSessionCopyWith<$Res>(_self.session, (value) {
    return _then(_self.copyWith(session: value));
  });
}
}


/// Adds pattern-matching-related methods to [AgentTranscript].
extension AgentTranscriptPatterns on AgentTranscript {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AgentTranscript value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AgentTranscript() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AgentTranscript value)  $default,){
final _that = this;
switch (_that) {
case _AgentTranscript():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AgentTranscript value)?  $default,){
final _that = this;
switch (_that) {
case _AgentTranscript() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( AgentSession session,  int version,  List<AgentTranscriptItem> items)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AgentTranscript() when $default != null:
return $default(_that.session,_that.version,_that.items);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( AgentSession session,  int version,  List<AgentTranscriptItem> items)  $default,) {final _that = this;
switch (_that) {
case _AgentTranscript():
return $default(_that.session,_that.version,_that.items);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( AgentSession session,  int version,  List<AgentTranscriptItem> items)?  $default,) {final _that = this;
switch (_that) {
case _AgentTranscript() when $default != null:
return $default(_that.session,_that.version,_that.items);case _:
  return null;

}
}

}

/// @nodoc


class _AgentTranscript implements AgentTranscript {
  const _AgentTranscript({required this.session, this.version = 0, final  List<AgentTranscriptItem> items = const <AgentTranscriptItem>[]}): _items = items;
  

@override final  AgentSession session;
/// Moves with every change.
@override@JsonKey() final  int version;
 final  List<AgentTranscriptItem> _items;
@override@JsonKey() List<AgentTranscriptItem> get items {
  if (_items is EqualUnmodifiableListView) return _items;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_items);
}


/// Create a copy of AgentTranscript
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AgentTranscriptCopyWith<_AgentTranscript> get copyWith => __$AgentTranscriptCopyWithImpl<_AgentTranscript>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AgentTranscript&&(identical(other.session, session) || other.session == session)&&(identical(other.version, version) || other.version == version)&&const DeepCollectionEquality().equals(other._items, _items));
}


@override
int get hashCode => Object.hash(runtimeType,session,version,const DeepCollectionEquality().hash(_items));

@override
String toString() {
  return 'AgentTranscript(session: $session, version: $version, items: $items)';
}


}

/// @nodoc
abstract mixin class _$AgentTranscriptCopyWith<$Res> implements $AgentTranscriptCopyWith<$Res> {
  factory _$AgentTranscriptCopyWith(_AgentTranscript value, $Res Function(_AgentTranscript) _then) = __$AgentTranscriptCopyWithImpl;
@override @useResult
$Res call({
 AgentSession session, int version, List<AgentTranscriptItem> items
});


@override $AgentSessionCopyWith<$Res> get session;

}
/// @nodoc
class __$AgentTranscriptCopyWithImpl<$Res>
    implements _$AgentTranscriptCopyWith<$Res> {
  __$AgentTranscriptCopyWithImpl(this._self, this._then);

  final _AgentTranscript _self;
  final $Res Function(_AgentTranscript) _then;

/// Create a copy of AgentTranscript
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? session = null,Object? version = null,Object? items = null,}) {
  return _then(_AgentTranscript(
session: null == session ? _self.session : session // ignore: cast_nullable_to_non_nullable
as AgentSession,version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as int,items: null == items ? _self._items : items // ignore: cast_nullable_to_non_nullable
as List<AgentTranscriptItem>,
  ));
}

/// Create a copy of AgentTranscript
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AgentSessionCopyWith<$Res> get session {
  
  return $AgentSessionCopyWith<$Res>(_self.session, (value) {
    return _then(_self.copyWith(session: value));
  });
}
}

// dart format on
