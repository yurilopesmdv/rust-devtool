from typing import Annotated

from fastapi import APIRouter, Depends

from app.modules.example.schemas import ExampleResponse
from app.modules.example.service import ExampleService, get_service

router = APIRouter(prefix="/api/v1/example", tags=["example"])
Service = Annotated[ExampleService, Depends(get_service)]


@router.get("", response_model=ExampleResponse)
async def get_example(service: Service) -> ExampleResponse:
    return await service.get()
