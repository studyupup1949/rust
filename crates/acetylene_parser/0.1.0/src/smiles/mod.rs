use regex::{Regex, RegexSet};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use types::Substance;

#[derive(Debug)]
struct SmilesDict {
  pub typ: String,
  pub val: String,
  pub chg: Option<i32> // charge
}

struct ClosureDict {
  pub pos: usize,
  pub rank: usize
}

/// Function that computes whether or not a given vector of ring closures has
/// closed all of the open ring closure numbers.
///
/// Basically this is delimiter counting, where the delimiters are single
/// numbers. If the count of each is even, we know we've closed all of the open
/// ones.
fn closures_closed(closure_pos_queue: &Vec<ClosureDict>) -> bool {
  let ranks: HashSet<usize> = closure_pos_queue.iter().map(|dict| {
    dict.rank
  }).collect();

  ranks.iter().map(|&this_rank| {
    closure_pos_queue.iter().filter(|&dict| {
      dict.rank == this_rank
    }).count()
  }).all(|rank_count| { rank_count % 2 == 0 })
}

/// Function for turning "+++" and "---" or "+3" and so on into a signed int.
fn count_charges(chg: &str) -> i32 {
  if chg.starts_with("+") { // positive charge case
    if chg.ends_with("+") { // case of a string of pluses
      chg.len() as i32
    } else { // case of plus then integer
      i32::from_str(&chg[1..chg.len()]).unwrap()
    }
  } else if chg.starts_with("-") { // negative charge case
    if chg.ends_with("-") { // case of a string of minuses
      chg.len() as i32 * (-1)
    } else { // case of minus then integer
      i32::from_str(&chg[1..chg.len()]).unwrap() * (-1)
    }
  } else {
    panic!("Tried to count charges in a bad string: {:?}", chg);
  }
}

