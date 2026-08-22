//! Cross-validates the Rust gramadan implementation against BuNaMo XML data.
//!
//! Tests:
//! 1. Noun declension guessing (lemma+gender) vs BuNaMo declension
//! 2. Noun genitive generation (lemma+gender+declension) vs BuNaMo genitive
//! 3. Verb conjugation class inference vs BuNaMo future-form method

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use gramadan::adjective;
use gramadan::features::Gender;
use gramadan::noun;
use gramadan::noun::Declension;
use gramadan::singular_info;
use gramadan::verb;

fn main() {
    let mut args = std::env::args().skip(1).peekable();

    let mut dump_guesses = false;
    let mut tearma_tsv: Option<String> = None;
    let mut kaikki_tsv: Option<String> = None;
    let mut data_dir = "../data".to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dump-guesses" => dump_guesses = true,
            "--tearma" => tearma_tsv = args.next(),
            "--kaikki" => kaikki_tsv = args.next(),
            _ => data_dir = arg,
        }
    }

    let data_path = Path::new(&data_dir);

    if dump_guesses {
        dump_noun_guesses(data_path);
        return;
    }

    if let Some(ref tsv_path) = tearma_tsv {
        validate_tearma(data_path, tsv_path);
        return;
    }

    if let Some(ref tsv_path) = kaikki_tsv {
        validate_kaikki(data_path, tsv_path);
        return;
    }

    println!("=== Gramadán Rust Cross-Validation ===\n");

    validate_noun_declension_guessing(data_path);
    println!();
    validate_noun_genitive_generation(data_path);
    println!();
    validate_verb_conjugation(data_path);
    println!();
    validate_verb_paradigms(data_path);
    println!();
    validate_verbal_adjective(data_path);
    println!();
    validate_verbal_noun(data_path);
    println!();
    validate_adjective_declension_guessing(data_path);
    println!();
    validate_adjective_form_generation(data_path);
}

// ---- Noun XML parsing ----

struct BuNaMoNoun {
    lemma: String,
    gender: Gender,
    declension: i8,
    sg_gen: String,
}

fn parse_noun_xml(path: &Path) -> Option<BuNaMoNoun> {
    let content = fs::read_to_string(path).ok()?;
    // Strip BOM
    let content = content.trim_start_matches('\u{feff}');

    let mut lemma = String::new();
    let mut gender = None;
    let mut declension: i8 = 0;
    let mut sg_gen = String::new();

    let mut reader = quick_xml::Reader::from_str(content);
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Empty(ref e)) | Ok(quick_xml::events::Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"noun" => {
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"default" => {
                                    lemma = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"declension" => {
                                    let v = String::from_utf8_lossy(&attr.value);
                                    declension = v.parse().unwrap_or(0);
                                }
                                _ => {}
                            }
                        }
                    }
                    b"sgNom" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"gender" {
                                let v = String::from_utf8_lossy(&attr.value);
                                gender = Some(match v.as_ref() {
                                    "masc" => Gender::Masc,
                                    "fem" => Gender::Fem,
                                    _ => return None,
                                });
                            }
                        }
                    }
                    b"sgGen" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"default" {
                                sg_gen =
                                    String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(_) => return None,
            _ => {}
        }
    }

    if lemma.is_empty() || gender.is_none() {
        return None;
    }

    Some(BuNaMoNoun {
        lemma,
        gender: gender.unwrap(),
        declension,
        sg_gen,
    })
}

// ---- Verb XML parsing ----

struct BuNaMoVerb {
    lemma: String,
    future_indep_base: String,
}

fn parse_verb_xml(path: &Path) -> Option<BuNaMoVerb> {
    let content = fs::read_to_string(path).ok()?;
    let content = content.trim_start_matches('\u{feff}');

    let mut lemma = String::new();
    let mut future_indep_base = String::new();

    let mut reader = quick_xml::Reader::from_str(content);

    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Empty(ref e)) | Ok(quick_xml::events::Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"verb" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"default" {
                                lemma = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    b"tenseForm" => {
                        let mut is_fut = false;
                        let mut is_indep = false;
                        let mut is_base = false;
                        let mut default = String::new();

                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"default" => {
                                    default = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"tense" => {
                                    is_fut = attr.value.as_ref() == b"Fut";
                                }
                                b"dependency" => {
                                    is_indep = attr.value.as_ref() == b"Indep";
                                }
                                b"person" => {
                                    is_base = attr.value.as_ref() == b"Base";
                                }
                                _ => {}
                            }
                        }

                        if is_fut && is_indep && is_base && future_indep_base.is_empty() {
                            future_indep_base = default;
                        }
                    }
                    _ => {}
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(_) => return None,
            _ => {}
        }
    }

    if lemma.is_empty() {
        return None;
    }

    Some(BuNaMoVerb {
        lemma,
        future_indep_base,
    })
}

// ---- Dump mode ----

