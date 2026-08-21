use activitystreams_vocabulary::{ActivityVocabulary, Object, VocabularyType, VocabularyTypes};

use external_vocab::{ExternalType, TestObject};

#[test]
fn test_external_object() {
    let obj = Object::new().with_kind(ExternalType::TestObject);
    let test_ext_obj = TestObject::<ExternalType>::new();
    let test_vocab_obj = TestObject::<VocabularyTypes>::new();

    assert_eq!(ExternalType::TestObject.as_str(), "TestObject");

    let vocab = VocabularyType::from(ExternalType::TestObject);

    assert_eq!(
        vocab,
        VocabularyType::Iri(ExternalType::TestObject.as_str().try_into().unwrap())
    );
    assert!(test_ext_obj.kind().contains(vocab.as_str()));
    assert!(test_vocab_obj.kind().contains(vocab));

    let json_str = r#"{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "TestObject"
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
        serde_json::from_str::<TestObject>(json_str).unwrap(),
        test_vocab_obj
    );
    assert_eq!(
        serde_json::from_str::<TestObject<ExternalType>>(json_str).unwrap(),
        test_ext_obj
    );

    assert_eq!(serde_json::from_str::<Object>(json_str).unwrap(), obj);

    let json_str = r#"{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object"
}"#;
    assert!(serde_json::from_str::<Object>(json_str).is_ok());
    assert!(serde_json::from_str::<TestObject>(json_str).is_err());
    assert!(serde_json::from_str::<TestObject<ExternalType>>(json_str).is_err());
}
