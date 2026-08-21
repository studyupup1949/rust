use regex::{Regex, Captures};
use types::Substance;

fn get_named_capture_index(re: &Regex, cap_name: &str) -> Option<usize> {
  re.capture_names().position(|name| {
    name == Some(cap_name)
  })
}
fn get_named_capture_position(re: &Regex, caps: &Captures, cap_name: &str) ->
  Option<(usize, usize)> {
    let cap_ind = get_named_capture_index(re, cap_name);
    if let Some(cap_ind) = cap_ind {
      caps.pos(cap_ind)
    } else {
      None
    }
  }

/// Tokenizes a informal "school textbook" chemical formula into individual
/// substances.
pub fn tokenize_formula(input: &str) -> Substance {
  lazy_static! {
    static ref FORMULA_RE_EXP: String = String::from(r#"(?x)
      (?P<compound>
        (?P<element> 
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
        )|
        (?P<functional_group>
          \((?:[^)]*(?:
            ( 
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
            )+
          )?
        )+
        (?:[^(])*\))+)
        (?P<quantity>\d+)?
        (?P<charge>
          (?P<negative_literal>-{1,3}[^+])|
          (?P<positive_literal>\+{1,3}[^-])|
          (?P<charge_numeric>[-,\+]\d+
        )
      )*"#);

    static ref RE: Regex = Regex::new(&FORMULA_RE_EXP).unwrap();
  }

  let mut cur_char_ind = 0;
  let whole_sub_caps = RE.captures(&input).unwrap();

  // HACK
  // A quantity parsing hack. I think it would be better to recurse on
  // functional groups, but that's another redo of this section.
  let mut sub_qty = 1;
  if whole_sub_caps[0].len() == input.len() {
    sub_qty = whole_sub_caps.name("quantity")
      .and_then(|qty_str| {
        Some(qty_str.parse().unwrap_or(0))
      }).unwrap_or(1)
  }
  
  // incomplete substance. this is type sacrilege but I don't want to drag
  // unnecessary Options into the Substance defintion.
  let mut cur_sub = Substance {
    symbol: input.to_string(),
    quantity: sub_qty,
    charge: whole_sub_caps.name("charge")
      .and_then(|chg_str| {
        Some(chg_str.parse().unwrap_or(0))
      }),
    groups: None
  };
  // temporary storage for the functional groups vector before we box it up.
  let mut groups_vec = vec![];

  loop {
    let cur_test_string = String::from(&input[cur_char_ind..input.len()]);
    
    if let Some(caps) = RE.captures(&cur_test_string) {
      let compound_cap = &caps.name("compound");
      let func_cap = &caps.name("functional_group");

      if func_cap.is_some() || compound_cap.is_some() {
        let compound_str = func_cap.unwrap_or(compound_cap.unwrap());
        let quantity = caps.name("quantity");
        let charge = caps.name("charge");

        let new_sub = Substance {
          symbol: compound_str.to_string(),
          quantity: quantity
            .and_then(|qty_str| {
              Some(qty_str.parse().unwrap_or(1))
            }).unwrap_or(1),
          charge: charge
            .and_then(|chg_str| {
              Some(chg_str.parse().unwrap_or(0))
            }),
          groups: None // NOTE no recursion into smaller functional groups.
        };

        
        groups_vec.push(new_sub);

        if quantity.is_some() && !func_cap.is_some() {
          cur_char_ind += quantity.unwrap().len();
        }
        if charge.is_some() {
          cur_char_ind += charge.unwrap().len();
        }

        if func_cap.is_some() {
          // for functional groups, we want to parse the inside, not skip over
          // completely.
          cur_char_ind += 1;

        } else {
          cur_char_ind += get_named_capture_position(&RE, &caps, "compound")
            .unwrap().1;
        }
      }
    } else {
      cur_char_ind += 1;
    }
    
    if cur_char_ind >= input.len() {
      break;
    }
  }

  if groups_vec.len() > 0 {
    cur_sub.groups = Some(Box::new(groups_vec));
  }

  cur_sub
}


////// Private function tests
#[test]
fn tokenize_formula_single() {
  let sub = Substance {
    symbol: "H".to_string(),
    quantity: 1,
    charge: None,
    groups: None
  };
  assert_eq!(tokenize_formula("H"),
             sub);
}
