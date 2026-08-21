#![cfg(test)]

extern crate acon;
use acon::{Acon, AconError};

fn key_eq(input: &str, key: &str, string: &str) -> Acon {
	let acon = input.parse::<Acon>().unwrap();
	assert_eq!(acon.path(key).unwrap().string(), string);
	acon
}

fn key_eqt(acon: &Acon, key: &str, string: &str) {
	assert_eq!(acon.path(key).unwrap().string(), string);
}

#[test]
fn neg_duplicate_keys() {
	let value = r#"
		key value1
		key2 value2
		key value3
		key2 value4
	"#;
	let acon = value.parse::<Acon>();
	assert_eq!(acon, Err(AconError::OverwritingKey(Some(4))));
}

#[test]
fn neg_duplicate_keys_table() {
	let value = r#"
		key value1
		key2 value2
		{ key
		}
		key2 value4
	"#;
	let acon = value.parse::<Acon>();
	assert_eq!(acon, Err(AconError::OverwritingKey(Some(5))));
}

#[test]
fn neg_duplicate_keys_array() {
	let value = r#"
		key value1
		key2 value2
		[ key
		]
		key2 value4
	"#;
	let acon = value.parse::<Acon>();
	assert_eq!(acon, Err(AconError::OverwritingKey(Some(5))));
}

#[test]
fn neg_duplicate_keys_nested() {
	let value = r#"
		{ key
			{ key
				key value
				[
				]
				key value
			}
		}
	"#;
	let acon = value.parse::<Acon>();
	assert_eq!(acon, Err(AconError::OverwritingKey(Some(7))));
}

#[test]
fn inspect_message() {
	let value = r#"
		[
			{ message
				recipient me
				sender you
				[ content
					Hey what is this ACON thingy all about?
					I mean, we've got TOML, JSON, XML, and SGML.
					Why do we need this data serilization language?
				]
			}
			{ message
				sender me
				recipient you
				[ content
					ACON means Awk-Compatible Object Notation.
					TOML, JSON, etc are great serialization languages, but they're quite complex.
					We need tools and languages that are easily
					parsable and friendly for bash scripting.
					ACON allows just that!
				]
			}
		]
	"#;
	let acon = value.parse::<Acon>().unwrap();
	assert_eq!(acon.get("").unwrap().array().get(1).unwrap().table()
	           .get("message").unwrap().table().get("recipient").unwrap().string(), "you");
	assert_eq!(acon.path(".1.message.recipient"), Some(&Acon::String("you".to_string())));
}

#[test]
fn inspect_dollar_closing() {
	let value = r#"
	{ table
		{ table
			{ table
				[ array
					{ table
						key value

	$ This word as the first word on a line closes all nestings

	[ reason
		I want to get rid of it all.
		If a program crashes whilst serializing (like a script that
		gets an error). Then another program can append $ to the
		end of the stream, clearing that stream.
	]
	"#;
	key_eq(value, "table.table.table.array.0.table.key", "value");
}

#[test]
fn dollar_closing_array_whitespace() {
	let value = r#"
	[ array



	$
	"#;
	let acon = value.parse::<Acon>().unwrap();
	assert_eq!(acon.path("array.2"), Some(&Acon::String("".to_string())));
}

#[test]
fn dollar_duplicate() {
	let value = r#"
	{ table
		key value

	$
	{ table

	$
	"#;
	let acon = value.parse::<Acon>();
	assert_eq!(acon, Err(AconError::OverwritingKey(Some(8))));
}

#[test]
fn neg_ending_array() {
	let value = r#"
	[ array
		value

	"#;
	let acon = value.parse::<Acon>();
	assert_eq!(acon, Err(AconError::TopNodeIsArray));
}

#[test]
fn neg_ending_table() {
	let value = r#"
	{ table
		key value

	"#;
	let acon = value.parse::<Acon>();
	assert_eq!(acon, Err(AconError::MultipleTopNodes));
}

#[test]
fn unnamed_table() {
	let value = r#"
	{
		key value
	}
	"#;
	key_eq(value, ".key", "value");
}

#[test]
fn unnamed_table_2() {
	let value = r#"
	{ named
		key value
	}
	"#;
	key_eq(value, "named.key", "value");
}

#[test]
fn unnamed_array() {
	let value = r#"
	[
		[
			[
				0
	$
	"#;
	key_eq(value, ".0.0.0", "0");
}

#[test]
fn unnamed_array_2() {
	let value = r#"
	[
		[
			[ name
				0
	$
	"#;
	key_eq(value, ".0.0.name.0", "0");
}

#[test]
fn unnamed_elements() {
	let value = r#"
		{ a
			{
				b c
			}
		}
	"#;
	key_eq(value, "a..b", "c");
}

#[test]
fn similarity_acon() {
	let value = r#"
		{ menu
			id file
			value File
			{ popup
				[ menuitem
					{
						value New
						onclick CreateNewDoc()
					}
					{
						value Open
						onclick OpenDoc()
					}
					{
						value Close
						onclick CloseDoc()
					}
				]
			}
		}
	"#;
	key_eq(value, "menu.popup.menuitem.2.value", "Close");
}

#[test]
fn dot_separation() {
	let value = r#"
		{
			{
				lorem ipsum
			}
		}
	"#;
	key_eq(value, "..lorem", "ipsum");
}

#[test]
fn dot_separation_in_array() {
	let value = r#"
		[
			{
				lorem ipsum
			}
		]
	"#;
	key_eq(value, ".0.lorem", "ipsum");
}

#[test]
fn dot_separation_in_array_named_table() {
	let value = r#"
		[
			{ dolor
				lorem ipsum
			}
		]
	"#;
	key_eq(value, ".0.dolor.lorem", "ipsum");
}

#[test]
fn attempt_edges() {
	let value = r#"
		lorem ipsum
		{ dolor
			sit amet
		}
		[ deleniti
		placeat quia
		]
		[
			[
				{ ipsam
					beatae vel
					Iusto enim
				}
			]
			[
				{
					aut quidem
					Sit vitae
				}
			]
		]
	"#;
	let acon = key_eq(value, ".1.0.Sit", "vitae");
	key_eqt(&acon, "deleniti.0", "placeat quia");
}

#[test]
fn named_array_in_unnamed_array() {
	let value = r#"
		[
			[ lorem
				ipsum
			]
		]
	"#;
	key_eq(value, ".0.lorem.0", "ipsum");
}

#[test]
fn named_table_in_unnamed_array() {
	let value = r#"
		[
			{ lorem
				ipsum dolor
			}
		]
	"#;
	key_eq(value, ".0.lorem.ipsum", "dolor");
}

#[test]
fn comment() {
	let value = r#"
		# Comment
		[
			{ lorem
				ipsum dolor
			}
		]
	"#;
	let parsed = key_eq(value, ".0.lorem.ipsum", "dolor");
	assert_eq!(parsed.table().contains_key("#"), false);
}
