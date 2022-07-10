import re
from gramadan import opers

class Opers(opers.Opers):
    @staticmethod
    def Demutate(text: str) -> str:
        demut = opers.Opers.Demutate(text)

        # remove t-prothesis
        demut = re.sub("^[tn]-", "", demut)

        return demut
