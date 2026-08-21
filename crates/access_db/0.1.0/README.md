

### 数据库连接
  
[node 版本](https://www.npmjs.com/package/access-db)  
  
目前仅支持 mysql

```rust
use access_db::AccessMysql;

let myconn = AccessMysql::new(1, 50, "mysql://root:12345678@localhost:3306/dev_db".to_string());

let id = myconn.run_one(myset!("feedback", {
    "content": "ADFaadf",
    "uid": 9,
}));

myconn.run_one(mydel!("feedback", 50));

myconn.run_one(myupdate!("feedback", 56, {
    "content": "更新后的内容，一一一一"
}));


let sql1 = myget!("feedback", 33, "id as id, feedback.content as cc");
#[derive(Serialize, Deserialize, Debug)]
struct Feedback {
    id: u64,
    cc: String
}
let res_get: (Vec<Feedback>, Option<(u64, String)>) = myconn.run(sql1);

let sql_f = myfind!("feedback", {
    p0: ["uid", ">", 330],
    r: "p0",
    page: 2,
    limit: 5,
    select: "id, content as cc",
});
let res_find: (Vec<Feedback>, Option<(u64, String)>) = myconn.run(sql_f);

let res_count: (Vec<u64>, Option<u64>) = myconn.run(mycount!("feedback", {}));


```