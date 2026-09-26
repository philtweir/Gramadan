import re
from gramadan import opers
from gramadan.features import Mutation
from gramadan.explainer import Explanation
from .explainer import ExplainStr


class Opers(opers.Opers):
    @staticmethod
    def Mutate(mutation: Mutation, text: str | ExplainStr, explanation: str | None = None) -> ExplainStr:
        new_text = opers.Opers.Mutate(mutation, str(text))
        if isinstance(text, ExplainStr):
            new_text = text.extend(new_text)
        else:
            new_text = ExplainStr(new_text)
        if explanation:
            new_text.because(Explanation(
                mutation, explanation, text, new_text, mutation == Mutation.Nil or text == new_text
            ))
        return new_text

    @staticmethod
    def Demutate(text: str, explanation: str | None = None) -> ExplainStr:
        demut = opers.Opers.Demutate(text)

        # remove t-prothesis
        demut = re.sub("^[tn]-", "", demut)

        if isinstance(text, ExplainStr):
            demut = text.extend(demut)
        else:
            demut = ExplainStr(demut)
        if explanation:
            demut.because(Explanation("Demutation", explanation, text, demut, text == demut))

        return demut

    @staticmethod
    def IsSlenderEnding(text: str) -> bool:
        # Note that IsSlender is _specifically_
        # a slender _consonant_ ending, and will
        # return False for a slender vowel ending.
        # This returns True either way.
        return text[-1].lower() in opers.Opers.VowelsSlender or opers.Opers.IsSlender(text)
