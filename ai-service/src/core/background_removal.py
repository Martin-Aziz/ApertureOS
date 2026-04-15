import asyncio
from dataclasses import dataclass

try:
    from rembg import remove as rembg_remove
except BaseException:  # pragma: no cover
    rembg_remove = None


class BackgroundRemovalError(RuntimeError):
    pass


@dataclass(frozen=True)
class ModelStatus:
    ready: bool
    provider: str


class BackgroundRemovalService:
    def __init__(self) -> None:
        self._provider = "rembg" if rembg_remove is not None else "unavailable"

    def status(self) -> ModelStatus:
        return ModelStatus(
            ready=rembg_remove is not None,
            provider=self._provider,
        )

    async def remove_background(self, image_bytes: bytes) -> bytes:
        if not image_bytes:
            raise BackgroundRemovalError("Input image cannot be empty")

        if rembg_remove is None:
            raise BackgroundRemovalError(
                "REMBG is unavailable in this environment"
            )

        try:
            return await asyncio.to_thread(rembg_remove, image_bytes)
        except Exception as error:
            raise BackgroundRemovalError(
                "Background removal inference failed"
            ) from error
