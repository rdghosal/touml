__all__ = ("StrClass", "IntClass")

import typing as t

from pydantic import field_validator, Field

from .base import Base, StrMixin, IntMixin


def init_value_3(*arg, b: int = 2, **kwargs) -> dict[str, int]:
    return {
        "a": 1,
        "b": 2,
        "c": 3,
    }


def random_func(a, /, b=2, *, c) -> dict[str, int]:
    return {
        "a": 1,
        "b": 2,
        "c": 3,
    }


class StrClass(Base, StrMixin):
    value: str
    some_other_values: tuple[int, ...]
    yet_another_value: set[int]

    def get_concatenated(self) -> str:
        return f"{self.value}{self.some_other_value}{self.yet_another_value}"


class IntClass(Base, IntMixin):
    value_1: int
    value_2: list[int]
    value_3: dict[str, int] | None = Field(default_factory=init_value_3, description='hello')

    @field_validator("value_1")
    @classmethod
    def check_even(cls, v: t.Any) -> int:
        if not isinstance(v, int) or v % 2:
            raise ValueError("Only even values allowed!")
        return v
