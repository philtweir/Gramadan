#!/usr/bin/env python3
"""
Runs NualeargaisNounDeclensionGuesser and NualeargaisFullNounDeclensionGuesser
against all BuNaMo noun XMLs and outputs tab-separated results for diffing
against the Rust cross-validator.

Output format (one line per noun):
    {lemma}\t{declension}\t{gender}\t{simple_guess}\t{full_guess}

Usage:
    python python_guessers.py [noun_xml_dir]

Defaults to /home/philtweir/Cód/Oscailte/Gramadan/data/noun/
"""
import sys
import os
import glob
import logging

# Suppress the v2 import warning
logging.disable(logging.WARNING)

# We need the Python package on the path
PYTHON_PKG = os.path.join(os.path.dirname(__file__), '..', '..', 'Python')
sys.path.insert(0, os.path.abspath(PYTHON_PKG))

# Activate the v2 package (patches opers, features, etc.)
import gramadan.v2  # noqa: F401

from lxml import etree as ET
from gramadan.features import Gender, FormSg, FormList
from gramadan import noun as noun_module
from gramadan.v2.noun import Noun
from gramadan.v2.noun_nualeargais import (
    NualeargaisNounDeclensionGuesser,
    NualeargaisFullNounDeclensionGuesser,
)

logging.disable(logging.NOTSET)

NOUN_DIR = sys.argv[1] if len(sys.argv) > 1 else (
    os.path.join(os.path.dirname(__file__), '..', '..', 'data', 'noun')
)
NOUN_DIR = os.path.abspath(NOUN_DIR)

simple_guesser = NualeargaisNounDeclensionGuesser()
full_guesser = NualeargaisFullNounDeclensionGuesser()

xml_files = sorted(glob.glob(os.path.join(NOUN_DIR, '*.xml')))

for xml_path in xml_files:
    try:
        tree = ET.parse(xml_path)
    except Exception as e:
        print(f"# PARSE ERROR {xml_path}: {e}", file=sys.stderr)
        continue

    root = tree.getroot()

    # Read declension
    try:
        declension = int(root.get('declension', '0'))
    except (ValueError, TypeError):
        declension = 0

    # Skip if not declension 1-5
    if declension < 1 or declension > 5:
        continue

    # Read lemma
    lemma = root.get('default', '')
    if not lemma:
        continue

    # Read sgNom elements
    sg_nom_els = root.findall('sgNom')
    sg_gen_els = root.findall('sgGen')

    # Skip if count != 1 for either
    if len(sg_nom_els) != 1 or len(sg_gen_els) != 1:
        continue

    nom_el = sg_nom_els[0]
    gen_el = sg_gen_els[0]

    nom_val = nom_el.get('default', '')
    gen_val = gen_el.get('default', '')
    gender_str = nom_el.get('gender', '')

    if gender_str == 'masc':
        gender = Gender.Masc
    elif gender_str == 'fem':
        gender = Gender.Fem
    else:
        continue

    # Build a v1 Noun with just sgNom and sgGen populated
    sg_nom_form = FormSg(nom_val, gender)
    sg_gen_form = FormSg(gen_val, gender)

    sg_nom_list = FormList([sg_nom_form])
    sg_gen_list = FormList([sg_gen_form])

    v1_noun = noun_module.Noun(
        sgNom=sg_nom_list,
        sgGen=sg_gen_list,
        declension=declension,
    )
    focal = Noun(v1=v1_noun)

    gender_label = 'masc' if gender == Gender.Masc else 'fem'

    # Run simple guesser
    simple_guess = None
    try:
        simple_guess = simple_guesser.guess(focal)
    except Exception as e:
        print(f"# SIMPLE GUESSER ERROR {lemma}: {e}", file=sys.stderr)

    # Run full guesser
    full_guess = None
    try:
        full_guess = full_guesser.guess(focal)
    except Exception as e:
        print(f"# FULL GUESSER ERROR {lemma}: {e}", file=sys.stderr)

    simple_str = str(simple_guess) if simple_guess is not None else 'ERROR'
    full_str = str(full_guess) if full_guess is not None else 'ERROR'

    print(f"{lemma}\t{declension}\t{gender_label}\t{simple_str}\t{full_str}")