fn tokenize_smiles_raw(input: &str) -> Vec<SmilesDict> {
  let atom_fields: Vec<&'static str> = vec![
    "raw_atom",
    "open_bracket",
    "weight",
    "element",
    "chiral_count",
    "chiral_named",
    "chiral_symbols",
    "hcount",
    "positive_count",
    "positive_symbols",
    "negative_count",
    "negative_symbols",
    "error_1",
    "error_2",
    "close_bracket",
    "error_3"
  ];
  let bond_fields: Vec<&'static str> = vec!["bond"];
  let dot_fields: Vec<&'static str> = vec!["dot"];
  let closure_fields: Vec<&'static str> = vec!["closure"];
  let close_branch_fields: Vec<&'static str> = vec!["close_branch"];
  let open_branch_fields: Vec<&'static str> = vec!["open_branch"];
  
  lazy_static! {
    static ref ATOM_RE_EXP: String = String::from(r#"(?x)
      (?P<raw_atom>Cl|Br|[cnospBCNOFPSI])|   # 'raw' means outside of brackets
                                             # SMILES is 'ground state' only,
                                             # so hydrogen can't appear outside
                                             # of brackets, 'on its own'.
      (
        (?P<open_bracket>\[)                 # Start bracket
        (?P<weight>\d+)?                     # Atomic weight (optional)
        (                                    # valid term or error
         (                                   # valid term    
          (?P<element>                       # element or aromatic
             C[laroudsemf]?|
             Os?|
             N[eaibdpos]?|
             S[icernbmg]?|
             P[drmtboau]?|
             H[eofgas]?|
             c|n|o|s|p|
             A[lrsgutcm]|
             B[eraik]?|
             Dy|E[urs]|
             F[erm]?|
             G[aed]|
             I[nr]?|
             Kr?|
             L[iaur]|
             M[gnodt]|
             R[buhenaf]|
             T[icebmalh]|
             U|V|W|Xe|Yb?|Z[nr]|\*"
          )                                  
          (                                  # Chirality can be
           (?P<chiral_count>@\d+)|           #   @1 @2 @3 ...    
           (?P<chiral_named>                 # or
            @TH[12]|                         #   @TA1 @TA2
            @AL[12]|                         #   @AL1 @AL2
            @SP[123]|                        #   @SP1 @SP2 @SP3    
            @TB(1[0-9]?|20?|[3-9])|          #   @TB{1-20}    
            @OH(1[0-9]?|2[0-9]?|30?|[4-9]))| #   @OH{1-30}    
            (?P<chiral_symbols>@+)           # or @@@@....,
           )?                                # and chirality is optional    
          (?P<hcount>H\d*)?                  # Optional hydrogen count    
          (?P<charge>                        # Charges can be    
           (?P<positive_count>\+\d+)|        #   +<number>    
           (?P<positive_symbols>\++)|        #   +++...including a single '+'    
           (?P<negative_count>-\d+)|         #   -<number>    
           (?P<negative_symbols>-+)          #   ---... including a single '-'
          )?                                 # and are optional    
         (?P<error_1>[^\]]+)?                # anything left is an error    
        )                                    # End of stuff in []s, except    
        |((?P<error_2>[^\]]*)                # If other error, we get here
      ))
      ((?P<close_bracket>\])|                # End bracket    
      (?P<error_3>$)))"#);                   // unexpected end of string
    
    static ref BOND_RE_EXP: String = String::from(r"(?P<bond>[=#/\\:~-])");
    static ref DOT_RE_EXP: String = String::from(r"(?P<dot>\.)");
    static ref CLOSURE_RE_EXP: String = String::from(r"(?P<closure>\d|%\d\d?)");
    static ref CLOSE_BRANCH_RE_EXP: String = String::from(r"(?P<close_branch>\))");
    static ref OPEN_BRANCH_RE_EXP: String = String::from(r"(?P<open_branch>\()");

    static ref REGEX_LIST: Vec<Regex> = vec![
      Regex::new(&ATOM_RE_EXP).unwrap(),
      Regex::new(&BOND_RE_EXP).unwrap(),
      Regex::new(&DOT_RE_EXP).unwrap(),
      Regex::new(&CLOSURE_RE_EXP).unwrap(),
      Regex::new(&CLOSE_BRANCH_RE_EXP).unwrap(),
      Regex::new(&OPEN_BRANCH_RE_EXP).unwrap()
    ];
  }
  

  let start_str_list: Vec<&str> = vec![&ATOM_RE_EXP];
  let atom_str_list: Vec<&str> = vec![
    &ATOM_RE_EXP,
    &BOND_RE_EXP,
    &DOT_RE_EXP,
    &CLOSURE_RE_EXP,
    &CLOSE_BRANCH_RE_EXP,
    &OPEN_BRANCH_RE_EXP
  ];
  let bond_str_list: Vec<&str> = vec![
    &ATOM_RE_EXP,
    &CLOSURE_RE_EXP
  ];
  let close_branch_str_list: Vec<&str> = vec![
    &ATOM_RE_EXP,
    &BOND_RE_EXP,
    &DOT_RE_EXP,
    &CLOSE_BRANCH_RE_EXP,
    &OPEN_BRANCH_RE_EXP
  ];
  let open_branch_str_list: Vec<&str> = vec![
    &ATOM_RE_EXP,
    &BOND_RE_EXP,
    &DOT_RE_EXP
  ];
  let dot_str_list: Vec<&str> = vec![
    &ATOM_RE_EXP
  ];
  let closure_str_list: Vec<&str> = atom_str_list.clone(); // same as atom  

  let regex_list_names: Vec<&str> = vec![
    "atom",
    "bond",
    "dot",
    "closure",
    "close_branch",
    "open_branch"
  ];

  let mut allowed_states: HashMap<&str, Vec<&str>> = HashMap::new();
  allowed_states.insert("start", vec!["atom"]);
  // CC, C=C, C(C)C, C(C)C, C.C, C1CCC1
  allowed_states.insert("atom", vec!["atom", "bond", "close_branch",
                                     "open_branch", "dot", "closure"]);
  // C=C, C=1CCC=1
  allowed_states.insert("bond", vec!["atom", "closure"]);
  // C(C)C, C(C)=C, C(C).C, C(C(C))C, C(C)(C)C
  allowed_states.insert("close_branch", vec!["atom", "bond", "dot",
                                             "close_branch", "open_branch"]);
  // C(C), C(=C), C(.C) (really!)
  allowed_states.insert("open_branch", vec!["atom", "bond", "dot"]);
  // C.C  -- allow a dot? as in C..C  
  allowed_states.insert("dot", vec!["atom"]);
  // C1CCC1, C1=CCC1, C12CC1C2, C1C(CC)1, C1(CC)CC1, c1ccccc1.[NH4+]
  allowed_states.insert("closure", vec!["atom", "bond", "closure",
                                        "close_branch", "open_branch", "dot"]);

  // 1. Create a RegexSet from each `allowed_states` table
  let mut state_regex_dict: HashMap<&str, RegexSet> = HashMap::new();
  state_regex_dict.insert("start", RegexSet::new(&start_str_list).unwrap());
  state_regex_dict.insert("atom", RegexSet::new(&atom_str_list).unwrap());
  state_regex_dict.insert("bond", RegexSet::new(&bond_str_list).unwrap());
  state_regex_dict.insert("close_branch", RegexSet::new(&close_branch_str_list).unwrap());
  state_regex_dict.insert("open_branch", RegexSet::new(&open_branch_str_list).unwrap());
  state_regex_dict.insert("dot", RegexSet::new(&dot_str_list).unwrap());
  state_regex_dict.insert("closure", RegexSet::new(&closure_str_list).unwrap());
  

  let mut fields_dict: HashMap<&str, &Vec<&str>> = HashMap::new();
  fields_dict.insert("atom", &atom_fields);
  fields_dict.insert("bond", &bond_fields);
  fields_dict.insert("close_branch", &close_branch_fields);
  fields_dict.insert("open_branch", &open_branch_fields);
  fields_dict.insert("dot", &dot_fields);
  fields_dict.insert("closure", &closure_fields);

  // New algorithm (comments duplicated where they happen):
  // 1. Create a RegexSet from each `allowed_states` table (see above)
  // 2. Initialize a character index to zero
  // 3. Enter start state
  // 4. Run the current state RegexSet on the **whole string**
  // 5. Run the matched Regexes again on the **whole string**
  // 6. Push the earliest (position) group of captured tokens onto the result
  // Vec
  // 7. Set the character index to the end of the first group
  // 8. Run the new state RegexSet on the **substring sliced from the current
  //    character index**


  // 2. Initialize a character index to zero
  // The char index will be incremented later in the loop.
  let mut cur_char_ind = 0;
  
  let mut expected_states: &Vec<&str> = allowed_states.get("start").unwrap();
  let mut cur_regex: &Regex = &REGEX_LIST[0];
  let mut tokens: Vec<SmilesDict> = vec![];
  
  // 3. Enter start state
  let mut found_state: &str = "start";

  // This first loop checks to see which regexes match, and then narrows down
  // the fields we're interested in for when we use the matched regex below.
  loop {
    // 4. Run the current state RegexSet on the **whole string**
    // 8. Run the new state RegexSet on the **substring sliced from the current
    //    character index**
    let cur_test_string = String::from(&input[cur_char_ind..input.len()]);
        
    let cur_regex_set = state_regex_dict.get(found_state).unwrap();

    // RegexSet returns indices only...
    let matched_regex_inds: Vec<usize> = cur_regex_set
      .matches(cur_test_string.as_str())
      .into_iter()
      .collect();

    // ...so we convert them to string names here.
    let matched_regexes: Vec<Option<&&str>> = matched_regex_inds
      .iter()
      .map(|&ind| {
        regex_list_names.get(ind)
      })
      .collect();

    // compute the intersection of the expected states and the matched regex
    // names for safety's sake
    let expected: HashSet<_> = expected_states.iter().cloned().collect();
    let matched: HashSet<_> = matched_regexes.iter().cloned()
      .map(|item| { *item.unwrap() }).collect();
    let ok_states: Vec<_> = matched.intersection(&expected).cloned().collect();

    if ok_states.is_empty() {
      panic!(format!("Got myself into a bad state. Expected {:?} \n\
                      but got {:?}. \n\
                      I was trying to match on: {}",
                     expected, matched, &cur_test_string));
      
    }

    let captures: Vec<((usize, usize), &str, &str, Option<i32>)> = ok_states.iter()
      .map(|&state_name| {
        cur_regex = &REGEX_LIST[regex_list_names.iter().position(|&name| {
          name == state_name
        }).unwrap()];

        // 5. Run the matched Regexes again on the **whole string**.
        // We have to redo the regex match with the specific regex that matched,
        // since RegexSet doesn't give info about captures for efficiency
        // reasons.
        let maybe_captures = cur_regex.captures(cur_test_string.as_str());
        if let Some(cur_captures) = maybe_captures {
          // Check to see if there is a charge
          let mut charge = None;
          if state_name == "atom" {
            charge = cur_captures.name("charge").and_then(|charge| {
              Some(count_charges(charge))
            });
          }

          // we can unwrap since 0th capture is guaranteed among any captures
          Some((cur_captures.pos(0).unwrap(),
                cur_captures.at(0).unwrap(),
                state_name,
                charge))
        } else {
          None
        }
      })
      .filter(|item| {  item.is_some() })
      .map(|ok_item| { ok_item.unwrap() })
      .collect();

    let earliest_capture = captures.iter().by_ref().min_by_key(|cap_tuple| {
      (cap_tuple.0).0
    }).unwrap();
    
    // 6. Push the first group of captured tokens onto the result Vec
    tokens.push(SmilesDict {
      typ: String::from(earliest_capture.2),
      val: String::from(earliest_capture.1),
      chg: earliest_capture.3
    });

    // NOTE: Kind of a hack--if the whole substance is the same as the one we
    // just computed the charge for, we can use it when we add the "whole
    // substance" to the tokens.
    let mut sub_charge = None;
    if &earliest_capture.1.len() == &input.len(){
      // is length comparison faster than value comparison?
      sub_charge = earliest_capture.3;
    }

    // Get the new list of expected states
    found_state = earliest_capture.2;
    expected_states = allowed_states.get(found_state).unwrap();

    // 7. Set the character index to the end of the first group
    cur_char_ind = (earliest_capture.0).1 + cur_char_ind;

    // Drop out if we got the whole string
    if cur_char_ind == input.len() {
      // Put the original "whole substance" at the beginning

      tokens.insert(0, SmilesDict {
        typ: String::from("atom"),
        val: String::from(input),
        chg: sub_charge // charge hack from above
      });
      return tokens;
    }    
  }
}

