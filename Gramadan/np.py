import xml.etree.ElementTree as ET
from typing import Optional, Union
from features import Gender, Mutation, FormSg, Form
from noun import Noun
from possessive import Possessive
from adjective import Adjective

# A class for a noun phrase:
class NP:
    disambig: str = ""

    def getNickname() -> str:
        ret: str = self.getLemma() + " NP"
        if self.disambig != "":
            ret += " " + self.disambig
        ret = ret.Replace(" ", "_")
        return ret

    def __init__(
        self,
        sgNom: Optional[list[FormSg]],
        sgGen: Optional[list[FormSg]],
        sgDat: Optional[list[FormSg]],
        sgNomArt: Optional[list[FormSg]],
        sgGenArt: Optional[list[FormSg]],
        sgDatArtN: Optional[list[FormSg]],
        sgDatArtS: Optional[list[FormSg]],
        plNom: Optional[list[Form]],
        plGen: Optional[list[Form]],
        plDat: Optional[list[Form]],
        plNomArt: Optional[list[Form]],
        plGenArt: Optional[list[Form]],
        plDatArt: Optional[list[Form]],
        isDefinite: bool = False,
        isPossessed: bool = False,
        isImmutable: bool = False,
        forceNominative: bool = False,
        disambig: str = "",
    ):
        # Noun phrase forms in the singular, without article:
        self.sgNom: list[FormSg] = []
        if sgNom is not None:
            self.sgNom = sgNom
        self.sgGen: list[FormSg] = []
        if sgGen is not None:
            self.sgGen = sgGen
        self.sgDat: list[FormSg] = []  # head noun left unmutated
        if sgDat is not None:
            self.sgDat = sgDat

        # Noun phrase forms in the singular, with article:
        self.sgNomArt: list[FormSg] = []
        if sgNomArt is not None:
            self.sgNomArt = sgNomArt
        self.sgGenArt: list[FormSg] = []
        if sgGenArt is not None:
            self.sgGenArt = sgGenArt
        # northern system, as if with article but the article is *not* included, head noun unmutated
        self.sgDatArtN: list[FormSg] = []
        if sgDatArtN is not None:
            self.sgDatArtN = sgDatArtN
        # southern system, as if with article but the article is *not* included, head noun unmutated
        self.sgDatArtS: list[FormSg] = []
        if sgDatArtS is not None:
            self.sgDatArtS = sgDatArtS

        # Noun phrase forms in the plural, without article:
        self.plNom: list[Form] = []
        if plNom is not None:
            self.plNom = plNom
        self.plGen: list[Form] = []
        if plGen is not None:
            self.plGen = plGen
        self.plDat: list[Form] = []  # head noun left unmutated
        if plDat is not None:
            self.plDat = plDat

        # Noun phrase forms in the plural, with article:
        self.plNomArt: list[Form] = []
        if plNomArt is not None:
            self.plNomArt = plNomArt
        self.plGenArt: list[Form] = []
        if plGenArt is not None:
            self.plGenArt = plGenArt
        self.plDatArt: list[Form] = []
        if (
            plDatArt is not None
        ):  # as if with article but the article is *not* included, head noun unmutated
            self.plDatArt = plDatArt

        # Whether self noun phrase is definite:
        self.isDefinite: bool = isDefinite  # If True, the articleless forms are definite and there are no articled forms.
        # If False, the articleless forms are indefinite and the articled forms are definite.
        # Whether self noun phrase is determined by a possessive pronoun:
        self.isPossessed: bool = isPossessed  # if True, only sgNom, sgDat, sgGen, plNom, plDat, plGen exist, the others are empty.

        # Whether self NP's head noun cannot be mutated by prepositions:
        self.isImmutable: bool = isImmutable  # Eg. "blitz", dative "leis an blitz mhór"

        # Should the unarticled nominative be used in place of the unarticled genitive?
        self.forceNominative: bool = forceNominative

        self.disambig = disambig

    # Returns the noun phrase's lemma:
    def getLemma() -> str:
        ret: str = ""
        if self.sgNom.Count != 0:
            ret = self.sgNom[0].value
        if ret == "" and self.sgNomArt.Count != 0:
            ret = self.sgNomArt[0].value
        if ret == "" and self.plNom.Count != 0:
            ret = self.plNom[0].value
        if ret == "" and self.plNomArt.Count != 0:
            ret = self.plNomArt[0].value
        return ret

    # Returns the noun phrase's gender:
    def getGender(self) -> Gender:
        ret: Gender = Gender.Masc
        if self.sgNom.Count != 0:
            ret = self.sgNom[0].gender
        elif self.sgNomArt.Count != 0:
            ret = self.sgNomArt[0].gender
        return ret

    def hasGender(self) -> bool:
        ret: bool = False
        if self.sgNom.Count != 0 or self.sgNomArt.Count != 0:
            ret = True
        return ret

    # Prints a user-friendly summary of the noun phrase's forms:
    def print(self) -> str:
        ret: str = ""
        ret += "sgNom: "
        f: Form
        for f in self.sgNom:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "sgGen: "
        f: Form
        for f in self.sgGen:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "plNom: "
        f: Form
        for f in self.plNom:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "plGen: "
        f: Form
        for f in self.plGen:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "sgNomArt: "
        f: Form
        for f in self.sgNomArt:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "sgGenArt: "
        f: Form
        for f in self.sgGenArt:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "plNomArt: "
        f: Form
        for f in self.plNomArt:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "plGenArt: "
        f: Form
        for f in self.plGenArt:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += Environment.NewLine
        ret += "sgDat: "
        f: Form
        for f in self.sgDat:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "sgDatArtN: "
        f: Form
        for f in self.sgDatArtN:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "sgDatArtS: "
        f: Form
        for f in self.sgDatArtS:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "plDat: "
        f: Form
        for f in self.plDat:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "plDatArt: "
        f: Form
        for f in self.plDatArt:
            ret += "[" + f.value + "] "
        ret += "\n"
        return ret

    # Creates a noun phrase from an explicit listing of all the basic forms:
    def create(
        gender: Gender, sgNom: str, sgGen: str, plNom: str, plGen: str, sgDatArtN: str
    ):
        # region singular-nominative
        # without article:
        sgNom = [FormSg(sgNom, gender)]
        # { # with article:
        mut: Mutation = Mutation.PrefT if gender == Gender.Masc else Mutation.Len3
        value: str = "an " + Opers.Mutate(mut, sgNom)
        sgNomArt = [FormSg(value, gender)]
        # }
        # endregion
        # region singular-genitive
        # { # without article:
        value: str = sgNom
        sgGen = [FormSg(value, gender)]
        # }
        # { # with article:
        mut: Mutation = Mutation.Len3 if gender == Gender.Masc else Mutation.PrefH
        article: str = "an" if gender == Gender.Masc else "na"
        value: str = article + " " + Opers.Mutate(mut, sgGen)
        sgGenArt = [FormSg(value, gender)]
        # }
        # endregion
        # region plural-nominative
        # { # without article:
        plNom = [Form(plNom)]
        # }
        # { # with article:
        value: str = "na " + Opers.Mutate(Mutation.PrefH, plNom)
        plNomArt = [Form(value)]
        # }
        # endregion
        # region plural-genitive
        # { # without article:
        plGen = [Form(plNom)]
        # }
        # { # with article:
        value: str = "na " + Opers.Mutate(Mutation.Ecl1, plGen)
        plGenArt = [Form(value)]
        # }
        # endregion
        # region singular-dative
        # { # without article:
        sgDat = [FormSg(sgNom, gender)]
        # }
        # { # with article:
        sgDatArtN = [FormSg(sgDatArtN, gender)]
        sgDatArtS = [FormSg(sgNom, gender)]

        mut: Mutation = Mutation.PrefT if gender == Gender.Masc else Mutation.Len3
        value: str = "an " + Opers.Mutate(mut, sgNom)
        sgNomArt = [FormSg(value, gender)]
        # }
        # endregion
        # region plural-dative
        # { # without article:
        plDat = [Form(plNom)]
        # }
        # { # with article:
        plDatArt = [Form(plNom)]
        # }
        # endregion
        return cls(
            sgNom=sgNom,
            sgGen=sgGen,
            sgDat=sgDat,
            sgNomArt=sgNomArt,
            sgGenArt=sgGenArt,
            sgDatArtN=sgDatArtN,
            sgDatArtS=sgDatArtS,
            plNom=plNom,
            plGen=plGen,
            plDat=plDat,
            plNomArt=plNomArt,
            plGenArt=plGenArt,
            plDatArt=plDatArt,
            isDefinite=isDefinite,
            isPossessed=isPossessed,
            isImmutable=isImmutable,
            forceNominative=forceNominative,
        )

    # Creates a noun phrase from a noun determined by a possessive pronoun:
    @classmethod
    def create_from_possessive(cls, head: Noun, poss: Possessive) -> "NP":
        np = cls.create_from_noun(head)
        np._makePossessive(poss)
        return np

    # Creates a noun phrase from a noun modified by an adjective determined by a possessive pronoun:
    @classmethod
    def create_from_noun_adjective_possessive(
        cls, head: Noun, mod: Adjective, poss: Possessive
    ) -> "NP":
        np = cls.create_from_noun_adjective(noun, mod)
        np._makePossessive(poss)
        return np

    # Creates a noun phrase from a noun:
    @classmethod
    def create_from_noun(head: Noun) -> "NP":
        isDefinite = head.isDefinite
        isImmutable = head.isImmutable
        # region singular-nominative
        headForm: FormSg
        for headForm in head.sgNom:
            # without article:
            sgNom.append(FormSg(headForm.value, headForm.gender))

        if not head.isDefinite:  # with article:
            mut: Mutation = (
                Mutation.PrefT if headForm.gender == Gender.Masc else Mutation.Len3
            )
            if head.isImmutable:
                mut = Mutation.Nil
            value: str = "an " + Opers.Mutate(mut, headForm.value)
            sgNomArt.append(FormSg(value, headForm.gender))

        # endregion
        # region singular-genitive
        headForm: FormSg
        for headForm in head.sgGen:
            # without article:
            mut: Mutation = Mutation.Len1 if head.isProper else Mutation.Nil
            # proper nouns are always lenited in the genitive
            if head.isImmutable:
                mut = Mutation.Nil
            value: str = Opers.Mutate(mut, headForm.value)
            sgGen.append(FormSg(value, headForm.gender))

        # with article:
        if not head.isDefinite or head.allowArticledGenitive:
            mut: Mutation = (
                Mutation.Len3 if headForm.gender == Gender.Masc else Mutation.PrefH
            )
            if head.isImmutable:
                mut = Mutation.Nil
            article: str = "an" if headForm.gender == Gender.Masc else "na"
            value: str = article + " " + Opers.Mutate(mut, headForm.value)
            sgGenArt.append(FormSg(value, headForm.gender))

        # endregion
        # region plural-nominative
        headForm: Form
        for headForm in head.plNom:
            # without article:
            plNom.append(Form(headForm.value))

        if not head.isDefinite:  # with article:
            mut: Mutation = Mutation.PrefH
            if head.isImmutable:
                mut = Mutation.Nil
            value: str = "na " + Opers.Mutate(mut, headForm.value)
            plNomArt.append(Form(value))

        # endregion
        # region plural-genitive
        headForm: Form
        for headForm in head.plGen:
            # without article:
            mut: Mutation = Mutation.Len1 if head.isProper else Mutation.Nil
            # proper nouns are always lenited in the articleless genitive
            if head.isImmutable:
                mut = Mutation.Nil
            value: str = Opers.Mutate(mut, headForm.value)
            plGen.append(Form(value))

        if not head.isDefinite or head.allowArticledGenitive:  # with article:
            mut: Mutation = Mutation.Ecl1
            if head.isImmutable:
                mut = Mutation.Nil
            value: str = "na " + Opers.Mutate(mut, headForm.value)
            plGenArt.append(Form(value))

        # endregion
        # region singular-dative
        headForm: FormSg
        for headForm in head.sgDat:
            # without article:
            sgDat.append(FormSg(headForm.value, headForm.gender))

        if not head.isDefinite:  # with article:
            sgDatArtN.append(FormSg(headForm.value, headForm.gender))
            sgDatArtS.append(FormSg(headForm.value, headForm.gender))

        # endregion
        # region plural-dative
        headForm: Form
        for headForm in head.plNom:
            # without article:
            plDat.append(Form(headForm.value))
        if not head.isDefinite:  # with article:
            plDatArt.append(Form(headForm.value))
        # endregion
        return cls(
            sgNom=sgNom,
            sgGen=sgGen,
            sgDat=sgDat,
            sgNomArt=sgNomArt,
            sgGenArt=sgGenArt,
            sgDatArtN=sgDatArtN,
            sgDatArtS=sgDatArtS,
            plNom=plNom,
            plGen=plGen,
            plDat=plDat,
            plNomArt=plNomArt,
            plGenArt=plGenArt,
            plDatArt=plDatArt,
            isDefinite=isDefinite,
            isPossessed=isPossessed,
            isImmutable=isImmutable,
            forceNominative=forceNominative,
        )

    # Creates a noun phrase from a noun modified by an adjective:
    @classmethod
    def create_from_noun_adjective(cls, head: Noun, mod: Adjective):
        if mod.isPre:
            prefixedHead: Noun = Noun.create_from_xml(head.printXml())
            # create a copy of the head noun
            prefix: str = mod.getLemma()
            f: Form
            for f in prefixedHead.sgNom:
                f.value = Opers.Prefix(prefix, f.value)
            f: Form
            for f in prefixedHead.sgGen:
                f.value = Opers.Prefix(prefix, f.value)
            f: Form
            for f in prefixedHead.sgDat:
                f.value = Opers.Prefix(prefix, f.value)
            f: Form
            for f in prefixedHead.sgVoc:
                f.value = Opers.Prefix(prefix, f.value)
            f: Form
            for f in prefixedHead.plNom:
                f.value = Opers.Prefix(prefix, f.value)
            f: Form
            for f in prefixedHead.plGen:
                f.value = Opers.Prefix(prefix, f.value)
            f: Form
            for f in prefixedHead.plVoc:
                f.value = Opers.Prefix(prefix, f.value)
            f: Form
            for f in prefixedHead.count:
                f.value = Opers.Prefix(prefix, f.value)
            np = cls.create_from_noun(prefixedHead)
            isDefinite = np.isDefinite
            sgNom = np.sgNom
            sgNomArt = np.sgNomArt
            sgGen = np.sgGen
            sgGenArt = np.sgGenArt
            sgDat = np.sgDat
            sgDatArtN = np.sgDatArtN
            sgDatArtS = np.sgDatArtS
            plNom = np.plNom
            plNomArt = np.plNomArt
            plGen = np.plGen
            plGenArt = np.plGenArt
            plDat = np.plDat
            plDatArt = np.plDatArt
        else:
            isDefinite = head.isDefinite
            isImmutable = head.isImmutable
            forceNominative = True
            sgNom = []
            sgNomArt = []
            sgGen = []
            sgGenArt = []
            sgDat = []
            sgDatArtN = []
            sgDatArtS = []
            plNom = []
            plNomArt = []
            plGen = []
            plGenArt = []
            plDat = []
            plDatArt = []
            # region singular-nominative
            headForm: FormSg
            for headForm in head.sgNom:
                # without article:
                modForm: Form
                for modForm in mod.sgNom:
                    mutA: Mutation = (
                        Mutation.Nil
                        if headForm.gender == Gender.Masc
                        else Mutation.Len1
                    )
                    value: str = (
                        headForm.value + " " + Opers.Mutate(mutA, modForm.value)
                    )
                    sgNom.append(FormSg(value, headForm.gender))

                if not head.isDefinite:  # with article:
                    modForm: Form
                    for modForm in mod.sgNom:
                        mutN: Mutation = (
                            Mutation.PrefT
                            if headForm.gender == Gender.Masc
                            else Mutation.Len3
                        )
                        if head.isImmutable:
                            mutN = Mutation.Nil
                        mutA: Mutation = (
                            Mutation.Nil
                            if headForm.gender == Gender.Masc
                            else Mutation.Len1
                        )
                        value: str = (
                            "an "
                            + Opers.Mutate(mutN, headForm.value)
                            + " "
                            + Opers.Mutate(mutA, modForm.value)
                        )
                        sgNomArt.append(FormSg(value, headForm.gender))

            # endregion
            # region singular-genitive
            headForm: FormSg
            for headForm in head.sgGen:
                # without article:
                modForms: list[Form] = (
                    mod.sgGenMasc if headForm.gender == Gender.Masc else mod.sgGenFem
                )
                modForm: Form
                for modForm in modForms:
                    mutN: Mutation = Mutation.Len1 if head.isProper else Mutation.Nil
                    # proper nouns are always lenited in the genitive
                    if head.isImmutable:
                        mutN = Mutation.Nil
                    mutA: Mutation = (
                        Mutation.Len1
                        if headForm.gender == Gender.Masc
                        else Mutation.Nil
                    )
                    value: str = (
                        Opers.Mutate(mutN, headForm.value)
                        + " "
                        + Opers.Mutate(mutA, modForm.value)
                    )
                    sgGen.append(FormSg(value, headForm.gender))

            headForm: FormSg
            for headForm in head.sgGen:
                # with article:
                if not head.isDefinite or head.allowArticledGenitive:
                    modForms: list[Form] = (
                        mod.sgGenMasc
                        if headForm.gender == Gender.Masc
                        else mod.sgGenFem
                    )
                    modForm: Form
                    for modForm in modForms:
                        mutN: Mutation = (
                            Mutation.Len3
                            if headForm.gender == Gender.Masc
                            else Mutation.PrefH
                        )
                        if head.isImmutable:
                            mutN = Mutation.Nil
                        mutA: Mutation = (
                            Mutation.Len1
                            if headForm.gender == Gender.Masc
                            else Mutation.Nil
                        )
                        article: str = "an" if headForm.gender == Gender.Masc else "na"
                        value: str = (
                            article
                            + " "
                            + Opers.Mutate(mutN, headForm.value)
                            + " "
                            + Opers.Mutate(mutA, modForm.value)
                        )
                        sgGenArt.append(FormSg(value, headForm.gender))

            # endregion
            # region plural-nominative
            headForm: Form
            for headForm in head.plNom:
                # without article:
                modForm: Form
                for modForm in mod.plNom:
                    mutA: Mutation = (
                        Mutation.Len1
                        if Opers.IsSlender(headForm.value)
                        else Mutation.Nil
                    )
                    value: str = (
                        headForm.value + " " + Opers.Mutate(mutA, modForm.value)
                    )
                    plNom.append(Form(value))

                if not head.isDefinite:  # with article:
                    modForm: Form
                    for modForm in mod.plNom:
                        mutN: Mutation = Mutation.PrefH
                        if head.isImmutable:
                            mutN = Mutation.Nil
                        mutA: Mutation = (
                            Mutation.Len1
                            if Opers.IsSlender(headForm.value)
                            else Mutation.Nil
                        )
                        value: str = (
                            "na "
                            + Opers.Mutate(mutN, headForm.value)
                            + " "
                            + Opers.Mutate(mutA, modForm.value)
                        )
                        plNomArt.append(Form(value))

            # endregion
            # region plural-genitive
            headForm: FormPlGen
            for headForm in head.plGen:
                # without article:
                modForms: list[Form] = (
                    mod.plNom if headForm.strength == Strength.Strong else mod.sgNom
                )
                modForm: Form
                for modForm in modForms:
                    mutA: Mutation = (
                        Mutation.Len1
                        if Opers.IsSlender(headForm.value)
                        else Mutation.Nil
                    )
                    if headForm.strength == Strength.Weak:
                        mutA = (
                            Mutation.Len1
                            if Opers.IsSlenderI(headForm.value)
                            else Mutation.Nil
                        )
                        # "Gael", "captaen" are not slender
                    value: str = (
                        headForm.value + " " + Opers.Mutate(mutA, modForm.value)
                    )
                    plGen.append(Form(value))

            headForm: FormPlGen
            for headForm in head.plGen:
                # with article:
                if not head.isDefinite or head.allowArticledGenitive:
                    modForms: list[Form] = (
                        mod.plNom if headForm.strength == Strength.Strong else mod.sgNom
                    )
                    modForm: Form
                    for modForm in modForms:
                        mutN: Mutation = Mutation.Ecl1
                        if head.isImmutable:
                            mutN = Mutation.Nil
                        mutA: Mutation = (
                            Mutation.Len1
                            if Opers.IsSlender(headForm.value)
                            else Mutation.Nil
                        )
                        if headForm.strength == Strength.Weak:
                            mutA = (
                                Mutation.Len1
                                if Opers.IsSlenderI(headForm.value)
                                else Mutation.Nil
                            )
                            # "Gael", "captaen" are not slender
                        value: str = (
                            "na "
                            + Opers.Mutate(mutN, headForm.value)
                            + " "
                            + Opers.Mutate(mutA, modForm.value)
                        )
                        plGenArt.append(Form(value))

            # endregion
            # region singular-dative
            headForm: FormSg
            for headForm in head.sgDat:
                # without article:
                modForm: Form
                for modForm in mod.sgNom:
                    mutA: Mutation = (
                        Mutation.Nil
                        if headForm.gender == Gender.Masc
                        else Mutation.Len1
                    )
                    value: str = (
                        headForm.value + " " + Opers.Mutate(mutA, modForm.value)
                    )
                    sgDat.append(FormSg(value, headForm.gender))

                if not head.isDefinite:  # with article:
                    modForm: Form
                    for modForm in mod.sgNom:
                        mutA: Mutation = (
                            Mutation.Nil
                            if headForm.gender == Gender.Masc
                            else Mutation.Len1
                        )
                        value: str = (
                            headForm.value + " " + Opers.Mutate(mutA, modForm.value)
                        )
                        sgDatArtS.append(FormSg(value, headForm.gender))

                    modForm: Form
                    for modForm in mod.sgNom:
                        value: str = (
                            headForm.value
                            + " "
                            + Opers.Mutate(Mutation.Len1, modForm.value)
                        )
                        sgDatArtN.append(FormSg(value, headForm.gender))

            # endregion
            # region plural-dative
            headForm: Form
            for headForm in head.plNom:
                # without article:
                modForm: Form
                for modForm in mod.plNom:
                    mutA: Mutation = (
                        Mutation.Len1
                        if Opers.IsSlender(headForm.value)
                        else Mutation.Nil
                    )
                    value: str = (
                        headForm.value + " " + Opers.Mutate(mutA, modForm.value)
                    )
                    plDat.append(Form(value))

                if not head.isDefinite:  # with article:
                    modForm: Form
                    for modForm in mod.plNom:
                        mutA: Mutation = (
                            Mutation.Len1
                            if Opers.IsSlender(headForm.value)
                            else Mutation.Nil
                        )
                        value: str = (
                            headForm.value + " " + Opers.Mutate(mutA, modForm.value)
                        )
                        plDatArt.append(Form(value))

                # endregion
        return cls(
            sgNom=sgNom,
            sgGen=sgGen,
            sgDat=sgDat,
            sgNomArt=sgNomArt,
            sgGenArt=sgGenArt,
            sgDatArtN=sgDatArtN,
            sgDatArtS=sgDatArtS,
            plNom=plNom,
            plGen=plGen,
            plDat=plDat,
            plNomArt=plNomArt,
            plGenArt=plGenArt,
            plDatArt=plDatArt,
            isDefinite=isDefinite,
            isPossessed=isPossessed,
            isImmutable=isImmutable,
            forceNominative=forceNominative,
        )

        # Constructor helper: Adds a possessive pronoun to sgNom, sgDat, sgGen, plNom, plDat, plGen of itself, empties all other forms:
        def _makePossessive(self, poss: Possessive):
            self.isDefinite = True
            self.isPossessed = True
            # region singular-nominative
            headForm: FormSg
            for headForm in self.sgNom:
                # & vs && ?
                if poss.apos.Count > 0 and (
                    Opers.StartsVowel(headForm.value)
                    or Opers.StartsFVowel(headForm.value)
                ):
                    possForm: Form
                    for possForm in poss.apos:
                        value: str = possForm.value + Opers.Mutate(
                            poss.mutation, headForm.value
                        )
                        headForm.value = value

                else:
                    possForm: Form
                    for possForm in poss.full:
                        value: str = (
                            possForm.value
                            + " "
                            + Opers.Mutate(poss.mutation, headForm.value)
                        )
                        headForm.value = value

            # endregion
            # region singular-dative
            headForm: FormSg
            for headForm in self.sgDat:
                if poss.apos.Count > 0 and (
                    Opers.StartsVowel(headForm.value)
                    or Opers.StartsFVowel(headForm.value)
                ):
                    possForm: Form
                    for possForm in poss.apos:
                        value: str = possForm.value + Opers.Mutate(
                            poss.mutation, headForm.value
                        )
                        headForm.value = value

                else:
                    possForm: Form
                    for possForm in poss.full:
                        value: str = (
                            possForm.value
                            + " "
                            + Opers.Mutate(poss.mutation, headForm.value)
                        )
                        headForm.value = value

            # endregion
            # region singular-genitive
            headForm: FormSg
            for headForm in self.sgGen:
                if poss.apos.Count > 0 and (
                    Opers.StartsVowel(headForm.value)
                    or Opers.StartsFVowel(headForm.value)
                ):
                    possForm: Form
                    for possForm in poss.apos:
                        value: str = possForm.value + Opers.Mutate(
                            poss.mutation, headForm.value
                        )
                        headForm.value = value

                else:
                    possForm: Form
                    for possForm in poss.full:
                        value: str = (
                            possForm.value
                            + " "
                            + Opers.Mutate(poss.mutation, headForm.value)
                        )
                        headForm.value = value

            # endregion
            # region plural-nominative
            headForm: Form
            for headForm in self.plNom:
                if poss.apos.Count > 0 and (
                    Opers.StartsVowel(headForm.value)
                    or Opers.StartsFVowel(headForm.value)
                ):
                    possForm: Form
                    for possForm in poss.apos:
                        value: str = possForm.value + Opers.Mutate(
                            poss.mutation, headForm.value
                        )
                        headForm.value = value

                else:
                    possForm: Form
                    for possForm in poss.full:
                        value: str = (
                            possForm.value
                            + " "
                            + Opers.Mutate(poss.mutation, headForm.value)
                        )
                        headForm.value = value

            # endregion
            # region plural-dative
            headForm: Form
            for headForm in self.plDat:
                if poss.apos.Count > 0 and (
                    Opers.StartsVowel(headForm.value)
                    or Opers.StartsFVowel(headForm.value)
                ):
                    possForm: Form
                    for possForm in poss.apos:
                        value: str = possForm.value + Opers.Mutate(
                            poss.mutation, headForm.value
                        )
                        headForm.value = value

                else:
                    possForm: Form
                    for possForm in poss.full:
                        value: str = (
                            possForm.value
                            + " "
                            + Opers.Mutate(poss.mutation, headForm.value)
                        )
                        headForm.value = value

            # endregion
            # region plural-genitive
            headForm: Form
            for headForm in self.plGen:
                if poss.apos.Count > 0 & (
                    Opers.StartsVowel(headForm.value)
                    or Opers.StartsFVowel(headForm.value)
                ):
                    possForm: Form
                    for possForm in poss.apos:
                        value: str = possForm.value + Opers.Mutate(
                            poss.mutation, headForm.value
                        )
                        headForm.value = value

                else:
                    possForm: Form
                    for possForm in poss.full:
                        value: str = (
                            possForm.value
                            + " "
                            + Opers.Mutate(poss.mutation, headForm.value)
                        )
                        headForm.value = value

            # endregion
            # region empty-all-others
            self.sgDatArtN = []
            self.sgDatArtS = []
            self.sgGenArt = []
            self.sgNomArt = []
            self.plDatArt = []
            self.plGenArt = []
            self.plNomArt = []
            # endregion

    # Prints the noun phrase in BuNaMo format:
    def printXml() -> ET.ElementTree:
        root: ET.Element = ET.Element("nounPhrase")
        doc: ET.ElementTree = ET.ElementTree(root)
        root.set("default", self.getLemma())
        root.set("disambig", self.disambig)
        root.set("isImmutable", ("1" if self.isImmutable else "0"))
        root.set("isDefinite", ("1" if self.isDefinite else "0"))
        root.set("isPossessed", ("1" if self.isPossessed else "0"))
        root.set("forceNominative", ("1" if self.forceNominative else "0"))
        f: FormSg
        for f in self.sgNom:
            el = ET.SubElement(root, "sgNom")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        f: FormSg
        for f in self.sgGen:
            el = ET.SubElement(root, "sgGen")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        f: FormSg
        for f in self.sgNomArt:
            el = ET.SubElement(root, "sgNomArt")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        f: FormSg
        for f in self.sgGenArt:
            el = ET.SubElement(root, "sgGenArt")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        f: Form
        for f in self.plNom:
            el = ET.SubElement(root, "plNom")
            el.set("default", f.value)

        f: Form
        for f in self.plGen:
            el = ET.SubElement(root, "plGen")
            el.set("default", f.value)

        f: Form
        for f in self.plNomArt:
            el = ET.SubElement(root, "plNomArt")
            el.set("default", f.value)

        f: Form
        for f in self.plGenArt:
            el = ET.SubElement(root, "plGenArt")
            el.set("default", f.value)

        f: FormSg
        for f in self.sgDat:
            el = ET.SubElement(root, "sgDat")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        f: FormSg
        for f in self.sgDatArtN:
            el = ET.SubElement(root, "sgDatArtN")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        f: FormSg
        for f in self.sgDatArtS:
            el = ET.SubElement(root, "sgDatArtS")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        f: Form
        for f in self.plDat:
            el = ET.SubElement(root, "plDat")
            el.set("default", f.value)

        f: Form
        for f in self.plDatArt:
            el = ET.SubElement(root, "plDatArt")
            el.set("default", f.value)

        return doc

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET.ElementTree]):
        if isinstance(doc, str):
            xml = ET.parse(doc)
            return cls.create_from_xml(xml)

        root = doc.getroot()
        disambig = root.get("disambig")
        isDefinite = True if doc.DocumentElement.get("isDefinite") == "1" else False
        isPossessed = True if doc.DocumentElement.get("isPossessed") == "1" else False
        isImmutable = True if doc.DocumentElement.get("isImmutable") == "1" else False
        forceNominative = (
            True if doc.DocumentElement.get("forceNominative") == "1" else False
        )
        sgNom = []
        sgNomArt = []
        sgGen = []
        sgGenArt = []
        sgDat = []
        sgDatArtN = []
        sgDatArtS = []
        plNom = []
        plNomArt = []
        plGen = []
        plGenArt = []
        plDat = []
        plDatArt = []
        el: ET.Element
        for el in doc.SelectNodes("/*/sgNom"):
            sgNom.append(
                FormSg(
                    el.get("default"),
                    (Gender.Fem if el.get("gender") == "fem" else Gender.Masc),
                )
            )

        el: ET.Element
        for el in doc.SelectNodes("/*/sgGen"):
            sgGen.append(
                FormSg(
                    el.get("default"),
                    (Gender.Fem if el.get("gender") == "fem" else Gender.Masc),
                )
            )

        el: ET.Element
        for el in doc.SelectNodes("/*/sgNomArt"):
            sgNomArt.append(
                FormSg(
                    el.get("default"),
                    (Gender.Fem if el.get("gender") == "fem" else Gender.Masc),
                )
            )

        el: ET.Element
        for el in doc.SelectNodes("/*/sgGenArt"):
            sgGenArt.append(
                FormSg(
                    el.get("default"),
                    (Gender.Fem if el.get("gender") == "fem" else Gender.Masc),
                )
            )

        el: ET.Element
        for el in doc.SelectNodes("/*/plNom"):
            plNom.append(Form(el.get("default")))

        el: ET.Element
        for el in doc.SelectNodes("/*/plGen"):
            plGen.append(Form(el.get("default")))

        el: ET.Element
        for el in doc.SelectNodes("/*/plNomArt"):
            plNomArt.append(Form(el.get("default")))

        el: ET.Element
        for el in doc.SelectNodes("/*/plGenArt"):
            plGenArt.append(Form(el.get("default")))

        el: ET.Element
        for el in doc.SelectNodes("/*/sgDat"):
            sgDat.append(
                FormSg(
                    el.get("default"),
                    (Gender.Fem if el.get("gender") == "fem" else Gender.Masc),
                )
            )

        el: ET.Element
        for el in doc.SelectNodes("/*/sgDatArtN"):
            sgDatArtN.append(
                FormSg(
                    el.get("default"),
                    (Gender.Fem if el.get("gender") == "fem" else Gender.Masc),
                )
            )

        el: ET.Element
        for el in doc.SelectNodes("/*/sgDatArtS"):
            sgDatArtS.append(
                FormSg(
                    el.get("default"),
                    (Gender.Fem if el.get("gender") == "fem" else Gender.Masc),
                )
            )

        el: ET.Element
        for el in doc.SelectNodes("/*/plDat"):
            plDat.append(Form(el.get("default")))

        el: ET.Element
        for el in doc.SelectNodes("/*/plDatArt"):
            plDatArt.append(Form(el.get("default")))

        return cls(
            sgNom=sgNom,
            sgGen=sgGen,
            sgDat=sgDat,
            sgNomArt=sgNomArt,
            sgGenArt=sgGenArt,
            sgDatArtN=sgDatArtN,
            sgDatArtS=sgDatArtS,
            plNom=plNom,
            plGen=plGen,
            plDat=plDat,
            plNomArt=plNomArt,
            plGenArt=plGenArt,
            plDatArt=plDatArt,
            isDefinite=isDefinite,
            isPossessed=isPossessed,
            isImmutable=isImmutable,
            forceNominative=forceNominative,
            disambig=disambig,
        )
