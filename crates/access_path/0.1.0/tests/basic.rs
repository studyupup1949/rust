use access_path::KeyPath;

// ═══════════════════════════════════════════════════════════════
//  基础测试
// ═══════════════════════════════════════════════════════════════

#[derive(KeyPath, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

#[test]
fn get_field() {
    let p = Person {
        name: "Alice".into(),
        age: 30,
    };
    assert_eq!(*Person::name_path().get(&p), "Alice");
    assert_eq!(*Person::age_path().get(&p), 30);
}

#[test]
fn get_mut_field() {
    let mut p = Person {
        name: "Bob".into(),
        age: 25,
    };
    *Person::name_path().get_mut(&mut p) = "Charlie".into();
    assert_eq!(p.name, "Charlie");
    assert_eq!(p.age, 25); // unchanged
}

#[test]
fn set_field() {
    let mut p = Person {
        name: "Dave".into(),
        age: 40,
    };
    Person::age_path().set(&mut p, 50);
    assert_eq!(p.age, 50);
    assert_eq!(p.name, "Dave"); // unchanged
}

#[test]
fn path_is_copy() {
    let np = Person::name_path();
    let np2 = np; // Copy
    let p = Person {
        name: "Eve".into(),
        age: 20,
    };
    assert_eq!(*np.get(&p), "Eve");
    assert_eq!(*np2.get(&p), "Eve"); // still works after copy
}

// ═══════════════════════════════════════════════════════════════
//  多种字段类型
// ═══════════════════════════════════════════════════════════════

#[derive(KeyPath, Debug, PartialEq)]
struct VariousTypes {
    int: i32,
    float: f64,
    boolean: bool,
    string_val: String,
}

#[test]
fn various_types() {
    let v = VariousTypes {
        int: 42,
        float: 3.14,
        boolean: true,
        string_val: "hello".into(),
    };

    assert_eq!(*VariousTypes::int_path().get(&v), 42);
    assert!((*VariousTypes::float_path().get(&v) - 3.14).abs() < 1e-10);
    assert!(*VariousTypes::boolean_path().get(&v));
    assert_eq!(*VariousTypes::string_val_path().get(&v), "hello");
}

// ═══════════════════════════════════════════════════════════════
//  多个字段 + 路径作为变量
// ═══════════════════════════════════════════════════════════════

#[derive(KeyPath, Debug, PartialEq)]
struct Point {
    x: f64,
    y: f64,
    label: String,
}

#[test]
fn path_variable() {
    let x_path = Point::x_path();
    let y_path = Point::y_path();

    let pt = Point {
        x: 1.0,
        y: 2.0,
        label: "origin".into(),
    };
    assert!((*x_path.get(&pt) - 1.0).abs() < 1e-10);
    assert!((*y_path.get(&pt) - 2.0).abs() < 1e-10);
}

#[test]
fn path_variable_mut() {
    let x_path = Point::x_path();
    let mut pt = Point {
        x: 0.0,
        y: 0.0,
        label: "".into(),
    };
    *x_path.get_mut(&mut pt) = 10.0;
    assert!((pt.x - 10.0).abs() < 1e-10);
}

// ═══════════════════════════════════════════════════════════════
//  存储路径在集合中
// ═══════════════════════════════════════════════════════════════

#[test]
fn paths_are_zst() {
    use std::mem::size_of_val;
    let xp = Point::x_path();
    let yp = Point::y_path();
    assert_eq!(size_of_val(&xp), 0);
    assert_eq!(size_of_val(&yp), 0);
}

// ═══════════════════════════════════════════════════════════════
//  泛型 struct
// ═══════════════════════════════════════════════════════════════

#[derive(KeyPath, Debug, PartialEq)]
struct Wrapper<T: Clone> {
    value: T,
    label: String,
}

#[test]
fn generic_struct() {
    let w = Wrapper::<i32> {
        value: 42,
        label: "num".into(),
    };
    assert_eq!(*Wrapper::<i32>::value_path().get(&w), 42);
    assert_eq!(*Wrapper::<i32>::label_path().get(&w), "num");
}

#[test]
fn generic_struct_string() {
    let w = Wrapper::<String> {
        value: "hello".to_string(),
        label: "greeting".into(),
    };
    assert_eq!(*Wrapper::<String>::value_path().get(&w), "hello");
}

// ═══════════════════════════════════════════════════════════════
//  生命周期参数
// ═══════════════════════════════════════════════════════════════

#[derive(KeyPath, Debug, PartialEq)]
struct Borrowed<'a> {
    name: &'a str,
    age: u32,
}

#[test]
fn lifetime_param() {
    let b = Borrowed {
        name: "Alice",
        age: 30,
    };
    assert_eq!(*Borrowed::name_path().get(&b), "Alice");
    assert_eq!(*Borrowed::age_path().get(&b), 30);
}

// ═══════════════════════════════════════════════════════════════
//  const 泛型
// ═══════════════════════════════════════════════════════════════

#[derive(KeyPath, Debug, PartialEq)]
struct Array<const N: usize> {
    len: usize,
    data: [u8; N],
}

#[test]
fn const_generic() {
    let a = Array::<5> {
        len: 5,
        data: [1, 2, 3, 4, 5],
    };
    assert_eq!(*Array::<5>::len_path().get(&a), 5);
}

// ═══════════════════════════════════════════════════════════════
//  where 子句
// ═══════════════════════════════════════════════════════════════

#[derive(KeyPath, Debug, PartialEq)]
struct WithWhere<T>
where
    T: Default + PartialEq,
{
    value: T,
    extra: String,
}

#[test]
fn with_where() {
    let s = WithWhere::<i32> {
        value: 42,
        extra: "x".into(),
    };
    assert_eq!(*WithWhere::<i32>::value_path().get(&s), 42);
}

// ═══════════════════════════════════════════════════════════════
//  嵌套路径（手动组合）
// ═══════════════════════════════════════════════════════════════

#[derive(KeyPath, Debug, PartialEq)]
struct Company {
    ceo: Person,
    name: String,
}

#[test]
fn nested_manual() {
    let c = Company {
        ceo: Person {
            name: "Alice".into(),
            age: 30,
        },
        name: "Acme".into(),
    };

    // 使用 then() 组合路径：
    let ceo_age = Company::ceo_path().then(Person::age_path());
    assert_eq!(*ceo_age.get(&c), 30);
}

#[test]
fn nested_set_via_composed() {
    let mut c = Company {
        ceo: Person {
            name: "Alice".into(),
            age: 30,
        },
        name: "Acme".into(),
    };

    let ceo_age = Company::ceo_path().then(Person::age_path());
    ceo_age.set(&mut c, 50);
    let ceo_name = Company::ceo_path().then(Person::name_path());
    ceo_name.set(&mut c, "Bob".into());

    assert_eq!(c.ceo.age, 50);
    assert_eq!(c.ceo.name, "Bob");
}

// ═══════════════════════════════════════════════════════════════
//  空 struct
// ═══════════════════════════════════════════════════════════════

#[derive(KeyPath, Debug, PartialEq)]
struct Empty {}

#[test]
fn empty_struct() {
    let _e = Empty {};
}
