use ab_radix_trie::Trie;
use serde_json;
use serde_json::json;
fn main() {
    let mut trie: Trie<i32> = Trie::new();
    trie.insert("romanus", None);
    trie.insert("romulus", Some(10));
    trie.insert("rubens", None);
    trie.insert("ruber", None);
    trie.insert("rubicon", None);
    trie.insert("rubicundus", None);

    let x = serde_json::to_value(&trie).unwrap();
    let s = serde_json::to_string_pretty(&x).unwrap();
    println!("{}", s);


    let json = json!({
  "char_count": 26,
  "children": {
    "r": {
      "children": {
        "o": {
          "children": {
            "a": {
              "children": {},
              "terminal": true,
              "text": "anus",
              "value": null,
              "visit_count": 0,
              "weight": 4
            },
            "u": {
              "children": {},
              "terminal": true,
              "text": "ulus",
              "value": 10,
              "visit_count": 0,
              "weight": 4
            }
          },
          "terminal": false,
          "text": "om",
          "value": null,
          "visit_count": 0,
          "weight": 10
        },
        "u": {
          "children": {
            "e": {
              "children": {
                "n": {
                  "children": {},
                  "terminal": true,
                  "text": "ns",
                  "value": null,
                  "visit_count": 0,
                  "weight": 2
                },
                "r": {
                  "children": {},
                  "terminal": true,
                  "text": "r",
                  "value": null,
                  "visit_count": 0,
                  "weight": 1
                }
              },
              "terminal": false,
              "text": "e",
              "value": null,
              "visit_count": 0,
              "weight": 4
            },
            "i": {
              "children": {
                "o": {
                  "children": {},
                  "terminal": true,
                  "text": "on",
                  "value": null,
                  "visit_count": 0,
                  "weight": 2
                },
                "u": {
                  "children": {},
                  "terminal": true,
                  "text": "undus",
                  "value": null,
                  "visit_count": 0,
                  "weight": 5
                }
              },
              "terminal": false,
              "text": "ic",
              "value": null,
              "visit_count": 0,
              "weight": 9
            }
          },
          "terminal": false,
          "text": "ub",
          "value": null,
          "visit_count": 0,
          "weight": 15
        }
      },
      "terminal": false,
      "text": "r",
      "value": null,
      "visit_count": 0,
      "weight": 26
    }
  },
  "node_count": 11
});

    let trie : Trie<i32> = serde_json::from_value(json).unwrap();
    println!("{:?}", trie);
}