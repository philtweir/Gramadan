from typing import Optional
from features import FormSg, Form

# A class for a prepositional phrase:
class PP:
    def __init__(
        self,
        sg: Optional[list[FormSg]],
        sgArtN: Optional[list[FormSg]],
        pl: Optional[list[Form]],
        plArt: Optional[list[Form]],
        prepNick: str = "",
    ):
        # Forms:
        self.sg: list[FormSg] = []  # singular, no article
        if sg is not None:
            self.sg = sg

        self.sgArtN: list[FormSg] = []  # singular, with article, northern system
        if sgArtN is not None:
            self.sgArtN = sgArtN

        self.sgArtS: list[FormSg] = []  # singular, with article, southern system
        if sgArtS is not None:
            self.sgArtS = sgArtS

        self.pl: list[Form] = []  # plural, no article
        if pl is not None:
            self.pl = pl

        self.plArt: list[Form] = []  # plural, with article
        if plArt is not None:
            self.plArt = plArt

        # The nickname of the preposition from which this prepositional phrase was created:
        self.prepNick: str = prepNick

    # Returns the prepositional phrase's lemma:
    def getLemma() -> str:
        ret: str = ""
        if this.sg.Count != 0:
            ret = this.sg[0].value
        if ret == "" and this.sgArtS.Count != 0:
            ret = this.sgArtS[0].value
        if ret == "" and this.sgArtN.Count != 0:
            ret = this.sgArtN[0].value
        if ret == "" and this.pl.Count != 0:
            ret = this.pl[0].value
        if ret == "" and this.plArt.Count != 0:
            ret = this.plArt[0].value
        return ret

        def getNickname() -> str:
            ret: str = getLemma() + " PP"
            ret = ret.Replace(" ", "_")
            return ret

        # Returns the prepositional phrase's gender:
        def getGender() -> Gender:
            ret: Gender = Gender.Masc
            if this.sg.Count != 0:
                ret = this.sg[0].gender
            elif this.sgArtS.Count != 0:
                ret = this.sgArtS[0].gender
            elif this.sgArtN.Count != 0:
                ret = this.sgArtN[0].gender
            return ret

        def hasGender() -> bool:
            ret: bool = False

            if this.sg.Count != 0 or this.sgArtS.Count != 0 or this.sgArtN.Count != 0:
                ret = True
            return ret

        # Is the prepositional phrase invalid? This can happen if it has been constructed from an unsupported preposition:
        def isInvalid() -> bool:
            ret: bool = True
            if this.sg.Count != 0:
                ret = False
            if ret and this.sgArtS.Count != 0:
                ret = False
            if ret and this.sgArtN.Count != 0:
                ret = False
            if ret and this.pl.Count != 0:
                ret = False
            if ret and this.plArt.Count != 0:
                ret = False
            return ret

        # Prints a user-friendly summary of the prepositional phrase's forms:
        def print(self):
            ret: str = ""
            ret += "uatha, gan alt:                  "
            f: Form
            for f in this.sg:
                ret += "[" + f.value + "] "
            ret += "\n"

            ret += "uatha, alt, córas lárnach:       "
            f: Form
            for f in this.sgArtS:
                ret += "[" + f.value + "] "
            ret += "\n"
            ret += "uatha, alt, córas an tséimhithe: "
            f: Form
            for f in this.sgArtN:
                ret += "[" + f.value + "] "
            ret += "\n"

            ret += "iolra, gan alt:                  "
            f: Form
            for f in this.pl:
                ret += "[" + f.value + "] "
            ret += "\n"

            ret += "iolra, alt:                      "
            f: Form
            for f in this.plArt:
                ret += "[" + f.value + "] "
            ret += "\n"
            ret = ret.Replace("] [", ", ").Replace("[", "").Replace("] ", "")
            return ret

        def printXml(self) -> ET.ElementTree:
            root: ET.Element = ET.Element("prepositionalPhrase")
            doc: ET.ElementTree = ET.ElementTree(root)

            root.set("default", self.getLemma())
            root.set("prepNick", self.prepNick)
            f: FormSg
            for f in this.sg:
                el = ET.SubElement(root, "sg")
                el.set("default", f.value)
                el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

            f: FormSg
            for f in this.sgArtS:
                el = ET.SubElement(root, "sgArtS")
                el.set("default", f.value)
                el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

            f: FormSg
            for f in this.sgArtN:
                el = ET.SubElement(root, "sgArtN")
                el = ET.SubElement(root, "default", f.value)
                el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

            f: Form
            for f in this.pl:
                el = ET.SubElement(root, "pl")
                el.set("default", f.value)

            f: Form
            for f in this.plArt:
                el = ET.SubElement(root, "plArt")
                el.set("default", f.value)

            return doc

        # Creates a prepositional phrase from a preposition and a noun phrase:
        @classmethod
        def create(cls, prep: Preposition, np: NP):
            if not np.isPossessed:
                pp = cls._populateFromUnpossNP(prep, np)
            else:
                pp = cls._populateFromPossNP(prep, np)
            return pp

        # Populates "this" as a prepositional phrase composed from a preposition and an unpossessed noun phrase:
        @classmethod
        def _populateFromUnpossNP(cls, prep: Preposition, np: NP):
            prepNick = prep.getNickname()
            sg: list[FormSg] = []  # singular, no article
            sgArtN: list[FormSg] = []  # singular, with article, northern system
            sgArtS: list[FormSg] = []  # singular, with article, southern system
            pl: list[Form] = []  # plural, no article
            plArt: list[Form] = []  # plural, with article

            if prepNick == "ag_prep":
                f: FormSg
                for f in np.sgDat:
                    sg.append(FormSg("ag " + f.value, f.gender))
                f: Form
                for f in np.plDat:
                    pl.append(Form("ag " + f.value))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg(
                            "ag an " + Opers.Mutate(Mutation.Len3, f.value), f.gender
                        )
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "ag an "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(Form("ag na " + Opers.Mutate(Mutation.PrefH, f.value)))

            if prepNick == "ar_prep":
                f: FormSg
                for f in np.sgDat:
                    sg.append(
                        FormSg("ar " + Opers.Mutate(Mutation.Len1, f.value), f.gender)
                    )
                f: Form
                for f in np.plDat:
                    pl.append(Form("ar " + Opers.Mutate(Mutation.Len1, f.value)))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg(
                            "ar an " + Opers.Mutate(Mutation.Len3, f.value), f.gender
                        )
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "ar an "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(Form("ar na " + Opers.Mutate(Mutation.PrefH, f.value)))

            if prepNick == "thar_prep":
                f: FormSg
                for f in np.sgDat:
                    sg.append(
                        FormSg("thar " + Opers.Mutate(Mutation.Len1, f.value), f.gender)
                    )
                f: Form
                for f in np.plDat:
                    pl.append(Form("thar " + Opers.Mutate(Mutation.Len1, f.value)))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg(
                            "thar an " + Opers.Mutate(Mutation.Len3, f.value), f.gender
                        )
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "thar an "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(
                        Form("thar na " + Opers.Mutate(Mutation.PrefH, f.value))
                    )

            if prepNick == "as_prep":
                f: FormSg
                for f in np.sgDat:
                    sg.append(FormSg("as " + f.value, f.gender))
                f: Form
                for f in np.plDat:
                    pl.append(Form("as " + f.value))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg(
                            "as an " + Opers.Mutate(Mutation.Len3, f.value), f.gender
                        )
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "as an "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(Form("as na " + Opers.Mutate(Mutation.PrefH, f.value)))

            if prepNick == "chuig_prep":
                f: FormSg
                for f in np.sgDat:
                    sg.append(FormSg("chuig " + f.value, f.gender))
                f: Form
                for f in np.plDat:
                    pl.append(Form("chuig " + f.value))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg(
                            "chuig an " + Opers.Mutate(Mutation.Len3, f.value), f.gender
                        )
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "chuig an "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(
                        Form("chuig na " + Opers.Mutate(Mutation.PrefH, f.value))
                    )

            if prepNick == "de_prep":
                f: FormSg
                for f in np.sgDat:
                    txt: str = Opers.Mutate(Mutation.Len1, f.value)
                    if Opers.StartsVowelFhx(txt):
                        txt = "d'" + txt
                    else:
                        txt = "de " + txt
                    sg.append(FormSg(txt, f.gender))
                f: Form
                for f in np.plDat:
                    txt: str = Opers.Mutate(Mutation.Len1, f.value)
                    if Opers.StartsVowelFhx(txt):
                        txt = "d'" + txt
                    else:
                        txt = "de " + txt
                    pl.append(Form(txt))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg("den " + Opers.Mutate(Mutation.Len3, f.value), f.gender)
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "den "
                            + Opers.Mutate(
                                (
                                    Mutation.Len3
                                    if f.gender == Gender.Fem
                                    else Mutation.Len2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(Form("de na " + Opers.Mutate(Mutation.PrefH, f.value)))

            if prepNick == "do_prep":
                f: FormSg
                for f in np.sgDat:
                    txt: str = Opers.Mutate(Mutation.Len1, f.value)
                    if Opers.StartsVowelFhx(txt):
                        txt = "d'" + txt
                    else:
                        txt = "do " + txt
                    sg.append(FormSg(txt, f.gender))

                f: Form
                for f in np.plDat:
                    txt: str = Opers.Mutate(Mutation.Len1, f.value)
                    if Opers.StartsVowelFhx(txt):
                        txt = "d'" + txt
                    else:
                        txt = "do " + txt
                    pl.append(Form(txt))

                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg("don " + Opers.Mutate(Mutation.Len3, f.value), f.gender)
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "don "
                            + Opers.Mutate(
                                (
                                    Mutation.Len3
                                    if f.gender == Gender.Fem
                                    else Mutation.Len2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(Form("do na " + Opers.Mutate(Mutation.PrefH, f.value)))

            if prepNick == "faoi_prep":
                f: FormSg
                for f in np.sgDat:
                    sg.append(
                        FormSg("faoi " + Opers.Mutate(Mutation.Len1, f.value), f.gender)
                    )
                f: Form
                for f in np.plDat:
                    pl.append(Form("faoi " + Opers.Mutate(Mutation.Len1, f.value)))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg(
                            "faoin " + Opers.Mutate(Mutation.Len3, f.value), f.gender
                        )
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "faoin "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(
                        Form("faoi na " + Opers.Mutate(Mutation.PrefH, f.value))
                    )

            if prepNick == "i_prep":
                f: FormSg
                for f in np.sgDat:
                    if Opers.StartsVowel(f.value):
                        sg.append(FormSg("in " + f.value, f.gender))
                    else:
                        sg.append(
                            FormSg(
                                "i " + Opers.Mutate(Mutation.Ecl1x, f.value), f.gender
                            )
                        )

                f: Form
                for f in np.plDat:
                    if Opers.StartsVowel(f.value):
                        pl.append(Form("in " + f.value))
                    else:
                        pl.append(Form("i " + Opers.Mutate(Mutation.Ecl1x, f.value)))

                f: FormSg
                for f in np.sgDatArtN:
                    txt: str = Opers.Mutate(Mutation.Len3, f.value)
                    if Opers.StartsVowelFhx(txt):
                        txt = "san " + txt
                    else:
                        txt = "sa " + txt
                    sgArtN.append(FormSg(txt, f.gender))

                f: FormSg
                for f in np.sgDatArtS:
                    txt: str = Opers.Mutate(
                        (Mutation.Len3 if f.gender == Gender.Fem else Mutation.Len2),
                        f.value,
                    )
                    if Opers.StartsVowelFhx(txt):
                        txt = "san " + txt
                    else:
                        txt = "sa " + txt
                    sgArtS.append(FormSg(txt, f.gender))

                f: Form
                for f in np.plDatArt:
                    plArt.append(Form("sna " + Opers.Mutate(Mutation.PrefH, f.value)))

            if prepNick == "le_prep":
                f: FormSg
                for f in np.sgDat:
                    sg.append(
                        FormSg("le " + Opers.Mutate(Mutation.PrefH, f.value), f.gender)
                    )
                f: Form
                for f in np.plDat:
                    pl.append(Form("le " + Opers.Mutate(Mutation.PrefH, f.value)))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg(
                            "leis an " + Opers.Mutate(Mutation.Len3, f.value), f.gender
                        )
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "leis an "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(
                        Form("leis na " + Opers.Mutate(Mutation.PrefH, f.value))
                    )

            if prepNick == "ó_prep":
                f: FormSg
                for f in np.sgDat:
                    sg.append(
                        FormSg("ó " + Opers.Mutate(Mutation.Len1, f.value), f.gender)
                    )
                f: Form
                for f in np.plDat:
                    pl.append(Form("ó " + Opers.Mutate(Mutation.Len1, f.value)))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg("ón " + Opers.Mutate(Mutation.Len3, f.value), f.gender)
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "ón "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(Form("ó na " + Opers.Mutate(Mutation.PrefH, f.value)))

            if prepNick == "roimh_prep":
                f: FormSg
                for f in np.sgDat:
                    sg.append(
                        FormSg(
                            "roimh " + Opers.Mutate(Mutation.Len1, f.value), f.gender
                        )
                    )
                f: Form
                for f in np.plDat:
                    pl.append(Form("roimh " + Opers.Mutate(Mutation.Len1, f.value)))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg(
                            "roimh an " + Opers.Mutate(Mutation.Len3, f.value), f.gender
                        )
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "roimh an "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(
                        Form("roimh na " + Opers.Mutate(Mutation.PrefH, f.value))
                    )

            if prepNick == "trí_prep":
                f: FormSg
                for f in np.sgDat:
                    sg.append(
                        FormSg("trí " + Opers.Mutate(Mutation.Len1, f.value), f.gender)
                    )
                f: Form
                for f in np.plDat:
                    pl.append(Form("trí " + Opers.Mutate(Mutation.Len1, f.value)))
                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg(
                            "tríd an " + Opers.Mutate(Mutation.Len3, f.value), f.gender
                        )
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "tríd an "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(
                        Form("trí na " + Opers.Mutate(Mutation.PrefH, f.value))
                    )

            if prepNick == "um_prep":
                f: FormSg
                for f in np.sgDat:
                    txt: str = f.value
                    if not Opers.StartsBilabial(txt):
                        txt = Opers.Mutate(Mutation.Len1, txt)
                    sg.append(FormSg("um " + txt, f.gender))

                f: Form
                for f in np.plDat:
                    txt: str = f.value
                    if not Opers.StartsBilabial(txt):
                        txt = Opers.Mutate(Mutation.Len1, txt)
                    pl.append(Form("um " + txt))

                f: FormSg
                for f in np.sgDatArtN:
                    sgArtN.append(
                        FormSg(
                            "um an " + Opers.Mutate(Mutation.Len3, f.value), f.gender
                        )
                    )
                f: FormSg
                for f in np.sgDatArtS:
                    sgArtS.append(
                        FormSg(
                            "um an "
                            + Opers.Mutate(
                                (
                                    Mutation.Ecl3
                                    if f.gender == Gender.Fem
                                    else Mutation.Ecl2
                                ),
                                f.value,
                            ),
                            f.gender,
                        )
                    )
                f: Form
                for f in np.plDatArt:
                    plArt.append(Form("um na " + Opers.Mutate(Mutation.PrefH, f.value)))
            return cls(sg=sg, sgArtN=sgArtN, pl=pl, plArt=plArt, prepNick=prepNick)

        # Populates "this" as a prepositional phrase composed from a preposition and a possessed noun phrase:
        @classmethod
        def _populateFromPossNP(cls, prep: Preposition, np: NP):
            prepNick = prep.getNickname()
            sg: list[FormSg] = []  # singular, no article
            sgArtN: list[FormSg] = []  # singular, with article, northern system
            sgArtS: list[FormSg] = []  # singular, with article, southern system
            pl: list[Form] = []  # plural, no article
            plArt: list[Form] = []  # plural, with article
            if prepNick == "de_prep" or prepNick == "do_prep":
                f: FormSg
                for f in np.sgDat:
                    if f.value.StartsWith("a "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^a ", "dá "), f.gender)
                        )
                    elif f.value.StartsWith("ár "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^ár ", "dár "), f.gender)
                        )
                    else:
                        sg.append(FormSg(prep.getLemma() + " " + f.value, f.gender))

                f: Form
                for f in np.plDat:
                    if f.value.StartsWith("a "):
                        pl.append(Form(Regex.Replace(f.value, "^a ", "dá ")))
                    elif f.value.StartsWith("ár "):
                        pl.append(Form(Regex.Replace(f.value, "^ár ", "dár ")))
                    else:
                        pl.append(Form(prep.getLemma() + " " + f.value))
            elif prepNick == "faoi_prep":
                f: FormSg
                for f in np.sgDat:
                    if f.value.StartsWith("a "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^a ", "faoina "), f.gender)
                        )
                    elif f.value.StartsWith("ár "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^ár ", "faoinár "), f.gender)
                        )
                    else:
                        sg.append(FormSg(prep.getLemma() + " " + f.value, f.gender))

                f: Form
                for f in np.plDat:
                    if f.value.StartsWith("a "):
                        pl.append(Form(Regex.Replace(f.value, "^a ", "faoina ")))
                    elif f.value.StartsWith("ár "):
                        pl.append(Form(Regex.Replace(f.value, "^ár ", "faoinár ")))
                    else:
                        pl.append(Form(prep.getLemma() + " " + f.value))
            elif prepNick == "i_prep":
                f: FormSg
                for f in np.sgDat:
                    if f.value.StartsWith("a "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^a ", "ina "), f.gender)
                        )
                    elif f.value.StartsWith("ár "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^ár ", "inár "), f.gender)
                        )
                    elif f.value.StartsWith("bhur "):
                        sg.append(
                            FormSg(
                                Regex.Replace(f.value, "^bhur ", "in bhur "), f.gender
                            )
                        )
                    else:
                        sg.append(FormSg(prep.getLemma() + " " + f.value, f.gender))
                f: Form
                for f in np.plDat:
                    if f.value.StartsWith("a "):
                        pl.append(Form(Regex.Replace(f.value, "^a ", "ina ")))
                    elif f.value.StartsWith("ár "):
                        pl.append(Form(Regex.Replace(f.value, "^ár ", "inár ")))
                    elif f.value.StartsWith("bhur "):
                        pl.append(Form(Regex.Replace(f.value, "^bhur ", "in bhur ")))
                    else:
                        pl.append(Form(prep.getLemma() + " " + f.value))
            elif prepNick == "le_prep":
                f: FormSg
                for f in np.sgDat:
                    if f.value.StartsWith("a "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^a ", "lena "), f.gender)
                        )
                    elif f.value.StartsWith("ár "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^ár ", "lenár "), f.gender)
                        )
                    else:
                        sg.append(FormSg(prep.getLemma() + " " + f.value, f.gender))
                f: Form
                for f in np.plDat:
                    if f.value.StartsWith("a "):
                        pl.append(Form(Regex.Replace(f.value, "^a ", "lena ")))
                    elif f.value.StartsWith("ár "):
                        pl.append(Form(Regex.Replace(f.value, "^ár ", "lenár ")))
                    else:
                        pl.append(Form(prep.getLemma() + " " + f.value))
            elif prepNick == "ó_prep":
                f: FormSg
                for f in np.sgDat:
                    if f.value.StartsWith("a "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^a ", "óna "), f.gender)
                        )
                    elif f.value.StartsWith("ár "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^ár ", "ónár "), f.gender)
                        )
                    else:
                        sg.append(FormSg(prep.getLemma() + " " + f.value, f.gender))

                f: Form
                for f in np.plDat:
                    if f.value.StartsWith("a "):
                        pl.append(Form(Regex.Replace(f.value, "^a ", "óna ")))
                    elif f.value.StartsWith("ár "):
                        pl.append(Form(Regex.Replace(f.value, "^ár ", "ónár ")))
                    else:
                        pl.append(Form(prep.getLemma() + " " + f.value))
            elif prepNick == "trí_prep":
                f: FormSg
                for f in np.sgDat:
                    if f.value.StartsWith("a "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^a ", "trína "), f.gender)
                        )
                    elif f.value.StartsWith("ár "):
                        sg.append(
                            FormSg(Regex.Replace(f.value, "^ár ", "trínár "), f.gender)
                        )
                    else:
                        sg.append(FormSg(prep.getLemma() + " " + f.value, f.gender))
                f: Form
                for f in np.plDat:
                    if f.value.StartsWith("a "):
                        pl.append(Form(Regex.Replace(f.value, "^a ", "trína ")))
                    elif f.value.StartsWith("ár "):
                        pl.append(Form(Regex.Replace(f.value, "^ár ", "trínár ")))
                    else:
                        pl.append(Form(prep.getLemma() + " " + f.value))
            else:
                f: FormSg
                for f in np.sgDat:
                    sg.append(FormSg(prep.getLemma() + " " + f.value, f.gender))
                f: Form
                for f in np.plDat:
                    pl.append(Form(prep.getLemma() + " " + f.value))

            return cls(sg=sg, sgArtN=sgArtN, pl=pl, plArt=plArt, prepNick=prepNick)

        # Constructs a prepositional phrase from an XML file in BuNaMo format:
        @classmethod
        def create_from_xml(cls, doc: Union[str, ET.ElementTree]):
            if isinstance(doc, str):
                xml = ET.parse(doc)
                return cls.create_from_xml(xml)

            root = doc.getroot()
            prepNick = root.get("prepNick")
            sg: list[FormSg] = []  # singular, no article
            sgArtN: list[FormSg] = []  # singular, with article, northern system
            sgArtS: list[FormSg] = []  # singular, with article, southern system
            pl: list[Form] = []  # plural, no article
            plArt: list[Form] = []  # plural, with article

            for el in doc.SelectNodes("/*/sg"):
                sg.append(
                    FormSg(
                        el.get("default"),
                        (Gender.Fem if el.get("gender") == "fem" else Gender.Masc),
                    )
                )

            for el in doc.SelectNodes("/*/sgArtN"):
                sgArtN.append(
                    FormSg(
                        el.get("default"),
                        (Gender.Fem if el.get("gender") == "fem" else Gender.Masc),
                    )
                )

            for el in doc.SelectNodes("/*/sgArtS"):
                sgArtS.append(
                    FormSg(
                        el.get("default"),
                        (Gender.Fem if el.get("gender") == "fem" else Gender.Masc),
                    )
                )

            for el in doc.SelectNodes("/*/pl"):
                pl.append(Form(el.get("default")))

            for el in doc.SelectNodes("/*/plArt"):
                plArt.append(Form(el.get("default")))

            return cls(sg=sg, sgArtN=sgArtN, pl=pl, plArt=plArt, prepNick=prepNick)
