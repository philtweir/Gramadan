import re
from gramadan import opers

class Opers(opers.Opers):
    @staticmethod
    def Demutate(text: str) -> str:
        demut = opers.Opers.Demutate(text)

        # remove t-prothesis
        demut = re.sub("^[tn]-", "", demut)

        return demut

    @staticmethod
    def IsSlenderEnding(text: str) -> bool:
        # Note that IsSlender is _specifically_
        # a slender _consonant_ ending, and will
        # return False for a slender vowel ending.
        # This returns True either way.
        return text[-1].lower() in opers.Opers.VowelsSlender or opers.Opers.IsSlender(text)
