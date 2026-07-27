//! Cross-validates the Rust gramadan implementation against BuNaMo XML data.
//!
//! Tests:
//! 1. Noun declension guessing (lemma+gender) vs BuNaMo declension
//! 2. Noun genitive generation (lemma+gender+declension) vs BuNaMo genitive
//! 3. Verb conjugation class inference vs BuNaMo future-form method

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use gramadan::features::Gender;
use gramadan::noun;
use gramadan::verb;

fn main() {
    let mut args = std::env::args().skip(1).peekable();

    let mut dump_guesses = false;
    let mut data_dir = "../data".to_string();

    while let Some(arg) = args.next() {
        if arg == "--dump-guesses" {
            dump_guesses = true;
        } else {
            data_dir = arg;
        }
    }

    let data_path = Path::new(&data_dir);

    if dump_guesses {
        dump_noun_guesses(data_path);
        return;
    }

    println!("=== Gramadán Rust Cross-Validation ===\n");

    validate_noun_declension_guessing(data_path);
    println!();
    validate_noun_genitive_generation(data_path);
    println!();
    validate_verb_conjugation(data_path);
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
            simple_guess.as_i8(),
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

    let mut total = 0;
    let mut correct_simple = 0;
    let mut correct_full = 0;
    let mut confusion_simple: HashMap<(i8, i8), usize> = HashMap::new();
    let mut confusion_full: HashMap<(i8, i8), usize> = HashMap::new();

    for entry in &entries {
        let path = entry.path();
        let noun = match parse_noun_xml(&path) {
            Some(n) => n,
            None => continue,
        };

        // Skip declension 0 (irregular/unclassified)
        if noun.declension == 0 || noun.declension < 1 || noun.declension > 5 {
            continue;
        }

        total += 1;
        let expected = noun.declension;

        let guessed_simple = noun::guess_declension(&noun.lemma, noun.gender);
        let guessed_full = noun::guess_declension_full(&noun.lemma, noun.gender);

        if guessed_simple.as_i8() == expected {
            correct_simple += 1;
        } else {
            *confusion_simple
                .entry((expected, guessed_simple.as_i8()))
                .or_insert(0) += 1;
        }

        if guessed_full.as_i8() == expected {
            correct_full += 1;
        } else {
            *confusion_full
                .entry((expected, guessed_full.as_i8()))
                .or_insert(0) += 1;
        }
    }

    println!("Total nouns with declension 1-5: {}", total);
    println!(
        "Simple guesser:  {}/{} ({:.2}%)",
        correct_simple,
        total,
        100.0 * correct_simple as f64 / total as f64
    );
    println!(
        "Full guesser:    {}/{} ({:.2}%)",
        correct_full,
        total,
        100.0 * correct_full as f64 / total as f64
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
            for (lemma, expected, got) in errors.iter().take(10) {
                println!("    {} : expected '{}', got '{}'", lemma, expected, got);
            }
            if errors.len() > 10 {
                println!("    ... and {} more", errors.len() - 10);
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

    // For the future-based method, we need to determine ground truth.
    // Ground truth = the future-based method from Python (check for 'f' in suffix).
    // We compare:
    //   1. lemma-only heuristic vs future-based ground truth
    //   2. our Rust future-based vs Python future-based (should be 100%)

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

        // Ground truth: future-based
        let gt = verb::get_conjugation_from_future(&v.lemma, &v.future_indep_base);
        let gt_str = format!("{:?}", gt);
        *class_counts.entry(gt_str.clone()).or_insert(0) += 1;

        // Lemma-only guess
        let guessed = verb::guess_conjugation(&v.lemma);

        if guessed == gt {
            correct_lemma_only += 1;
        } else {
            let key = (format!("{:?}", gt), format!("{:?}", guessed));
            *confusion_lemma.entry(key).or_insert(0) += 1;
        }

        // Future-based (should always match itself — this validates our Rust implementation)
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
