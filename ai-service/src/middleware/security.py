import time
from collections import defaultdict, deque
from threading import Lock

from fastapi import Depends, Header, HTTPException, Request, status

from core.config import Settings, get_settings


class InMemoryRateLimiter:
    def __init__(self) -> None:
        self._events: dict[str, deque[float]] = defaultdict(deque)
        self._lock = Lock()

    def allow(self, key: str, limit: int, window_seconds: int) -> bool:
        now = time.time()
        threshold = now - window_seconds

        with self._lock:
            events = self._events[key]
            while events and events[0] < threshold:
                events.popleft()

            if len(events) >= limit:
                return False

            events.append(now)
            return True


rate_limiter = InMemoryRateLimiter()


def require_service_secret(
    settings: Settings = Depends(get_settings),
    x_ai_service_secret: str | None = Header(
        default=None,
        alias="x-ai-service-secret",
    ),
) -> None:
    if (
        not x_ai_service_secret
        or x_ai_service_secret != settings.ai_shared_secret
    ):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid service token",
        )


def enforce_rate_limit(
    request: Request,
    settings: Settings = Depends(get_settings),
) -> None:
    client_host = request.client.host if request.client else "unknown"
    limiter_key = f"{client_host}:{request.url.path}"

    is_allowed = rate_limiter.allow(
        key=limiter_key,
        limit=settings.ai_max_requests_per_minute,
        window_seconds=60,
    )

    if not is_allowed:
        raise HTTPException(
            status_code=status.HTTP_429_TOO_MANY_REQUESTS,
            detail="Rate limit exceeded",
        )
