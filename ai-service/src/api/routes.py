import base64
import binascii
import logging
from time import perf_counter

from fastapi import APIRouter, Depends, HTTPException, status
from fastapi.responses import PlainTextResponse

from core.background_removal import (
    BackgroundRemovalError,
    BackgroundRemovalService,
)
from core.config import Settings, get_settings
from middleware.security import enforce_rate_limit, require_service_secret
from schemas.background import (
    RemoveBackgroundRequest,
    RemoveBackgroundResponse,
)

logger = logging.getLogger(__name__)
router = APIRouter()
background_service = BackgroundRemovalService()


@router.get("/health/live")
async def live() -> dict[str, str]:
    return {"status": "live", "service": "pixelforge-ai-service"}


@router.get("/health/ready")
async def ready() -> dict[str, str]:
    status_info = background_service.status()
    if not status_info.ready:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="Background removal model is unavailable",
        )

    return {"status": "ready", "provider": status_info.provider}


@router.get("/metrics", response_class=PlainTextResponse)
async def metrics() -> str:
    status_info = background_service.status()
    model_ready = 1 if status_info.ready else 0

    return "\n".join(
        [
            "# HELP pixelforge_ai_service_info Static AI service metadata.",
            "# TYPE pixelforge_ai_service_info gauge",
            'pixelforge_ai_service_info{service="pixelforge-ai-service"} 1',
            "# HELP pixelforge_ai_model_ready Whether the model is ready.",
            "# TYPE pixelforge_ai_model_ready gauge",
            (
                "pixelforge_ai_model_ready"
                f'{{provider="{status_info.provider}"}} {model_ready}'
            ),
            "",
        ]
    )


@router.post(
    "/v1/remove-background",
    response_model=RemoveBackgroundResponse,
    dependencies=[
        Depends(require_service_secret),
        Depends(enforce_rate_limit),
    ],
)
async def remove_background(
    payload: RemoveBackgroundRequest,
    settings: Settings = Depends(get_settings),
) -> RemoveBackgroundResponse:
    started_at = perf_counter()

    try:
        image_bytes = base64.b64decode(payload.image_base64, validate=True)
    except binascii.Error as error:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="image_base64 must be a valid base64 payload",
        ) from error

    if len(image_bytes) > settings.ai_max_image_bytes:
        raise HTTPException(
            status_code=status.HTTP_413_REQUEST_ENTITY_TOO_LARGE,
            detail=(
                "Decoded image exceeds maximum size of "
                f"{settings.ai_max_image_bytes} bytes"
            ),
        )

    try:
        output_image = await background_service.remove_background(image_bytes)
    except BackgroundRemovalError as error:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=str(error),
        ) from error

    encoded_output = base64.b64encode(output_image).decode("ascii")
    processing_ms = int((perf_counter() - started_at) * 1000)

    logger.info(
        "background-removal-finished",
        extra={
            "processing_ms": processing_ms,
            "provider": background_service.status().provider,
        },
    )

    return RemoveBackgroundResponse(
        image_base64=encoded_output,
        provider=background_service.status().provider,
        processing_ms=processing_ms,
    )
