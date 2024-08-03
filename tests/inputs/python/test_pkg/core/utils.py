__all__ = ("do_something",)

import enum
from typing import ClassVar, Any

from ..models import StrClass, IntClass


class ClassicClass:
    A_CLS_VAR = 0
    ANOTHER_CLS_VAR = [  # FIXME: Generics are not working here...
        {
            "a_key": 1,
            "a_nested_key": {
                1: 2,
                3: 4,
            },
        }
    ]

    class Status(enum.Enum):
        STATUS_1 = 1
        STATUS_2 = 2

    def __init__(self, val) -> None:
        self.value = val


def do_something() -> None:
    s = StrClass(id=1, value="hello", some_other_value="world", yet_another_value="!")
    i = IntClass(id=1, value_1=1, value_2=[1, 2])

    print(s.get_concatenated())
    print(i.do_something())

    d: dict[str, typing.Any] = {1: {'2', '3'}, (1, 2): '3'}