/// Outputs one TSV line per noun (declension 1-5, single sgNom/sgGen):
///   lemma\tdeclension\tgender\tsimple_guess\tfull_guess
///
/// Matches the format of scripts/python_guessers.py for diffing.
fn dump_noun_guesses(data_path: &Path) {
    let noun_dir = data_path.join("noun");
    let mut entries: Vec<_> = fs::read_dir(&noun_dir)
        .expect("Cannot read noun directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();

    // Sort by filename for deterministic output (matches Python sorted())
    entries.sort_by_key(|e| e.path());

    for entry in &entries {
        let path = entry.path();
        let noun = match parse_noun_xml(&path) {
            Some(n) => n,
            None => continue,
        };

        if noun.declension < 1 || noun.declension > 5 || noun.sg_gen.is_empty() {
            continue;
        }

        let gender_str = match noun.gender {
            Gender::Masc => "masc",
            Gender::Fem => "fem",
        };

        let simple_guess = noun::guess_declension(&noun.lemma, noun.gender);
        let full_guess = noun::guess_declension_full(&noun.lemma, noun.gender);

        println!(
            "{}\t{}\t{}\t{}\t{}",
            noun.lemma,
            noun.declension,
            gender_str,
            simple_guess.map_or(-1, |d| d.as_i8()),
            full_guess.as_i8(),
        );
    }
}

// ---- Validation routines ----

fn validate_noun_declension_guessing(data_path: &Path) {
    println!("--- Noun declension guessing (lemma+gender only) ---");

    let noun_dir = data_path.join("noun");
    let entries: Vec<_> = fs::read_dir(&noun_dir)
        .expect("Cannot read noun directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();

    // First pass: build LemmaDb from all nouns with declension 1-5
    let mut db = noun::LemmaDb::new();
    let mut test_nouns: Vec<BuNaMoNoun> = Vec::new();

    for entry in &entries {
        let path = entry.path();
        let n = match parse_noun_xml(&path) {
            Some(n) => n,
            None => continue,
        };
        if n.declension >= 1 && n.declension <= 5 {
            db.insert(n.lemma.clone(), n.declension, n.gender);
            test_nouns.push(n);
        }
    }
    println!("LemmaDb: {} entries", db.len());

    let mut total = 0;
    let mut correct_simple = 0;
    let mut none_simple = 0;
    let mut correct_full = 0;
    let mut correct_compound = 0;
    let mut none_compound = 0;
    let mut confusion_simple: HashMap<(i8, i8), usize> = HashMap::new();
    let mut confusion_full: HashMap<(i8, i8), usize> = HashMap::new();
    let mut confusion_compound: HashMap<(i8, i8), usize> = HashMap::new();

    for noun in &test_nouns {
        total += 1;
        let expected = noun.declension;

        let guessed_simple = noun::guess_declension(&noun.lemma, noun.gender);
        let guessed_full = noun::guess_declension_full(&noun.lemma, noun.gender);
        let guessed_compound = noun::guess_declension_compound(&noun.lemma, noun.gender, &db);

        match guessed_simple {
            Some(gs) if gs.as_i8() == expected => { correct_simple += 1; }
            Some(gs) => {
                *confusion_simple
                    .entry((expected, gs.as_i8()))
                    .or_insert(0) += 1;
            }
            None => { none_simple += 1; }
        }

        if guessed_full.as_i8() == expected {
            correct_full += 1;
        } else {
            *confusion_full
                .entry((expected, guessed_full.as_i8()))
                .or_insert(0) += 1;
        }

        match guessed_compound {
            Some(gc) if gc.as_i8() == expected => { correct_compound += 1; }
            Some(gc) => {
                *confusion_compound
                    .entry((expected, gc.as_i8()))
                    .or_insert(0) += 1;
            }
            None => { none_compound += 1; }
        }
    }

    println!("Total nouns with declension 1-5: {}", total);
    let answered_simple = total - none_simple;
    println!(
        "Simple guesser:  {}/{} answered ({:.2}%), {} abstained, {} wrong",
        correct_simple,
        answered_simple,
        100.0 * correct_simple as f64 / answered_simple as f64,
        none_simple,
        answered_simple - correct_simple,
    );
    println!(
        "Full guesser:    {}/{} ({:.2}%)",
        correct_full,
        total,
        100.0 * correct_full as f64 / total as f64
    );
    let answered_compound = total - none_compound;
    println!(
        "Compound guesser: {}/{} answered ({:.2}%), {} abstained, {} wrong",
        correct_compound,
        answered_compound,
        100.0 * correct_compound as f64 / answered_compound as f64,
        none_compound,
        answered_compound - correct_compound,
    );

    println!("\nSimple guesser confusion matrix (expected→guessed, count):");
    let mut conf: Vec<_> = confusion_simple.iter().collect();
    conf.sort_by(|a, b| b.1.cmp(a.1));
    for ((exp, got), count) in conf.iter().take(15) {
        println!("  dec{}→dec{}: {}", exp, got, count);
    }

    println!("\nFull guesser confusion matrix (expected→guessed, count):");
    let mut conf: Vec<_> = confusion_full.iter().collect();
    conf.sort_by(|a, b| b.1.cmp(a.1));
    for ((exp, got), count) in conf.iter().take(15) {
        println!("  dec{}→dec{}: {}", exp, got, count);
    }

    println!("\nCompound guesser confusion matrix (expected→guessed, count):");
    let mut conf: Vec<_> = confusion_compound.iter().collect();
    conf.sort_by(|a, b| b.1.cmp(a.1));
    for ((exp, got), count) in conf.iter().take(15) {
        println!("  dec{}→dec{}: {}", exp, got, count);
    }
}

fn validate_noun_genitive_generation(data_path: &Path) {
    println!("--- Noun genitive generation (lemma+gender+declension) ---");

    let noun_dir = data_path.join("noun");
    let entries: Vec<_> = fs::read_dir(&noun_dir)
        .expect("Cannot read noun directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();

    let mut total = 0;
    let mut correct = 0;
    let mut errors_by_decl: HashMap<i8, Vec<(String, String, String)>> = HashMap::new();

    for entry in &entries {
        let path = entry.path();
        let noun = match parse_noun_xml(&path) {
            Some(n) => n,
            None => continue,
        };

        if noun.declension < 1 || noun.declension > 5 || noun.sg_gen.is_empty() {
            continue;
        }

        total += 1;
        let expected_gen = &noun.sg_gen;

        // Generate genitive using strategy selection
        let generated_gen = noun::generate_genitive(&noun.lemma, noun.gender, noun.declension);

        if let Some(ref gen) = generated_gen {
            if gen == expected_gen {
                correct += 1;
            } else {
                errors_by_decl
                    .entry(noun.declension)
                    .or_default()
                    .push((noun.lemma.clone(), expected_gen.clone(), gen.clone()));
            }
        } else {
            errors_by_decl
                .entry(noun.declension)
                .or_default()
                .push((noun.lemma.clone(), expected_gen.clone(), "(none)".to_string()));
        }
    }

    println!("Total nouns with declension 1-5 and genitive: {}", total);
    println!(
        "Correct genitives: {}/{} ({:.2}%)",
        correct,
        total,
        100.0 * correct as f64 / total as f64
    );

    for dec in 1..=5 {
        let errs = errors_by_decl.get(&dec).map_or(0, |v| v.len());
        println!("\n  Declension {}: {} errors", dec, errs);
        if let Some(errors) = errors_by_decl.get(&dec) {
            for (lemma, expected, got) in errors {
                println!("    {} : expected '{}', got '{}'", lemma, expected, got);
            }
        }
    }
}

fn validate_verb_conjugation(data_path: &Path) {
    println!("--- Verb conjugation class inference ---");

    let verb_dir = data_path.join("verb");
    let entries: Vec<_> = fs::read_dir(&verb_dir)
        .expect("Cannot read verb directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();

    let mut total = 0;
    let mut correct_lemma_only = 0;
    let mut correct_with_future = 0;
    let mut confusion_lemma: HashMap<(String, String), usize> = HashMap::new();

    let mut class_counts: HashMap<String, usize> = HashMap::new();

    for entry in &entries {
        let path = entry.path();
        let v = match parse_verb_xml(&path) {
            Some(v) => v,
            None => continue,
        };

        if v.future_indep_base.is_empty() {
            continue;
        }

        total += 1;

        let gt = verb::get_conjugation_from_future(&v.lemma, &v.future_indep_base);
        let gt_str = format!("{:?}", gt);
        *class_counts.entry(gt_str.clone()).or_insert(0) += 1;

        let guessed = verb::guess_conjugation(&v.lemma);

        if guessed == gt {
            correct_lemma_only += 1;
        } else {
            let key = (format!("{:?}", gt), format!("{:?}", guessed));
            *confusion_lemma.entry(key).or_insert(0) += 1;
        }

        let rust_future = verb::get_conjugation_from_future(&v.lemma, &v.future_indep_base);
        if rust_future == gt {
            correct_with_future += 1;
        }
    }

    println!("Total verbs with future form: {}", total);
    println!(
        "Class distribution: {:?}",
        class_counts
    );
    println!(
        "Lemma-only heuristic: {}/{} ({:.2}%)",
        correct_lemma_only,
        total,
        100.0 * correct_lemma_only as f64 / total as f64
    );
    println!(
        "Future-based method:  {}/{} ({:.2}%)",
        correct_with_future,
        total,
        100.0 * correct_with_future as f64 / total as f64
    );

    if !confusion_lemma.is_empty() {
        println!("\nLemma-only confusion (expected→guessed, count):");
        let mut conf: Vec<_> = confusion_lemma.iter().collect();
        conf.sort_by(|a, b| b.1.cmp(a.1));
        for ((exp, got), count) in conf.iter().take(10) {
            println!("  {}→{}: {}", exp, got, count);
        }
    }
}

fn validate_verb_paradigms(data_path: &Path) {
    println!("--- Verb paradigm generation (full form accuracy) ---");

    let verb_dir = data_path.join("verb");
    let mut entries: Vec<_> = fs::read_dir(&verb_dir)
        .expect("Cannot read verb directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();
    entries.sort_by_key(|e| e.path());

    let mut total_verbs = 0;
    let mut verbs_perfect = 0;
    let mut total_forms = 0;
    let mut correct_forms = 0;
    let mut skipped_irregular = 0;
    let mut errors_by_slot: HashMap<String, Vec<(String, String, String)>> = HashMap::new();

    let persons = ["base", "sg1", "sg2", "sg3", "pl1", "pl2", "pl3", "auto"];

    for entry in &entries {
        let path = entry.path();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let gold = verb::Verb::from_xml(&content);
        let lemma = gold.get_lemma().to_string();
        if lemma.is_empty() { continue; }

        // Use future-based classification (100% accurate) to isolate
        // form-generation errors from classification errors
        let fut_base = gold.fut.indep.base.first().map(|f| f.value.as_str()).unwrap_or("");
        let class = verb::get_conjugation_from_future(&lemma, fut_base);
        if class == verb::VerbConjugationClass::Irregular {
            skipped_irregular += 1;
            continue;
        }

        let gen = verb::Verb::from_lemma(&lemma, class);
        total_verbs += 1;
        let mut verb_ok = true;

        let tense_pairs: &[(&str, &verb::TenseForms, &verb::TenseForms)] = &[
            ("past",      &gold.past,      &gen.past),
            ("past_cont", &gold.past_cont, &gen.past_cont),
            ("pres",      &gold.pres,      &gen.pres),
            ("pres_cont", &gold.pres_cont, &gen.pres_cont),
            ("fut",       &gold.fut,       &gen.fut),
            ("cond",      &gold.cond,      &gen.cond),
        ];

        for (tense_name, gold_tf, gen_tf) in tense_pairs {
            for dep_name in &["indep", "dep"] {
                let (gold_pf, gen_pf) = match *dep_name {
                    "indep" => (&gold_tf.indep, &gen_tf.indep),
                    "dep"   => (&gold_tf.dep,   &gen_tf.dep),
                    _ => unreachable!(),
                };

                for (i, p_name) in persons.iter().enumerate() {
                    let gold_slot = get_person_forms(gold_pf, i);
                    let gen_slot = get_person_forms(gen_pf, i);

                    if gold_slot.is_empty() && gen_slot.is_empty() {
                        continue;
                    }

                    total_forms += 1;
                    let gold_val = gold_slot.first().map(|f| f.value.as_str()).unwrap_or("");
                    let gen_val = gen_slot.first().map(|f| f.value.as_str()).unwrap_or("");

                    if gold_val == gen_val {
                        correct_forms += 1;
                    } else {
                        verb_ok = false;
                        let slot = format!("{}.{}.{}", tense_name, dep_name, p_name);
                        errors_by_slot
                            .entry(slot)
                            .or_default()
                            .push((lemma.clone(), gold_val.to_string(), gen_val.to_string()));
                    }
                }
            }
        }

        let mood_pairs: &[(&str, &verb::PersonForms, &verb::PersonForms)] = &[
            ("imper", &gold.imper, &gen.imper),
            ("subj",  &gold.subj,  &gen.subj),
        ];

        for (mood_name, gold_pf, gen_pf) in mood_pairs {
            for (i, p_name) in persons.iter().enumerate() {
                let gold_slot = get_person_forms(gold_pf, i);
                let gen_slot = get_person_forms(gen_pf, i);

                if gold_slot.is_empty() && gen_slot.is_empty() {
                    continue;
                }

                total_forms += 1;
                let gold_val = gold_slot.first().map(|f| f.value.as_str()).unwrap_or("");
                let gen_val = gen_slot.first().map(|f| f.value.as_str()).unwrap_or("");

                if gold_val == gen_val {
                    correct_forms += 1;
                } else {
                    verb_ok = false;
                    let slot = format!("{}.{}", mood_name, p_name);
                    errors_by_slot
                        .entry(slot)
                        .or_default()
                        .push((lemma.clone(), gold_val.to_string(), gen_val.to_string()));
                }
            }
        }

        if verb_ok {
            verbs_perfect += 1;
        }
    }

    println!("Regular verbs tested: {} (skipped {} irregulars)", total_verbs, skipped_irregular);
    println!(
        "Perfect verbs (all forms match): {}/{} ({:.2}%)",
        verbs_perfect, total_verbs,
        100.0 * verbs_perfect as f64 / total_verbs as f64
    );
    println!(
        "Form-level accuracy: {}/{} ({:.2}%)",
        correct_forms, total_forms,
        100.0 * correct_forms as f64 / total_forms as f64
    );

    let mut slot_errors: Vec<_> = errors_by_slot.iter().collect();
    slot_errors.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    println!("\nErrors by slot (top 20):");
    for (slot, errors) in slot_errors.iter().take(20) {
        println!("  {} — {} errors", slot, errors.len());
        let show = 5;
        for (lemma, expected, got) in errors.iter().take(show) {
            println!("    {}: expected '{}', got '{}'", lemma, expected, got);
        }
        if errors.len() > show {
            println!("    ... and {} more", errors.len() - show);
        }
    }

    let mut all_failing: std::collections::HashSet<String> = std::collections::HashSet::new();
    for errors in errors_by_slot.values() {
        for (lemma, _, _) in errors {
            all_failing.insert(lemma.clone());
        }
    }
    let mut failing_sorted: Vec<_> = all_failing.into_iter().collect();
    failing_sorted.sort();
    println!("\nAll failing verb lemmas ({}):", failing_sorted.len());
    for l in &failing_sorted {
        print!("  {}", l);
    }
    println!();

    let mut per_verb: HashMap<String, usize> = HashMap::new();
    for errors in errors_by_slot.values() {
        for (lemma, _, _) in errors {
            *per_verb.entry(lemma.clone()).or_insert(0) += 1;
        }
    }
    let mut pv: Vec<_> = per_verb.into_iter().collect();
    pv.sort_by(|a, b| b.1.cmp(&a.1));
    println!("\nErrors per verb:");
    for (lemma, count) in &pv {
        println!("  {}: {}", lemma, count);
    }
}

fn get_person_forms(pf: &verb::PersonForms, index: usize) -> &Vec<gramadan::features::Form> {
    match index {
        0 => &pf.base,
        1 => &pf.sg1,
        2 => &pf.sg2,
        3 => &pf.sg3,
        4 => &pf.pl1,
        5 => &pf.pl2,
        6 => &pf.pl3,
        7 => &pf.auto,
        _ => unreachable!(),
    }
}

fn validate_verbal_adjective(data_path: &Path) {
    println!("--- Verbal adjective generation ---");

    let verb_dir = data_path.join("verb");
    let mut entries: Vec<_> = fs::read_dir(&verb_dir)
        .expect("Cannot read verb directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();
    entries.sort_by_key(|e| e.path());

    let mut total = 0;
    let mut correct = 0;
    let mut no_va = 0;
    let mut errors: Vec<(String, String, String, String)> = Vec::new(); // (lemma, conj, expected, got)

    for entry in &entries {
        let path = entry.path();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let gold = verb::Verb::from_xml(&content);
        let lemma = gold.get_lemma().to_string();
        if lemma.is_empty() { continue; }

        let gold_va = match gold.verbal_adjective.first() {
            Some(f) => f.value.clone(),
            None => { no_va += 1; continue; },
        };

        let fut_base = gold.fut.indep.base.first().map(|f| f.value.as_str()).unwrap_or("");
        let class = verb::get_conjugation_from_future(&lemma, fut_base);
        if class == verb::VerbConjugationClass::Irregular {
            continue;
        }

        let gen = verb::Verb::from_lemma(&lemma, class);
        let gen_va = gen.verbal_adjective.first().map(|f| f.value.as_str()).unwrap_or("");

        total += 1;
        if gen_va == gold_va {
            correct += 1;
        } else {
            let conj_name = match class {
                verb::VerbConjugationClass::First => "1",
                verb::VerbConjugationClass::Second => "2",
                _ => "?",
            };
            errors.push((lemma, conj_name.to_string(), gold_va, gen_va.to_string()));
        }
    }

    println!("Verbs with VA in BuNaMo: {} (skipped {} without VA)", total, no_va);
    println!(
        "Correct: {}/{} ({:.2}%)",
        correct, total,
        100.0 * correct as f64 / total as f64
    );
    println!("Errors: {}", errors.len());

    // Group by conjugation
    let errs_1: Vec<_> = errors.iter().filter(|e| e.1 == "1").collect();
    let errs_2: Vec<_> = errors.iter().filter(|e| e.1 == "2").collect();

    if !errs_1.is_empty() {
        println!("\n  1st conjugation errors ({}):", errs_1.len());
        for (lemma, _, expected, got) in errs_1.iter().take(30) {
            println!("    {} : expected '{}', got '{}'", lemma, expected, got);
        }
        if errs_1.len() > 30 {
            println!("    ... and {} more", errs_1.len() - 30);
        }
    }

    if !errs_2.is_empty() {
        println!("\n  2nd conjugation errors ({}):", errs_2.len());
        for (lemma, _, expected, got) in errs_2.iter().take(30) {
            println!("    {} : expected '{}', got '{}'", lemma, expected, got);
        }
        if errs_2.len() > 30 {
            println!("    ... and {} more", errs_2.len() - 30);
        }
    }
}

fn validate_verbal_noun(data_path: &Path) {
    println!("--- Verbal noun generation ---");

    let verb_dir = data_path.join("verb");
    let mut entries: Vec<_> = fs::read_dir(&verb_dir)
        .expect("Cannot read verb directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();
    entries.sort_by_key(|e| e.path());

    let mut total = 0;
    let mut correct = 0;
    let mut no_vn = 0;
    let mut abstained = 0;
    let mut errors: Vec<(String, String, String, String)> = Vec::new();

    for entry in &entries {
        let path = entry.path();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let gold = verb::Verb::from_xml(&content);
        let lemma = gold.get_lemma().to_string();
        if lemma.is_empty() { continue; }

        let gold_vn = match gold.verbal_noun.first() {
            Some(f) => f.value.clone(),
            None => { no_vn += 1; continue; },
        };

        let fut_base = gold.fut.indep.base.first().map(|f| f.value.as_str()).unwrap_or("");
        let class = verb::get_conjugation_from_future(&lemma, fut_base);
        if class == verb::VerbConjugationClass::Irregular {
            continue;
        }

        let gen = verb::Verb::from_lemma(&lemma, class);
        let gen_vn = gen.verbal_noun.first().map(|f| f.value.as_str()).unwrap_or("");

        total += 1;
        if gen_vn.is_empty() {
            abstained += 1;
            continue;
        }
        if gen_vn == gold_vn {
            correct += 1;
        } else {
            let conj_name = match class {
                verb::VerbConjugationClass::First => "1",
                verb::VerbConjugationClass::Second => "2",
                _ => "?",
            };
            errors.push((lemma, conj_name.to_string(), gold_vn, gen_vn.to_string()));
        }
    }

    println!("Verbs with VN in BuNaMo: {} (skipped {} without VN)", total, no_vn);
    if abstained > 0 {
        println!("Abstained: {}", abstained);
    }
    println!(
        "Correct: {}/{} ({:.2}%)",
        correct, total - abstained,
        100.0 * correct as f64 / (total - abstained) as f64
    );
    println!("Errors: {}", errors.len());

    let errs_1: Vec<_> = errors.iter().filter(|e| e.1 == "1").collect();
    let errs_2: Vec<_> = errors.iter().filter(|e| e.1 == "2").collect();

    if !errs_1.is_empty() {
        println!("\n  1st conjugation errors ({}):", errs_1.len());
        for (lemma, _, expected, got) in errs_1.iter().take(30) {
            println!("    {} : expected '{}', got '{}'", lemma, expected, got);
        }
        if errs_1.len() > 30 {
            println!("    ... and {} more", errs_1.len() - 30);
        }
    }

    if !errs_2.is_empty() {
        println!("\n  2nd conjugation errors ({}):", errs_2.len());
        for (lemma, _, expected, got) in errs_2.iter().take(30) {
            println!("    {} : expected '{}', got '{}'", lemma, expected, got);
        }
        if errs_2.len() > 30 {
            println!("    ... and {} more", errs_2.len() - 30);
        }
    }
}

// ---- Téarma validation ----

/// Validate the guesser against Téarma nouns.
///
/// Reads a TSV file with: lemma\tclass\tgender
/// Builds a LemmaDb from BuNaMo, then runs all three guessers.
fn validate_tearma(data_path: &Path, tsv_path: &str) {
    println!("=== Téarma Noun Declension Validation ===\n");

    // Build LemmaDb from BuNaMo
    let noun_dir = data_path.join("noun");
    let entries: Vec<_> = fs::read_dir(&noun_dir)
        .expect("Cannot read noun directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();

    let mut db = noun::LemmaDb::new();
    for entry in &entries {
        if let Some(n) = parse_noun_xml(&entry.path()) {
            if n.declension >= 1 && n.declension <= 5 {
                db.insert(n.lemma.clone(), n.declension, n.gender);
            }
        }
    }
    println!("LemmaDb: {} entries from BuNaMo", db.len());

    // Read Téarma TSV
    let content = fs::read_to_string(tsv_path).expect("Cannot read Téarma TSV");
    let mut total = 0;
    let mut correct_simple = 0;
    let mut correct_compound = 0;
    let mut confusion_simple: HashMap<(i8, i8), usize> = HashMap::new();
    let mut confusion_compound: HashMap<(i8, i8), usize> = HashMap::new();
    let mut compound_changed = 0;
    let mut compound_fixed = 0;
    let mut compound_broke = 0;

    for line in content.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 { continue; }

        let lemma = parts[0];
        let expected: i8 = match parts[1].parse() {
            Ok(v) if v >= 1 && v <= 5 => v,
            _ => continue,
        };
        let gender = match parts[2] {
            "masc" => Gender::Masc,
            "fem" => Gender::Fem,
            _ => continue,
        };

        total += 1;

        let simple = noun::guess_declension(lemma, gender);
        let compound = noun::guess_declension_compound(lemma, gender, &db);

        if let Some(s) = simple {
            if s.as_i8() == expected {
                correct_simple += 1;
            } else {
                *confusion_simple.entry((expected, s.as_i8())).or_insert(0) += 1;
            }
        }

        if let Some(c) = compound {
            if c.as_i8() == expected {
                correct_compound += 1;
            } else {
                *confusion_compound.entry((expected, c.as_i8())).or_insert(0) += 1;
            }
        }

        let simple_i8 = simple.map(|d| d.as_i8());
        let compound_i8 = compound.map(|d| d.as_i8());
        if compound_i8 != simple_i8 {
            compound_changed += 1;
            if compound_i8 == Some(expected) {
                compound_fixed += 1;
            } else if simple_i8 == Some(expected) {
                compound_broke += 1;
            }
        }
    }

    println!("Total Téarma nouns with class 1-5: {}", total);
    println!(
        "Simple guesser:   {}/{} ({:.2}%)",
        correct_simple, total, 100.0 * correct_simple as f64 / total as f64
    );
    println!(
        "Compound guesser: {}/{} ({:.2}%)",
        correct_compound, total, 100.0 * correct_compound as f64 / total as f64
    );
    println!(
        "\nCompound decomposition impact: {} changed, {} fixed, {} broke",
        compound_changed, compound_fixed, compound_broke
    );

    println!("\nSimple guesser confusion (expected→guessed):");
    let mut conf: Vec<_> = confusion_simple.iter().collect();
    conf.sort_by(|a, b| b.1.cmp(a.1));
    for ((exp, got), count) in conf.iter().take(10) {
        println!("  dec{}→dec{}: {}", exp, got, count);
    }

    println!("\nCompound guesser confusion (expected→guessed):");
    let mut conf: Vec<_> = confusion_compound.iter().collect();
    conf.sort_by(|a, b| b.1.cmp(a.1));
    for ((exp, got), count) in conf.iter().take(10) {
        println!("  dec{}→dec{}: {}", exp, got, count);
    }
}

// ---- Kaikki/Wiktionary validation ----

/// Recognise declension from (lemma, gender, known_genitive) by trying
/// each SingularInfo strategy and checking which produces the correct genitive.
/// Order: 5th → 4th → 3rd → 2nd → 1st (matches Python NounDeclensionGuesser).
/// Returns 0 if no strategy matches.
fn recognise_declension(lemma: &str, gender: Gender, genitive: &str) -> i8 {
    // 5th: various strategies
    let fifth_checks: &[fn(&str, Gender) -> singular_info::SingularInfo] = &[
        |l, g| singular_info::singular_info_l(l, g, ""),
        |l, g| singular_info::singular_info_n(l, g),
        |l, g| singular_info::singular_info_d(l, g),
        |l, g| singular_info::singular_info_ax(l, g, true, ""),
        |l, g| singular_info::singular_info_ax(l, g, false, ""),
    ];
    for check in fifth_checks {
        let si = check(lemma, gender);
        if si.genitive.first().map(|f| f.value.as_str()) == Some(genitive) {
            // 5th only if the genitive actually differs from nom
            // (otherwise 4th takes priority)
            if lemma != genitive {
                return 5;
            }
        }
    }

    // 4th: gen = nom
    if lemma == genitive {
        return 4;
    }

    // 3rd: SingularInfoA (broaden + -a)
    let si = singular_info::singular_info_a(lemma, gender, false, "", true);
    if si.genitive.first().map(|f| f.value.as_str()) == Some(genitive) {
        return 3;
    }

    // 2nd: SingularInfoE (slenderize + -e) without syncope
    let si = singular_info::singular_info_e(lemma, gender, false, false, "", true);
    if si.genitive.first().map(|f| f.value.as_str()) == Some(genitive) {
        return 2;
    }
    // 2nd: SingularInfoC for fem -ach → -aí
    if gender == Gender::Fem {
        let si = singular_info::singular_info_c(lemma, gender, "", false);
        if si.genitive.first().map(|f| f.value.as_str()) == Some(genitive) {
            return 2;
        }
    }
    // 2nd: SingularInfoE with syncope
    let si = singular_info::singular_info_e(lemma, gender, true, false, "", true);
    if si.genitive.first().map(|f| f.value.as_str()) == Some(genitive) {
        return 2;
    }

    // 1st: SingularInfoC (slenderize) — last, as it's the most greedy
    let si = singular_info::singular_info_c(lemma, gender, "", true);
    if si.genitive.first().map(|f| f.value.as_str()) == Some(genitive) {
        return 1;
    }

    0 // unrecognised
}

/// Validate guessers against Kaikki/Wiktionary entries that have a known genitive.
///
/// Reads TSV: lemma\tgender\tgenitive
/// Uses the genitive to recognise the "true" declension, then compares guessers.
fn validate_kaikki(data_path: &Path, tsv_path: &str) {
    println!("=== Kaikki/Wiktionary Novel Noun Validation ===\n");

    // Build LemmaDb from BuNaMo
    let noun_dir = data_path.join("noun");
    let entries: Vec<_> = fs::read_dir(&noun_dir)
        .expect("Cannot read noun directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();

    let mut db = noun::LemmaDb::new();
    for entry in &entries {
        if let Some(n) = parse_noun_xml(&entry.path()) {
            if n.declension >= 1 && n.declension <= 5 {
                db.insert(n.lemma.clone(), n.declension, n.gender);
            }
        }
    }
    println!("LemmaDb: {} entries from BuNaMo\n", db.len());

    let content = fs::read_to_string(tsv_path).expect("Cannot read Kaikki TSV");

    let mut total = 0;
    let mut recognised = 0;
    let mut unrecognised = 0;
    let mut correct_simple = 0;
    let mut correct_compound = 0;
    let mut gen_nom_equal = 0;
    let mut confusion_simple: HashMap<(i8, i8), usize> = HashMap::new();
    let mut confusion_compound: HashMap<(i8, i8), usize> = HashMap::new();
    let mut unrec_examples: Vec<(String, String, String)> = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 { continue; }

        let lemma = parts[0];
        let gender = match parts[1] {
            "masc" => Gender::Masc,
            "fem" => Gender::Fem,
            _ => continue,
        };
        let genitive = parts[2];

        total += 1;

        if lemma == genitive {
            gen_nom_equal += 1;
        }

        let expected = recognise_declension(lemma, gender, genitive);
        if expected == 0 {
            unrecognised += 1;
            if unrec_examples.len() < 30 {
                unrec_examples.push((lemma.to_string(), genitive.to_string(),
                    if gender == Gender::Masc { "masc" } else { "fem" }.to_string()));
            }
            continue;
        }
        recognised += 1;

        let simple = noun::guess_declension(lemma, gender);
        let compound = noun::guess_declension_compound(lemma, gender, &db);

        if let Some(s) = simple {
            if s.as_i8() == expected {
                correct_simple += 1;
            } else {
                *confusion_simple.entry((expected, s.as_i8())).or_insert(0) += 1;
            }
        }

        if let Some(c) = compound {
            if c.as_i8() == expected {
                correct_compound += 1;
            } else {
                *confusion_compound.entry((expected, c.as_i8())).or_insert(0) += 1;
            }
        }
    }

    println!("Total entries: {}", total);
    println!("  gen = nom: {}", gen_nom_equal);
    println!("  Recognised declension: {}", recognised);
    println!("  Unrecognised (no strategy matched): {}", unrecognised);
    println!(
        "\nSimple guesser:   {}/{} ({:.2}%)",
        correct_simple, recognised, 100.0 * correct_simple as f64 / recognised as f64
    );
    println!(
        "Compound guesser: {}/{} ({:.2}%)",
        correct_compound, recognised, 100.0 * correct_compound as f64 / recognised as f64
    );

    println!("\nSimple confusion (expected→guessed):");
    let mut conf: Vec<_> = confusion_simple.iter().collect();
    conf.sort_by(|a, b| b.1.cmp(a.1));
    for ((exp, got), count) in conf.iter().take(10) {
        println!("  dec{}→dec{}: {}", exp, got, count);
    }

    println!("\nCompound confusion (expected→guessed):");
    let mut conf: Vec<_> = confusion_compound.iter().collect();
    conf.sort_by(|a, b| b.1.cmp(a.1));
    for ((exp, got), count) in conf.iter().take(10) {
        println!("  dec{}→dec{}: {}", exp, got, count);
    }

    if !unrec_examples.is_empty() {
        println!("\nUnrecognised examples (genitive doesn't match any strategy):");
        for (lemma, gen, gender) in &unrec_examples {
            println!("  {} ({}): gen={}", lemma, gender, gen);
        }
    }
}

fn validate_adjective_declension_guessing(data_path: &Path) {
    println!("--- Adjective declension guessing (lemma only) ---");

    let adj_dir = data_path.join("adjective");
    let entries: Vec<_> = fs::read_dir(&adj_dir)
        .expect("Cannot read adjective directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();

    let mut total = 0;
    let mut skipped_dec0 = 0;
    let mut correct = 0;
    let mut confusion: HashMap<(i8, i8), usize> = HashMap::new();
    let mut errors: Vec<(String, i8, i8)> = Vec::new();

    for entry in &entries {
        let xml = match fs::read_to_string(entry.path()) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let lemma = xml.split("default=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap_or("");
        let declension: i8 = xml.split("declension=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if declension == 0 {
            skipped_dec0 += 1;
            continue;
        }

        total += 1;
        let guessed = adjective::guess_declension(lemma);

        if guessed.as_i8() == declension {
            correct += 1;
        } else {
            *confusion.entry((declension, guessed.as_i8())).or_insert(0) += 1;
            if errors.len() < 30 {
                errors.push((lemma.to_string(), declension, guessed.as_i8()));
            }
        }
    }

    println!("Total adjectives with declension 1-3: {}", total);
    println!("Skipped (declension 0): {}", skipped_dec0);
    println!(
        "Guesser: {}/{} ({:.2}%)",
        correct, total, 100.0 * correct as f64 / total as f64
    );

    if !confusion.is_empty() {
        println!("\nConfusion matrix (expected→guessed, count):");
        let mut conf: Vec<_> = confusion.iter().collect();
        conf.sort_by(|a, b| b.1.cmp(a.1));
        for ((exp, got), count) in &conf {
            println!("  dec{}→dec{}: {}", exp, got, count);
        }

        println!("\nMisclassified examples:");
        for (lemma, expected, got) in &errors {
            println!("  {}: expected dec{}, guessed dec{}", lemma, expected, got);
        }
    }
}

fn validate_adjective_form_generation(data_path: &Path) {
    println!("--- Adjective form generation (lemma+declension) ---");

    let adj_dir = data_path.join("adjective");
    let entries: Vec<_> = fs::read_dir(&adj_dir)
        .expect("Cannot read adjective directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "xml"))
        .collect();

    let slots = ["sgGenMasc", "sgGenFem", "plNom", "graded"];
    let mut total = 0;
    let mut skipped = 0;
    let mut slot_total: HashMap<String, usize> = HashMap::new();
    let mut slot_correct: HashMap<String, usize> = HashMap::new();
    let mut slot_errors: HashMap<String, Vec<(String, String, String)>> = HashMap::new();

    for entry in &entries {
        let xml = match fs::read_to_string(entry.path()) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let lemma = xml.split("default=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap_or("");
        let declension: i8 = xml.split("declension=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if declension < 1 || declension > 3 {
            continue;
        }

        let dec = match declension {
            1 => Declension::First,
            2 => Declension::Second,
            3 => Declension::Third,
            _ => continue,
        };

        total += 1;
        let forms = match adjective::generate_forms(lemma, dec) {
            Some(f) => f,
            None => {
                skipped += 1;
                continue;
            }
        };

        for slot in &slots {
            let tag = format!("<{} default=\"", slot);
            let expected = xml.split(&tag)
                .nth(1)
                .and_then(|s| s.split('"').next());

            let expected = match expected {
                Some(e) => e,
                None => continue,
            };

            let generated = match *slot {
                "sgGenMasc" => &forms.sg_gen_masc,
                "sgGenFem" => &forms.sg_gen_fem,
                "plNom" => &forms.pl_nom,
                "graded" => &forms.graded,
                _ => continue,
            };

            *slot_total.entry(slot.to_string()).or_insert(0) += 1;

            if generated == expected {
                *slot_correct.entry(slot.to_string()).or_insert(0) += 1;
            } else {
                let errs = slot_errors.entry(slot.to_string()).or_default();
                if errs.len() < 10 {
                    errs.push((
                        lemma.to_string(),
                        expected.to_string(),
                        generated.to_string(),
                    ));
                }
            }
        }
    }

    println!("Total adjectives with declension 1-3: {}", total);
    if skipped > 0 {
        println!("Skipped (abstained): {}", skipped);
    }

    let mut all_total = 0usize;
    let mut all_correct = 0usize;

    for slot in &slots {
        let st = *slot_total.get(*slot).unwrap_or(&0);
        let sc = *slot_correct.get(*slot).unwrap_or(&0);
        all_total += st;
        all_correct += sc;
        let pct = if st > 0 { 100.0 * sc as f64 / st as f64 } else { 0.0 };
        let err_count = st - sc;
        println!("  {}: {}/{} ({:.2}%) — {} errors", slot, sc, st, pct, err_count);

        if let Some(errs) = slot_errors.get(*slot) {
            for (lemma, expected, got) in errs {
                println!("    {}: expected '{}', got '{}'", lemma, expected, got);
            }
            if err_count > errs.len() {
                println!("    ... and {} more", err_count - errs.len());
            }
        }
    }

    let overall_pct = if all_total > 0 { 100.0 * all_correct as f64 / all_total as f64 } else { 0.0 };
    println!(
        "\nOverall form accuracy: {}/{} ({:.2}%)",
        all_correct, all_total, overall_pct
    );
}
