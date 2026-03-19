from __future__ import annotations

import contextvars
import json
import threading
import time
import uuid

from contextlib import contextmanager
from dataclasses import dataclass
from typing import Any, Callable

REGISTRY_REF_KEY = "__registry_ref__"
REGISTRY_META_KEY = "__registry_meta__"

_current_session_id: contextvars.ContextVar[str | None] = contextvars.ContextVar(
    "current_registry_session_id",
    default=None,
)

def _is_json_serializable(value: Any) -> bool:
    try:
        json.dumps(value)
        return True
    except (TypeError, OverflowError):
        return False
    
@dataclass
class RegistryEntry:
    value: Any
    expires_at: float
    created_at: float
    last_accessed_at: float

class InMemoryObjectRegistry:
    '''
    프로세스 내부에서만 유효한 object registry
    - thread-safe
    - session namespace 분리
    - TTL 기반 정리
    - background reaper thread 포함
    '''

    def __init__(
            self,
            default_ttl_seconds: int = 1800,
            cleanup_interval_seconds: int = 60,
    ) -> None:
        self.default_ttl_seconds = default_ttl_seconds
        self.cleanup_interval_seconds = cleanup_interval_seconds

        # {session_id: {object_id: RegistryEntry}}
        self._store: dict[str, dict[str, RegistryEntry]] = {}
        self._lock = threading.RLock()
        self._stop_event = threading.Event()

        self._reaper_thread = threading.Thread(
            target=self._reaper_loop,
            name="runtime-registry-reaper",
            daemon=True,
        )
        self._reaper_thread.start()
    
    def shutdown(self) -> None:
        self._stop_event.set()
        self._reaper_thread.join(timeout=1.0)

    def _now(self) -> float:
        return time.time()

    def _reaper_loop(self) -> None:
        while not self._stop_event.is_set():
            self._stop_event.wait(self.cleanup_interval_seconds)
            if self._stop_event.is_set():
                break
            self.cleanup_expired()
    
    def cleanup_expired(self) -> None:
        now = self._now()
        with self._lock:
            dead_sessions: list[str] = []
            for session_id, bucket in self._store.items():
                dead_keys = [
                    obj_id
                    for obj_id, entry in bucket.items()
                    if entry.expires_at <= now
                ]
                for obj_id in dead_keys:
                    del bucket[obj_id]
                
                if not bucket:
                    dead_sessions.append(session_id)
            
            for session_id in dead_sessions:
                del self._store[session_id]
        
    def start_session(self, session_id: str | None = None) -> str:
        sid = session_id or uuid.uuid4().hex
        with self._lock:
            self._store.setdefault(sid, {})
        return sid

    def clear_session(self, session_id: str) -> None:
        with self._lock:
            self._store.pop(session_id, None)
        
    def put(
            self,
            session_id: str,
            value: Any,
            ttl_seconds: int | None = None,
    ) -> str:
        ttl = ttl_seconds or self.default_ttl_seconds
        now = self._now()
        object_id = uuid.uuid4().hex

        with self._lock:
            bucket = self._store.setdefault(session_id, {})
            bucket[object_id] = RegistryEntry(
                value=value,
                created_at=now,
                last_accessed_at=now,
                expires_at=now + ttl,
            )
        
        return object_id

    def get(
            self,
            session_id: str,
            object_id: str,
            touch: bool = True,
    ) -> Any:
        now = self._now()

        with self._lock:
            bucket = self._store.get(session_id)
            if bucket is None:
                raise KeyError(f"registry session not found: {session_id}")

            entry = bucket.get(object_id)
            if entry is None:
                raise KeyError(
                    f"registry object not found: session_id={session_id}, object-id={object_id}"
                )

            if entry.expires_at <= now:
                del bucket[object_id]
                if not bucket:
                    self._store.pop(session_id, None)
                raise KeyError(
                    f"registry object expired: session_id={session_id}, object_id={object_id}"
                )

            if touch:
                entry.last_accessed_at = now
                entry.expires_at = now + self.default_ttl_seconds
            
            return entry.value
    
    def delete(self, session_id: str, object_id: str) -> None:
        with self._lock:
            bucket = self._store.get(session_id)
            if bucket is None:
                return
            bucket.pop(object_id, None)
            if not bucket:
                self._store.pop(session_id, None)

REGISTRY = InMemoryObjectRegistry()

def get_current_session_id() -> str:
    session_id = _current_session_id.get()
    if not session_id:
        raise RuntimeError(
            "No registry session is bound to the current context."
            "Use 'registry_session()' arount the graph execution."
        )
    return session_id

