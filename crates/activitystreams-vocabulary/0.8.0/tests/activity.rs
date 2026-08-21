use activitystreams_vocabulary::{ActivityVocabulary, Object, VocabularyType, VocabularyTypes};

use external_vocab::{ExternalType, TestActivity};

#[test]
fn test_external_activity() {
    let obj = Object::new().with_kind(ExternalType::TestActivity);
    let test_ext_obj = TestActivity::<ExternalType>::new();
    let test_vocab_obj = TestActivity::<VocabularyTypes>::new();

    assert_eq!(ExternalType::TestActivity.as_str(), "TestActivity");

    let vocab = VocabularyType::from(ExternalType::TestActivity);

    assert_eq!(
        vocab,
        VocabularyType::Iri(ExternalType::TestActivity.as_str().try_into().unwrap())
    );
    assert!(test_ext_obj.kind().contains(vocab.as_str()));
    assert!(test_vocab_obj.kind().contains(vocab));

    let json_str = r#"{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "TestActivity"
}"#;
    assert_eq!(
        serde_json::to_string_pretty(&test_ext_obj).unwrap(),
        json_str
    );
    assert_eq!(
        serde_json::to_string_pretty(&test_vocab_obj).unwrap(),
        json_str
    );

    assert_eq!(
        serde_json::from_str::<TestActivity>(json_str).unwrap(),
        test_vocab_obj
    );
    assert_eq!(
        serde_json::from_str::<TestActivity<ExternalType>>(json_str).unwrap(),
        test_ext_obj
    );

    assert_eq!(serde_json::from_str::<Object>(json_str).unwrap(), obj);

    let json_str = r#"{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object"
}"#;
    assert!(serde_json::from_str::<Object>(json_str).is_ok());
    assert!(serde_json::from_str::<TestActivity>(json_str).is_err());
    assert!(serde_json::from_str::<TestActivity<ExternalType>>(json_str).is_err());
}
