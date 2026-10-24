from pydantic import BaseModel, Field


class ExampleResponse(BaseModel):
    message: str


class ExampleInput(BaseModel):
    message: str = Field(strict=True, min_length=1, max_length=200)
