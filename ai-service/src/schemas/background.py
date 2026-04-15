from pydantic import BaseModel, Field, field_validator


class RemoveBackgroundRequest(BaseModel):
    image_base64: str = Field(min_length=8, max_length=20_000_000)
    output_format: str = Field(default="png")

    @field_validator("output_format")
    @classmethod
    def validate_output_format(cls, value: str) -> str:
        normalized = value.lower()
        if normalized != "png":
            raise ValueError(
                "Only png output is supported for this MVP endpoint"
            )
        return normalized


class RemoveBackgroundResponse(BaseModel):
    image_base64: str
    provider: str
    processing_ms: int
