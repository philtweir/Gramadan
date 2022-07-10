import sys
import os
import pandas as pd
from typing import Optional
from collections import Counter
from gramadan.features import Gender, FormSg
from gramadan.opers import Opers
from gramadan.v2.database import Database
from .verb import Verb
import tqdm

LEMMA_CSV = os.path.join(os.path.dirname(__file__), "lemma.csv")

class GuessingLemmatizer:
    lemma_map_50pc: dict[tuple[str, str], str]

    def load(self):
        df = pd.read_csv(LEMMA_CSV, comment="#").fillna("")
        self.lemma_map_50pc = {
            (row["POS"], row["from"]): row["to"]
            for _, row in df.iterrows()
        }

    def lemmatize_50pc(self, pos, form) -> Optional[str]:
        for n in range(min(len(form), 5), 0, -1):
            suffix = self.lemma_map_50pc.get((pos, form[-n:]), None)
            if suffix is not None:
                return form[:-n] + suffix
        return None

def run():
    database = Database(sys.argv[1])
    database.load()
    endings = Counter()
    suffices = Counter()
    with tqdm.tqdm(total=len(database)) as pbar:
        for pos, _, ent in database:
            for fld, forms in ent:
                lemma = ent.getLemma()
                for pref, suff in [(frm.value[:-n], frm.value[-n:]) for frm in forms for n in range(1, min(len(frm.value), 6))]:
                    pref = Opers.Demutate(pref)
                    lsuff = lemma[len(pref):]
                    if lemma.startswith(pref) and lsuff != suff:
                        endings[(pos, suff, lsuff)] += 1
                    suffices[(pos, suff)] += 1
        pbar.update(1)
    ratios = {}
    for (pos, suff, lsuff), count in endings.items():
        fraction = count / suffices[(pos, suff)]
        if fraction > 0.5 and count > 1:
            ratios[(suff, pos)] = {"Lemma Suffix": lsuff, "Fraction": fraction}
    print(len(ratios))
    df = pd.DataFrame(ratios).T
    to_remove = []
    for (suff, pos), row in df.iterrows():
        shrt = suff[1:]
        if (shrt, pos) in df.index and df["Lemma Suffix"][(shrt, pos)] == row["Lemma Suffix"][1:]:
            to_remove.append((suff, pos))
    df = df.drop(to_remove)
    success = Counter()
    for pos, _, ent in database:
        for fld, forms in ent:
            yes = False
            lemma = ent.getLemma()
            for frm, suff in [(frm.value, frm.value[-n:]) for frm in forms for n in range(1, min(len(frm.value), 6))]:
                if suff in df.index or frm == lemma:
                    yes = True
            if yes:
                success['hit'] += 1
            success['total'] += 1
    print(f"Using {len(df)} suffices, we can lemmatize {success['hit']/success['total']*100:.2f}% of non-trivial wordforms with >50% certainty")
    with open(LEMMA_CSV, "w") as f:
        f.write("# Lemmatizing suffices, where lemma != demutated form, with >50% probability\n")
        f.write("POS,from,to\n")
        for ix, row in df.iterrows():
            if len(ix[0]) == 1:
                print(ix[1], ix[0], "=>", row["Lemma Suffix"])
            f.write(f"{ix[1]},{ix[0]},{row['Lemma Suffix']}\n")

if __name__ == "__main__":
    run()
    lemmatizer = GuessingLemmatizer()
    lemmatizer.load()
    form = "cabhrú"
    lemma = lemmatizer.lemmatize_50pc("verb", form)
    print(lemma, form)
