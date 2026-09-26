from dataclasses import dataclass, asdict
from typing import Generator, Optional, Any
from collections import UserList
from contextvars import ContextVar
from contextlib import contextmanager

from gramadan import explainer
from gramadan.explainer import Action, Explanation

@dataclass
class Source:
    fm: "Form"

@dataclass
class Add:
    fm: str
    explanation: str

class ExplainStr(str):
    explanation: list[str | Explanation | Source | Add] | None = None
    _str: str

    def __init__(self, string: str, because: str | None=None, fm: str | None=None):
        str.__init__('--')
        self._str = str(string)
        if explaining():
            if fm is not None:
                self.explanation = list(fm.explanation or [])
            if because is not None:
                self.explanation = self.explanation or []
                self.explanation.append(
                    Add(string, because)
                )
            elif isinstance(string, ExplainStr):
                self.explanation = list(string.explanation or [])

    def __new__(cls, *args, **kw):
        # We use str to pass isinstance checks, but otherwise do not interact.
        return str.__new__(cls, args[0])

    def __iter__(self) -> Generator[str, None, None]:
        yield from self._str

    def __getitem__(self, idx: int | slice) -> str:
        if isinstance(idx, int):
            return self._str[idx]
        # If we have a single character, it is unlikely a
        # merge of explanations is useful or desirable. If a slice, then
        # we it is reasonably possible to be building something out of it
        return ExplainStr(self._str[idx], fm=self)

    def startswith(self, string: str) -> bool:
        return self._str.startswith(str(string))

    def endswith(self, string: str) -> bool:
        return self._str.endswith(str(string))

    def __str__(self) -> str:
        return self._str

    def lower(self) -> "ExplainStr":
        return ExplainStr(self._str.lower(), fm=self)

    def because(self, explanation: Explanation | str) -> "ExplainStr":
        if explaining():
            self.explanation = self.explanation or []
            self.explanation.append(explanation)
        return self

    def __add__(self, right: str) -> "ExplainStr":
        new = ExplainStr(str(self) + str(right), fm=self)
        if explaining() and isinstance(right, ExplainStr) and right.explanation:
            new.explanation = new.explanation or []
            new.explanation += right.explanation
        return new

    def __iadd__(self, right: str) -> str:
        return self + right

    def __eq__(self, other: object) -> bool:
        return self._str == str(other)

    def __hash__(self) -> int:
        return hash(self._str)

    def __ladd__(self, left: str) -> "ExplainStr":
        new = ExplainStr(str(left) + str(self), fm=self)
        return new

    def extend(self, new_text: str) -> "ExplainStr":
        if isinstance(new_text, ExplainStr):
            raise RuntimeError("Extending an explanation requires a new piece of text, not a merging explanation")
        return ExplainStr(new_text, fm=self)


class Explainer(UserList[Explanation]):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.forms = []

    def add_form(self, form: "Form") -> None:
        self.forms.append(form)

    def as_lines(self, include_nil: bool=False) -> str:
        lines = []
        for form in self.forms:
            if form.value.explanation is None:
                continue
            for step in form.value.explanation:
                if include_nil or not step.nil:
                    lines.append(form.value + " | " + str(step))
        return lines

@contextmanager
def open_explanation_context() -> Generator[Explainer, None, None]:
    # This holds the (reasonable) assumption that any operation that
    # wants to record an explanation this way operates in a single
    # Python context, or at least manages the sharing of data.
    tok = EXPLANATIONS.set(Explainer())
    try:
        yield EXPLANATIONS.get()
    finally:
        EXPLANATIONS.reset(tok)

def explaining() -> bool:
    try:
        EXPLANATIONS.get()
    except LookupError:
        return False
    return True

def finalize_form(form: "Form") -> Explainer | None:
    try:
        explanation = EXPLANATIONS.get()
        form.value.explanation = list(explanation)
        explanation.add_form(form)
        explanation.clear()
        return explanation
    except LookupError:
        return None

def explain(form: Any, include_nil: bool=False, indent: str="") -> list[str]:
    if isinstance(form, UserList): # FormList
        form = form[0]
    if not form.explanation:
        raise RuntimeError("Can only print an explanation if the steps were recorded during building")

    lines = []
    for explanation in form.explanation:
        if isinstance(explanation, Add):
            note = f"Add [{explanation.fm}]"
            if explanation.explanation:
                note += f": {explanation.explanation}"
        elif isinstance(explanation, Source):
            if (fl := explanation.fm._form_list) and (parent := fl.parent):
                source = fl.name or parent.find_form_list(explanation.fm._form_list)
                source_name = f"{parent.__class__.__name__} [{parent.getLemma()}] {source}"
                if explanation.fm.explanation:
                    lines.append(f"To get {explanation.fm} from {source_name}")
                    lines += explain(explanation.fm, include_nil=include_nil, indent=indent + "  ")
                    note = f"  => {explanation.fm}"
                else:
                    note = f"Took {explanation.fm} from {source_name}"
                properties = asdict(explanation.fm)
                properties.pop("value", None)
                if properties:
                    note += f" ({' '.join((k + '=' + str(v)) for k, v in properties.items())})"
            else:
                note = "Error: missing source"
        elif isinstance(explanation, Explanation):
            if not include_nil and explanation.nil:
                continue
            note = f"Changed [{explanation.fm}] to [{explanation.to}] ({explanation.action})"
            if explanation.explanation:
                note += f": {explanation.explanation}"
        else:
            note = str(explanation)
        lines.append(note)
    return [(indent + line) for line in lines]
EXPLANATIONS: ContextVar[Explainer] = ContextVar("explanations")
