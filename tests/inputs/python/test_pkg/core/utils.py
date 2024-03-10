__all__ = ("do_something",)

from ..models import StrClass, IntClass


def do_something() -> None:
    s = StrClass(id=1, value="hello", some_other_value="world", yet_another_value="!")
    i = IntClass(id=1, value_1=1, value_2=[1, 2])

    print(s.get_concatenated())
    print(i.do_something())
