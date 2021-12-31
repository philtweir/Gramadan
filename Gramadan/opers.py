from features import Mutation


class Opers:
    @staticmethod
    def Demutate(text: str) -> str:
        pattern: str
        pattern = "^[bB][hH]([fF].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        pattern = "^([bcdfgmpstBCDFGMPST])[hH](.*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1$2")
        pattern = "^[mM]([bB].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        pattern = "^[gG]([cC].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        pattern = "^[nN]([dD].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        pattern = "^[nN]([gG].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        pattern = "^[bB]([pP].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        pattern = "^[tT]([sS].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        pattern = "^[dD]([tT].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        pattern = "^[dD]'([fF])[hH](.*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1$2")
        pattern = "^[dD]'([aeiouaáéíóúAEIOUÁÉÍÓÚ].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        pattern = "^[hH]([aeiouaáéíóúAEIOUÁÉÍÓÚ].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        pattern = "^[nN]-([aeiouaáéíóúAEIOUÁÉÍÓÚ].*)$"
        if Regex.IsMatch(text, pattern):
            text = Regex.Replace(text, pattern, "$1")
        return text

    # Performs a mutation on the string:
    @staticmethod
    def Mutate(mutation: Mutation, text: str) -> str:
        ret: str = ""
        pattern: str

        if mutation == Mutation.Len1 or mutation == Mutation.Len1D:
            # lenition 1
            if ret == "":
                pattern = "^([pbmftdcgPBMFTDCG])[jJ]"
                if Regex.IsMatch(text, pattern):
                    ret = text
                    # do not mutate exotic words with J in second position, like Djibouti
            if ret == "":
                pattern = "^([pbmftdcgPBMFTDCG])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "$1h$2")
            if ret == "":
                pattern = "^([sS])([rnlRNLaeiouáéíóúAEIOUÁÉÍÓÚ].*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "$1h$2")
            if ret == "":
                ret = text
            if mutation == Mutation.Len1D:
                pattern = "^([aeiouáéíóúAEIOUÁÉÍÓÚfF])(.*)$"
                if Regex.IsMatch(ret, pattern):
                    ret = Regex.Replace(ret, pattern, "d'$1$2")
        elif mutation == Mutation.Len2 or mutation == Mutation.Len2D:
            # lenition 2: same as lenition 1 but leaves "d", "t" and "s" unmutated
            if ret == "":
                pattern = "^([pbmftdcgPBMFTDCG])[jJ]"
                if Regex.IsMatch(text, pattern):
                    ret = text
                    # do not mutate exotic words with J in second position, like Djibouti
            if ret == "":
                pattern = "^([pbmfcgPBMFCG])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "$1h$2")
            if ret == "":
                ret = text
            if mutation == Mutation.Len2D:
                pattern = "^([aeiouáéíóúAEIOUÁÉÍÓÚfF])(.*)$"
                if Regex.IsMatch(ret, pattern):
                    ret = Regex.Replace(ret, pattern, "d'$1$2")
        elif mutation == Mutation.Len3 or mutation == Mutation.Len3D:
            # lenition 3: same as lenition 2 but also changes "s" into "ts"
            if ret == "":
                pattern = "^([pbmftdcgPBMFTDCG])[jJ]"
                if Regex.IsMatch(text, pattern):
                    ret = text
                    # do not mutate exotic words with J in second position, like Djibouti
            if ret == "":
                pattern = "^([pbmfcgPBMFCG])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "$1h$2")
            if ret == "":
                pattern = "^([sS])([rnlRNLaeiouáéíóúAEIOUÁÉÍÓÚ].*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "t$1$2")
            if ret == "":
                ret = text
            if mutation == Mutation.Len3D:
                pattern = "^([aeiouáéíóúAEIOUÁÉÍÓÚfF])(.*)$"
                if Regex.IsMatch(ret, pattern):
                    ret = Regex.Replace(ret, pattern, "d'$1$2")
        elif mutation == Mutation.Ecl1:
            # eclisis 1
            if ret == "":
                pattern = "^([pP])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "b$1$2")
            if ret == "":
                pattern = "^([bB])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "m$1$2")
            if ret == "":
                pattern = "^([fF])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "bh$1$2")
            if ret == "":
                pattern = "^([cC])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "g$1$2")
            if ret == "":
                pattern = "^([gG])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "n$1$2")
            if ret == "":
                pattern = "^([tT])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "d$1$2")
            if ret == "":
                pattern = "^([dD])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "n$1$2")
            if ret == "":
                pattern = "^([aeiuoáéíúó])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "n-$1$2")
            if ret == "":
                pattern = "^([AEIUOÁÉÍÚÓ])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "n$1$2")
        elif mutation == Mutation.Ecl1x:
            # eclisis 1x: same as eclipsis 1 but leaves vowels unchanged
            if ret == "":
                pattern = "^([pP])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "b$1$2")
            if ret == "":
                pattern = "^([bB])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "m$1$2")
            if ret == "":
                pattern = "^([fF])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "bh$1$2")
            if ret == "":
                pattern = "^([cC])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "g$1$2")
            if ret == "":
                pattern = "^([gG])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "n$1$2")
            if ret == "":
                pattern = "^([tT])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "d$1$2")
            if ret == "":
                pattern = "^([dD])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "n$1$2")
        elif mutation == Mutation.Ecl2:
            # eclipsis 2: same as eclipsis 1 but leaves "t", "d" and vowels unchanged
            if ret == "":
                pattern = "^([pP])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "b$1$2")
            if ret == "":
                pattern = "^([bB])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "m$1$2")
            if ret == "":
                pattern = "^([fF])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "bh$1$2")
            if ret == "":
                pattern = "^([cC])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "g$1$2")
            if ret == "":
                pattern = "^([gG])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "n$1$2")
        elif mutation == Mutation.Ecl3:
            # eclipsis 3: same as eclipsis 2 but also changes "s" to "ts"
            if ret == "":
                pattern = "^([pP])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "b$1$2")
            if ret == "":
                pattern = "^([bB])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "m$1$2")
            if ret == "":
                pattern = "^([fF])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "bh$1$2")
            if ret == "":
                pattern = "^([cC])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "g$1$2")
            if ret == "":
                pattern = "^([gG])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "n$1$2")
            if ret == "":
                pattern = "^([sS])([rnlRNLaeiouáéíóúAEIOUÁÉÍÓÚ].*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "t$1$2")
        elif mutation == Mutation.PrefT:
            # t-prefixation
            if ret == "":
                pattern = "^([aeiuoáéíúó])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "t-$1$2")
            if ret == "":
                pattern = "^([AEIUOÁÉÍÚÓ])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "t$1$2")
        elif mutation == Mutation.PrefH:
            # h-prefixation
            if ret == "":
                pattern = "^([aeiuoáéíúóAEIUOÁÉÍÚÓ])(.*)$"
                if Regex.IsMatch(text, pattern):
                    ret = Regex.Replace(text, pattern, "h$1$2")

        if ret == "":
            ret = text
        return ret

    # Tells you whether the string ends in a "dentals" cosonant:
    @staticmethod
    def EndsDental(txt: str) -> bool:
        return Regex.IsMatch(txt, "[dntsDNTS]$")

    # Tells you whether the string ends in a slender consonant cluster:
    @staticmethod
    def IsSlender(txt: str) -> bool:
        return Regex.IsMatch(txt, "[eiéí][^aeiouáéíóú]+$")

    # Tells you whether the string ends in a slender consonant cluster where the slenderness is caused by an "i" (and not by an "e"):
    @staticmethod
    def IsSlenderI(txt: str) -> bool:
        return Regex.IsMatch(txt, "[ií][^aeiouáéíóú]+$")

    # Tells you whether the string has a vowel or 'fh' (but not 'fhl' or 'fhr') at its start:
    @staticmethod
    def StartsVowelFhx(txt: str) -> bool:
        ret: boot = False
        if Regex.IsMatch(txt, "^[aeiouáéíóúAEIOUÁÉÍÓÚ]"):
            ret = true
        if Regex.IsMatch(txt, "^fh[^lr]", re.I):
            ret = true
        return ret

    # Tells you whether the string ends in a vowel:
    @staticmethod
    def EndsVowel(txt: str) -> bool:
        ret: boot = False
        if Regex.IsMatch(txt, "[aeiouáéíóúAEIOUÁÉÍÓÚ]$"):
            ret = true
        return ret

    # Tells you whether the string starts in a vowel:
    @staticmethod
    def StartsVowel(txt: str) -> bool:
        ret: boot = False
        if Regex.IsMatch(txt, "^[aeiouáéíóúAEIOUÁÉÍÓÚ]"):
            ret = true
        return ret

    # Tells you whether the string starts in F followed by a vowel:
    @staticmethod
    def StartsFVowel(txt: str) -> bool:
        ret: boot = False
        if Regex.IsMatch(txt, "^[fF][aeiouáéíóúAEIOUÁÉÍÓÚ]"):
            ret = true
        return ret

    # Tells you whether the string starts in b, m, p:
    @staticmethod
    def StartsBilabial(txt: str) -> bool:
        ret: boot = False
        if Regex.IsMatch(txt, "^[bmpBMP]"):
            ret = true
        return ret

    # Character types, for convenience when writing regular expressions:
    Cosonants: str = "bcdfghjklmnpqrstvwxz"
    Vowels: str = "aeiouáéíóú"
    VowelsBroad: str = "aouáóú"
    VowelsSlender: str = "eiéí"

    # Performs regular slenderization (attenuation): if the base ends in a consonant, and if the vowel cluster immediately before this consonant
    # ends in a broad vowel, then it changes this vowel cluster such that it ends in a slender vowel now.
    # Note: a base that's already slender passes through unchanged.
    @staticmethod
    def Slenderize(bayse: str) -> str:
        ret: str = bayse

        sources: tuple[str] = ("ea", "éa", "ia", "ío", "io", "iu", "ae")
        targets: tuple[str] = ("i", "éi", "éi", "í", "i", "i", "aei")
        match: Match
        for source, target in zip(sources, targets):
            match = Regex.Match(
                bayse,
                "^(.*["
                + Opers.Cosonants
                + "])?"
                + source
                + "(["
                + Opers.Cosonants
                + "]+)$",
            )
            if match.Success:
                ret = match.Groups[1].Value + target + match.Groups[2].Value
                return ret

        # The generic case: insert "i" at the end of the vowel cluster:
        match = Regex.Match(
            bayse, "^(.*[" + Opers.VowelsBroad + "])([" + Opers.Cosonants + "]+)$"
        )
        if match.Success:
            ret = match.Groups[1].Value + "i" + match.Groups[2].Value

        return ret

    # Performs irregular slenderization (attenuation): if the base ends in a consonant, and if the vowel cluster immediately before this consonant
    # ends in a broad vowel, then it changes this vowel cluster into the target (the second argument).
    # Note: if the target does not end in a slender vowel, then regular slenderization is attempted instead.
    # Note: a base that's already attenuated passes through unchanged.
    @staticmethod
    def Slenderize(bayse: str, target: str) -> str:
        ret: str = bayse
        if not Regex.IsMatch(target, "[" + Opers.VowelsSlender + "]$"):
            ret = Opers.Slenderize(bayse)
            # attempt regular slenderization instead
        else:
            match: Match = Regex.Match(
                bayse,
                "^(.*?)["
                + Opers.Vowels
                + "]*["
                + Opers.VowelsBroad
                + "](["
                + Opers.Cosonants
                + "]+)$",
            )
            if match.Success:
                ret = match.Groups[1].Value + target + match.Groups[2].Value
        return ret

    # Performs regular broadening: if the base ends in a consonant, and if the vowel cluster immediately before this consonant
    # ends in a slender vowel, then it changes this vowel cluster such that it ends in a broad vowel now.
    # Note: a base that's already broad passes through unchanged.
    @staticmethod
    def Broaden(bayse: str) -> str:
        ret: str = bayse

        sources: tuple[str] = ("ói", "ei", "éi", "i", "aí", "í", "ui", "io")
        targets: tuple[str] = ("ó", "ea", "éa", "ea", "aío", "ío", "o", "ea")
        match: Match
        for source, target in zip(sources, targets):
            match = Regex.Match(
                bayse,
                "^(.*["
                + Opers.Cosonants
                + "])?"
                + source
                + "(["
                + Opers.Cosonants
                + "]+)$",
            )
            if match.Success:
                ret = match.Groups[1].Value + target + match.Groups[2].Value
                return ret

        # The generic case: remove "i" from the end of the vowel cluster:
        match = Regex.Match(bayse, "^(.*)i([" + Opers.Cosonants + "]+)$")
        if match.Success:
            ret = match.Groups[1].Value + match.Groups[2].Value

        return ret

    # Performs irregular broadening: if the base ends in a consonant, and if the vowel cluster immediately before this consonant
    # ends in a slender vowel, then it changes this vowel cluster into the target (the second argument).
    # Note: if the target does not end in a broad vowel, then regular broadening is attempted instead.
    # Note: a base that's already broad passes through unchanged.
    @staticmethod
    def Broaden(bayse: str, target: str) -> str:
        ret: str = bayse
        if not Regex.IsMatch(target, "[" + Opers.VowelsBroad + "]$"):
            ret = Opers.Broaden(bayse)
            # attempt regular broadening instead
        else:
            match: Match = Regex.Match(
                bayse,
                "^(.*?)["
                + Opers.Vowels
                + "]*["
                + Opers.VowelsSlender
                + "](["
                + Opers.Cosonants
                + "]+)$",
            )
            if match.Success:
                ret = match.Groups[1].Value + target + match.Groups[2].Value
        return ret

    # If the final consonant cluster consists of two consonants that differ in voicing,
    # and if neither one of them is "l", "n" or "r", then devoices the second one.
    @staticmethod
    def Devoice(bayse: str) -> str:
        ret: str = bayse
        match: Match = Regex.Match(bayse, "^(.*)sd$")
        if match.Success:
            ret = match.Groups[1].Value + "st"
            return ret
        # May need elaboration.
        return ret

    # Reduces any duplicated consonants at the end into a single consonant.
    @staticmethod
    def Unduplicate(bayse: str) -> str:
        ret: str = bayse

        match: Match = Regex.Match(
            bayse, "^.*[" + Opers.Cosonants + "][" + Opers.Cosonants + "]$"
        )
        if match.Success and bayse[bayse.Length - 1] == bayse[bayse.Length - 2]:
            ret = bayse.Substring(0, bayse.Length - 1)

        return ret

    # Performs syncope by removing the final vowel cluster,
    # then unduplicates and devoices the consonant cluster at the end.
    @staticmethod
    def Syncope(bayse: str) -> str:
        ret: str = bayse

        match: Match = Regex.Match(
            bayse,
            "^(.*["
            + Opers.Cosonants
            + "])?["
            + Opers.Vowels
            + "]+(["
            + Opers.Cosonants
            + "]+)$",
        )
        if match.Success:
            ret = Opers.Devoice(
                Opers.Unduplicate(match.Groups[1].Value + match.Groups[2].Value)
            )

        return ret

    @staticmethod
    def HighlightMutations(text: str) -> str:
        return HighlightMutations(text, "")

    @staticmethod
    def HighlightMutations(text: str, bayse: str) -> str:
        text = Regex.Replace(
            text, "(^| )([cdfgmpst])(h)", "$1$2<u class='lenition'>$3</u>", re.I
        )
        text = Regex.Replace(
            text, "(^| )(b)(h)([^f])", "$1$2<u class='lenition'>$3</u>$4", re.I
        )
        text = Regex.Replace(
            text, "(^| )(t)(s)", "$1<u class='lenition'>$2</u>$3", re.I
        )
        text = Regex.Replace(
            text, "(^| )(m)(b)", "$1<u class='eclipsis'>$2</u>$3", re.I
        )
        text = Regex.Replace(
            text, "(^| )(g)(c)", "$1<u class='eclipsis'>$2</u>$3", re.I
        )
        text = Regex.Replace(
            text, "(^| )(n)(d)", "$1<u class='eclipsis'>$2</u>$3", re.I
        )
        text = Regex.Replace(
            text, "(^| )(bh)(f)", "$1<u class='eclipsis'>$2</u>$3", re.I
        )
        text = Regex.Replace(
            text, "(^| )(n)(g)", "$1<u class='eclipsis'>$2</u>$3", re.I
        )
        text = Regex.Replace(
            text, "(^| )(b)(p)", "$1<u class='eclipsis'>$2</u>$3", re.I
        )
        text = Regex.Replace(
            text, "(^| )(d)(t)", "$1<u class='eclipsis'>$2</u>$3", re.I
        )
        text = Regex.Replace(
            text, "(^| )(n-)([aeiouáéíóú])", "$1<u class='eclipsis'>$2</u>$3"
        )
        if not bayse.StartsWith("n"):
            text = Regex.Replace(
                text, "(^| )(n)([AEIOUÁÉÍÓÚ])", "$1<u class='eclipsis'>$2</u>$3"
            )
        if not bayse.StartsWith("t-"):
            text = Regex.Replace(
                text, "(^| )(t-)([aeiouáéíóú])", "$1<u class='lenition'>$2</u>$3"
            )
        if not bayse.StartsWith("t"):
            text = Regex.Replace(
                text, "(^| )(t)([AEIOUÁÉÍÓÚ])", "$1<u class='lenition'>$2</u>$3"
            )
        if not bayse.StartsWith("h"):
            text = Regex.Replace(
                text, "(^| )(h)([aeiouáéíóú])", "$1<u class='lenition'>$2</u>$3", re.I
            )
        return text

    @staticmethod
    def Prefix(prefix: str, body: str) -> str:
        m: Mutation = Mutation.Len1
        if Opers.EndsDental(prefix):
            m = Mutation.Len2
            # pick the right mutation
        if prefix.Substring(prefix.Length - 1) == body.Substring(0):
            prefix += "-"
            # eg. "sean-nós"
        if EndsVowel(prefix) and StartsVowel(body):
            prefix += "-"
            # eg. "ró-éasca"
        if (
            body.Substring(0, 1) == body.Substring(0, 1).ToUpper()
        ):  # eg. "seanÉireannach" > "Sean-Éireannach"
            prefix = prefix.Substring(0, 1).ToUpper() + prefix.Substring(1)
            if not prefix.EndsWith("-"):
                prefix += "-"

        ret: str = prefix + Mutate(m, body)
        return ret
