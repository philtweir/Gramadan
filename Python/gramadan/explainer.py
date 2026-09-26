from __future__ import annotations
from typing import Literal, Any
from dataclasses import dataclass

Action = "Mutation" | Literal["Demutation"]

@dataclass
class Explanation:
    action: Action
    explanation: str
    fm: str | None
    to: str
    nil: bool

    def __str__(self) -> str:
        return f"[{self.fm}] -- {self.action} --> {self.to} << {self.explanation}"

class ExplainStrV1(str):
    def __new__(self, value: str, because: str, *args, **kwargs) -> Any:
        return str.__new__(value, *args, **kwargs)

ExplainStr = ExplainStrV1