// fn get_charge(input: SmilesDict) -> usize {
//   
// }

/// spec: http://www.daylight.com/dayhtml_tutorials/languages/smiles/index.html
fn vec_to_sub(input: Vec<SmilesDict>) -> Substance {
  // Algorithm is simple for now--if the type is an "atom", regex parse it for
  // charge and quantity, then create a Substance.
  
  let mut sub = Substance {
    symbol: String::from(input[0].val.to_owned()),
    quantity: 1,
    charge: input[0].chg,
    groups: None
  };

  if input.len() > 1 {
    // This is a vector of closure positions and numbers.
    let mut closure_pos_queue: Vec<ClosureDict> = vec![];
    
    let mut group_vec: Vec<Option<Substance>> = vec![];
    for (ind, cur_smile) in input.iter().skip(1).enumerate() {
      // We skip the first one since we know that's the whole input, and we've
      // taken care of it above.
      if cur_smile.typ == "atom" {
        group_vec.push(Some(Substance {
          symbol: String::from(cur_smile.val.as_str()),
          quantity: 1,
          charge: cur_smile.chg,
          groups: None
        }));
      } else if cur_smile.typ == "closure" {
        if closure_pos_queue.len() > 0 {
          // If we find a ring closure of the right rank (position in our
          // closure_pos_queue vector), we should take the previous atom and
          // concatenate it together (meaning "close the ring") with all of the
          // atoms up to the next closure.
          if closures_closed(&closure_pos_queue) {
            let first_pos = closure_pos_queue.iter()
              .min_by_key(|&dict| {
                dict.pos
              }).unwrap().pos - 1; // - 1 to get the previous atom
            // since we're guaranteed to be at the tip of the groups vector
            let last_pos = group_vec.len();

            // Why does this need an explicit let anyway?
            let defined_up_there = group_vec[first_pos].clone().unwrap().symbol;
            
            let concatenated_closure = Substance {
              symbol: group_vec
                .drain(first_pos - 1..last_pos)
                .fold(defined_up_there,
                      |acc_symbol, next_item| {
                        if let Some(ok_sub) = next_item {
                          acc_symbol.to_owned().push_str(ok_sub.symbol.as_str());
                          acc_symbol
                        } else {
                          acc_symbol
                        }
                      }),
              quantity: 1, 
              charge: Some(0), // TODO is a ring always neutrally charged? no way
              groups: None // TODO this too--we shouldn't recurse down to
                           // individual atoms here
            };
            
            // once we've closed the ring, we drop the remaining extra
            // substances that were inside the closures.
            group_vec.push(Some(concatenated_closure));
          } else {
            // even if the closures aren't all closed, we need to add the
            // a placeholder for the closure item so that the indexes line up
            group_vec.push(None);
          }
        } else {
          // So if we haven't seen this closure number before (closing the
          // ring), we should record where we found it.
          closure_pos_queue.push(ClosureDict {
            pos: ind,
            rank: cur_smile.val.parse().unwrap()
          });
        }
      }
    }
    sub.groups = Some(Box::new(group_vec.iter()
      .filter(|sub| {
        sub.is_some()
      })
      .map(|ok_sub| { ok_sub.clone().unwrap() })
      .collect()));
  }
  sub
}

