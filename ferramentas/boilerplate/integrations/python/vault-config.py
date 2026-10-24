import os
from pathlib import Path


def read_config(key: str, *, required: bool = True) -> str | None:
    direct = os.environ.get(key)
    path = os.environ.get(f"{key}_FILE")
    if direct is not None and path is not None:
        raise ValueError(f"{key}: configure valor OU arquivo, não ambos")
    value = Path(path).read_text().strip() if path else direct
    if required and not value:
        raise ValueError(f"{key} ausente")
    return value
