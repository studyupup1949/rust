## 介绍

accessor-macro 是一个 Rust 的派生宏（derive macro）库，用于自动生成结构体字段的 getter 和 setter 方法
该宏支持为字段生成简单的访问器方法，同时也支持在 setter 中添加范围检查逻辑，以确保设置的值在指定范围内

### 核心功能

#### 1. 自动生成 Getter 和 Setter

使用 #[derive(Accessor)] 宏来自动生成字段的 getter 和 setter
字段上使用 #[accessor(get, set)] 属性来标记需要生成 getter 和 setter 的字段


#### 2. 范围检查

在 setter 方法中可以添加范围检查逻辑，当设置的值超出指定范围时，setter 返回 false
使用 range=[min, max] 来定义字段的取值范围

##### Features

- `debug_panic`: 在 debug 模式下非法赋值时触发 panic

## 使用方法

### 1.  定义结构体并添加 `#[derive(Accessor)]`

在结构体定义前添加 #[derive(Accessor)] ，并在需要生成访问器和修改器的字段上添加 #[accessor] 属性。

```rust
use accessor_macro::Accessor;
use uintx::u24;

#[derive(Accessor, Debug)]
struct Person {
    // 生成getter和setter
    #[accessor(get, set)]
    name: String,
    // 生成getter和setter，同时添加范围检查
    #[accessor(get, set, range=[0, 200])]
    age: i32,
    // 生成getter和setter，同时添加范围检查, 范围使用表达式生成
    #[accessor(get, set, range=[u24::from(0), u24::from(100)])]
    number: u24,
    // 以结构体属性和方法作为范围检查条件
    self_test_min: i32,
    #[accessor(get, set, range=[self.self_test_min, self.get_test_max()])]
    self_test: i32,
}

impl Person {
    pub fn get_test_max(&self) -> i32 {
        100
    }
}
```

### 2.  属性说明

- get : 为字段生成 getter 方法。
- set : 为字段生成 setter 方法。
- unaligned : 未对齐字段。
- range=[min, max] : 在 setter 方法中添加范围检查，当设置的值超出范围时，返回 false 。

### 3.  示例代码

```rust
fn main() {
    let mut person = Person {
        name: "Alice".to_string(),
        age: 25,
        number: u24::from(50),
        self_test_min: 0,
        self_test: 50,
    };
    println!("person: {:?}", person);
    println!("--------------------------------------");
    println!("设置age为-1, -1 不在范围, 返回false");
    assert!(!person.set_age(-1));
    println!("person: {:?}", person);
    println!("--------------------------------------");
    println!("设置age为18, 18 在范围, 返回true");
    assert!(person.set_age(18));
    println!("person: {:?}", person);
    println!("--------------------------------------");
    println!("设置number为0, 0 在范围, 返回true");
    assert!(person.set_number(u24::from(0)));
    println!("person: {:?}", person);
    println!("--------------------------------------");
    println!("设置number为101, 101 不在范围, 返回false");
    assert!(!person.set_number(u24::from(101)));
    println!("person: {:?}", person);
    println!("--------------------------------------");
    println!("设置self_test为-1, -1 不在范围, 返回false");
    assert!(!person.set_self_test(-1));
    println!("person:{:?}", person);
    println!("--------------------------------------");
    println!("动态修改self_test_min为-100");
    person.self_test_min = -100;
    println!("设置self_test为-1, -1 在范围, 返回true");
    assert!(person.set_self_test(-1));
    println!("persion:{:?}", person);
    println!("--------------------------------------");
}
```

## 运行示例

你可以使用以下命令运行示例代码：

```rust
cargo run --example basic_usage
```

## 小声BB

这个项目是AI生成的，如果您有任何问题请咨询AI

如果您不幸联系了我，那么不好意思，我并没有能力做任何修改😊

当然如果能给该项目提交一个PR那就再好不过了👍

![OIP](./assets/rust.png)
