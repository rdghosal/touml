import abc
import typing as t

from pydantic import BaseModel, ConfigDict


class Base(BaseModel):
    model_config = ConfigDict(
        populate_by_name=True, strict=False, arbitrary_types_allowed=True
    )

    id: int


class BaseMixin(abc.ABC):
    @abc.abstractmethod
    def do_something(self) -> t.Any: ...


class IntMixin(BaseModel, BaseMixin):
    def do_something(self) -> int:
        if 2 + 2 == 4:
            return 0
        return 1


class StrMixin(BaseModel, BaseMixin):
    def do_something(self) -> str:
        return str(0) if 2 + 2 == 4 else str(1)