pub fn tokenize_smiles(input: &str) -> Substance {
  vec_to_sub(tokenize_smiles_raw(input))
}

mod test {
  use smiles::tokenize_smiles_raw;

  // TODO change these tests to assertions. visual spot checks cheer you up, but
  // still  
  #[test]
  fn benzene_closure() {
    let test_str = r"c1ccccc1";
    let res = tokenize_smiles_raw(test_str);
    println!("Tokens: {:?}", res);
  }

  #[test]
  fn smile_raw_hydronium() {
    println!("{:?}", tokenize_smiles_raw(r"[OH3+]"));
  }

  // Honestly right now I just want to make sure it doesn't panic.
  #[test]
  fn smile_at_all_the_things() {
    let test_strs = vec![
      r"CC",
      r"[OH3+]",
      r"O=C=O",
      r"[2H]O[2H]",
      r"C#N",
      r"[235U]",
      r"CCN(CC)CC",
      r"F/C=C/F",
      r"CC(=O)O",
      r"F/C=C\F",
      r"C1CCCCC1",
      r"N[C@@H](C)C(=O)O",
      r"c1ccccc1",
      r"N[C@H](C)C(=O)O",
      r"C",
      r"P",
      r"N",
      r"S",
      r"O",
      r"Cl",
      r"[S]",
      r"[Au]",
      r"[Fe+++]",
      r"[Fe+3]",
      r"[H+]",
      r"[Fe+2]",
      r"[OH-]",
      r"[Fe++]",
      r"[OH3+]",
      r"[NH4+]",
      r"CC",
      r"C=O",
      r"C=C",
      r"O=C=O",
      r"COC",
      r"C#N",
      r"CCO",
      r"[H][H]",
      r"C=CCC=CCO",
      r"C=C-C-C=C-C-O",
      r"OCC=CCC=C",
      r"CCN(CC)CC",
      r"CC(C)C(=O)O",
      r"[O-]N(=O)C=CC(CCC)C(C(C)C)CCC",
      r"[O-]N(=O)c1ccccc1",
      r"c1ccccc1(N(=O)[O-])",
      r"C1CCCCC1",
      r"CC1=CC(Br)CCC1",
      r"CC1=CC(CCC1)Br",
      r"C12C3C4C1C5C4C3C25",
      r"O1CCCCC1N1CCCCC1",
      r"[Na+].[O-]c1ccccc1",
      r"c1cc([O-].[Na+])ccc1",
      r"C1.C1",
      r"[12C]",
      r"[13C]",
      r"[C]",
      r"[13CH4]",
      r"F/C=C/C=C/C",
      r"F/C=C/C=CC",
      r"NC(C)(F)C(=O)O",
      r"NC(F)(C)C(=O)O",
      r"N[C@@H](C(=O)O)C",
      r"N[C@@H](C)C(=O)O",
      r"N[C@@](F)(C)C(=O)O",
      r"N[C@@]([H])(C)C(=O)O",
      r"N[C@H](C(=O)O)C",
      r"N[C@H](C)C(=O)O",
      r"N[C@](C)(F)C(=O)O",
      r"N[C@]([H])(C)C(=O)O",
      r"[C@@H](N)(C)C(=O)O",
      r"[C@H](N)(C)C(=O)O",
      r"[H][C@@](N)(C)C(=O)O",
      r"[H][C@](N)(C)C(=O)O",
      r"OC(Cl)=[C@]=C(C)F",
      r"OC=[C@]=CF",
      r"OC(Cl)=[C@AL1]=C(C)F",
      r"OC([H])=[C@AL1]=C([H])F",
      r"F[Po@SP1](Cl)(Br)I",
      r"F[Po@SP2](Br)(Cl)I",
      r"F[Po@SP3](Cl)(I)Br",
      r"S[As@@](F)(Cl)(Br)C=O",
      r"O=C[As@](F)(Cl)(Br)S",
      r"S[Co@@](F)(Cl)(Br)(I)C=O",
      r"O=C[Co@](F)(Cl)(Br)(I)S",
      r"[2H]O[2H]",
      r"[H][H]",
      r"c1ccc1",
      r"c1ccccc1",
      r"n1ccccc1",
      r"O=n1ccccc1",
      r"[O-][N+]c1ccccc1",
      r"Cn1cccc1",
      r"[nH]1cccc1",
      r"C=[N+]=[N-]",
      r"C[N+](=O)[O-]",
      r"CN(=O)=O",
      r"C=[N]=[N]",
      r"O=c1[nH]cccc1",
      r"Oc1ncccc1"
    ];
    for test_str in test_strs {
      println!("Testing: {}", &test_str);
      let res = tokenize_smiles_raw(test_str);
      println!("Tokens: {:?}", res);
    }
  }
}
