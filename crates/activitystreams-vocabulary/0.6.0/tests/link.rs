use activitystreams_vocabulary::{ActivityVocabulary, Iri, VocabularyType, VocabularyTypes};

use external_vocab::{ExternalType, TestLink};

#[test]
fn test_external_link() {
    let href = Iri::try_from("https://example.dev").unwrap();
    let test_ext_obj = TestLink::<ExternalType>::new().with_href(href.clone());
    let test_vocab_obj = TestLink::<VocabularyTypes>::new().with_href(href.clone());

    assert_eq!(ExternalType::TestLink.as_str(), "TestLink");

    let vocab = VocabularyType::from(ExternalType::TestLink);

    assert_eq!(
        vocab,
        VocabularyType::Iri(ExternalType::TestLink.as_str().try_into().unwrap())
    );
    assert!(test_ext_obj.kind().contains(vocab.as_str()));
    assert!(test_vocab_obj.kind().contains(vocab));

    let json_str = format!(
        r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "TestLink",
  "href": "{href}"
}}"#
    );
    assert_eq!(
        serde_json::to_string_pretty(&test_ext_obj).unwrap(),
        json_str
    );
    assert_eq!(
        serde_json::to_string_pretty(&test_vocab_obj).unwrap(),
        json_str
    );

    assert_eq!(
        serde_json::from_str::<TestLink>(&json_str).unwrap(),
        test_vocab_obj
    );
    assert_eq!(
        serde_json::from_str::<TestLink<ExternalType>>(&json_str).unwrap(),
        test_ext_obj
    );

    let json_str = r#"{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object"
}"#;
    assert!(serde_json::from_str::<TestLink>(json_str).is_err());
    assert!(serde_json::from_str::<TestLink<ExternalType>>(json_str).is_err());
}
