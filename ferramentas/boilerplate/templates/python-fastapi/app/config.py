import os


def read_config(key: str, *, required: bool = True) -> str | None:
    value = os.environ.get(key)
    if required and not value:
        raise ValueError(f"{key} ausente")
    return value