@contextmanager
def registry_session(
    session_id: str | None = None,
    *,
    clear_on_exit: bool = True,
) -> str:
    '''
    한 요청/한 graph run 단위로 session을 바인딩.
    async 환경에서도 contextvars 기반으로 context-local하게 유지됨.
    '''

    sid = REGISTRY.start_session(session_id)
    token = _current_session_id.set(sid)

    try:
        yield sid
    finally:
        _current_session_id.reset(token)
        if clear_on_exit:
            REGISTRY.clear_session(sid)

def make_registry_ref(
        object_id: str,
        *,
        session_id: str | None = None,
) -> dict[str, Any]:
    sid = session_id or get_current_session_id()
    return {
        REGISTRY_REF_KEY: object_id,
        REGISTRY_META_KEY: {
            "session_id": sid,
        }
    }

def is_registry_ref(value: Any) -> bool:
    return (
        isinstance(value, dict)
        and REGISTRY_REF_KEY in value
        and REGISTRY_META_KEY in value
        and isinstance(value[REGISTRY_META_KEY], dict)
        and "session_id" in value[REGISTRY_META_KEY]
    )

def save_nonserializable(
        value: Any,
        *,
        ttl_seconds: int | None = None,
        session_id: str | None = None,
) -> dict[str, Any]:
    sid = session_id or get_current_session_id()
    object_id = REGISTRY.put(sid, value, ttl_seconds=ttl_seconds)
    return make_registry_ref(object_id, session_id=sid)

def load_registry_ref(ref: dict[str, Any]) -> Any:
    object_id = ref[REGISTRY_REF_KEY]
    session_id = ref[REGISTRY_META_KEY]['session_id']
    return REGISTRY.get(session_id, object_id)

def externalize_state(
        value: Any,
        *,
        ttl_seconds: int | None = None,
        session_id: str | None = None,
) -> Any:
    '''
    Python 객체를 Rust runtime에 보낼 수 있는 JSON-friendly 구조로 변환.
    - JSON serializable 값은 그대로 둠
    - dict/list/tuple은 재귀적으로 처리
    - JSON 직렬화가 안 되는 leaf는 registry ref로 치환
    '''

    if is_registry_ref(value):
        return value
    
    if value is None or isinstance(value, (str, int, float, bool)):
        return value
    
    if isinstance(value, dict):
        return {
            str(k): externalize_state(v, ttl_seconds=ttl_seconds, session_id=session_id)
            for k, v in value.items()
        }
    
    if isinstance(value, list):
        return [
            externalize_state(v, ttl_seconds=ttl_seconds, session_id=session_id)
            for v in value
        ]

    if isinstance(value, tuple):
        return [
            externalize_state(v, ttl_seconds=ttl_seconds, session_id=session_id)
            for v in value
        ]
    
    if _is_json_serializable(value):
        return value
    
    return save_nonserializable(
        value,
        ttl_seconds=ttl_seconds,
        session_id=session_id
    )

def materialize_state(value: Any) -> Any:
    '''
    registry ref를 실제 Python 객체로 복원
    '''

    if is_registry_ref(value):
        return load_registry_ref(value)
    
    if isinstance(value, dict):
        return {k: materialize_state(v) for k, v in value.items()}
    
    if isinstance(value, list):
        return [materialize_state(v) for v in value]
    
    return value

def dumps_state_for_runtime(
        state: dict[str, Any],
        *,
        ttl_seconds: int | None = None,
) -> str:
    return json.dumps(
        externalize_state(state, ttl_seconds=ttl_seconds),
        ensure_ascii=False,
    )

def loads_state_from_runtime(state_json: str) -> dict[str, Any]:
    raw = json.loads(state_json)
    if not isinstance(raw, dict):
        raise TypeError("state_json must decode to dict")
    return materialize_state(raw)

def registry_managed_node(
        func: Callable[[dict[str, Any]], dict[str, Any]],
        *,
        ttl_seconds: int | None = None,
) -> Callable[[str], str]:
    '''
    Rust runtime이 기대하는 node 시그니처:
        (state_json: str) -> str
    를 자동으로 맞춰 주는 decorator.
    내부적으로:
    - state_json 로드
    - registry ref 복원
    - 원래 Python node 호출
    - 반환값에서 비직렬화 객체 externalize
    - JSON 문자열로 변환
    '''

    def wrapper(state_json: str) -> str:
        state = loads_state_from_runtime(state_json)
        update = func(state)

        if not isinstance(update, dict):
            raise TypeError(
                f"Node '{func.__name__} must return dict, got {type(update).__name__}"
            )
        safe_update = externalize_state(update, ttl_seconds=ttl_seconds)
        return json.dumps(safe_update, ensure_ascii=False)
    
    wrapper.__name__ = func.__name__
    wrapper.__doc__ = func.__doc__
    return wrapper

