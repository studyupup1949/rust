
use std::fmt::Debug;
use mysql::*;
use serde::de::DeserializeOwned;
use serde::Serialize;


pub use mysql::TxOpts;
pub use mysql::PooledConn;
pub use mysql::Transaction;
pub use mysql::prelude::*;

fn _type_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}

// 获取sql里面的 select 字段
fn get_select_field(sql: String) -> String {
    let re = regex::Regex::new(r"SELECT(.*)FROM").unwrap();
    let caps = re.captures(sql.as_str()).unwrap();

    let tmp_caps = caps[1].to_string();
    let table_field_vec: Vec<&str> = tmp_caps.split(",").collect();

    let mut field_vec: Vec<String> = vec![];
    for tf in table_field_vec.iter() {
        let temp_tf = *tf;
        if temp_tf.contains(" as ") {
            let tmpt: Vec<&str> = temp_tf.split("as").collect();
            let tmp_f = if let Some(l) = tmpt.last() {*l} else {""};
            let field: String = tmp_f.split_whitespace().collect();
            field_vec.push(field);
        } else if temp_tf.contains(" AS ") {
            let tmpt: Vec<&str> = temp_tf.split("AS").collect();
            let tmp_f = if let Some(l) = tmpt.last() {*l} else {""};
            let field: String = tmp_f.split_whitespace().collect();
            field_vec.push(field);
        } else if temp_tf.contains(".") {
            let tmpt: Vec<&str> = temp_tf.split(".").collect();
            let tmp_f = if let Some(l) = tmpt.last() {*l} else {""};
            let field: String = tmp_f.split_whitespace().collect();
            field_vec.push(field);
        } else {
            let field: String = temp_tf.split_whitespace().collect();
            field_vec.push(field);
        }
    }

    let result_field = field_vec.join(",");
    
    result_field
}


pub struct AccessMy {
    pub pool: Pool,
}

impl AccessMy {
    /// 创建一个 mysql 的连接池。min/max 为最小最大连接数
    /// ```
    /// let my_conn = AccessMy::new(10, 100, "mysql://root:12345678@localhost:3306/dev_db".to_string());
    /// ```
    pub fn new(min: usize, max: usize, url: &str) -> AccessMy {
        let pool = Pool::new_manual(min, max, url).unwrap();
        AccessMy {
            pool
        }
    }
}


/// 运行sql语句，返回上一条语句的id，如果上没有，则返回0
/// ### 用于：myset、myupdate、mydel、mysetmany
/// ### 示例
/// ```
/// let id = my_run_id(conn, myset!("feedback", {
///    "content": "ADFaadf",
///     "uid": 9,
/// }));
/// 
/// my_run_id(conn, mydel!("feedback", 50));
/// 
/// my_run_id(conn, myupdate!("feedback", 56, {
///     "content": "更新后的内容，一一一一"
/// }));
/// ```
/// 
/// 
pub fn my_run_id(conn: &mut PooledConn, sql: String) -> u64 {
    conn.query_drop(sql).unwrap();
    let id = conn.last_insert_id();
    id
}

/// 运行sql语句
/// ### 用于：myget、myfind、mycount
/// ### 示例
/// ```
/// let sql1 = myget!("feedback", 33, "id as id, feedback.content as cc");
/// #[derive(Serialize, Deserialize, Debug)]
/// struct Feedback {
///     id: u64,
///     cc: String
/// }
/// let res_get: (Vec<Feedback>, Option<(u64, String)>) = my_run(conn, sql1);
/// println!("get 结果 {:#?}", res_get);
/// ```
/// 
/// 
pub fn my_run<T, U>(conn: &mut PooledConn, sql: String) -> (Vec<U>, Option<T>)
where
    T: FromRow + Serialize + Clone + Debug,
    U: DeserializeOwned
{
    let tmp_f: String = get_select_field(sql.clone());
    let check_res: Vec<T> = conn.query(sql).unwrap_or(vec![]);
    if check_res.len() == 0 {
        (vec![], None)
    } else {
        let check_one = check_res[0].clone();
        let res: Vec<U> = json_res(check_res, tmp_f.as_str());
        (res, Some(check_one))
    }
}

/// 运行sql 语句，自定义返回字段
/// 如果 sql 语句中有多个 select 或 select 为 * 或 重新指定，就要自己修改了，此时就可以用这个函数
/// 一般情况下，用 run() 就够了
pub fn my_run_select<T, U>(conn: &mut PooledConn, sql: String, fields: &str) -> (Vec<U>, Option<T>)
where
    T: FromRow + Serialize + Clone + Debug,
    U: DeserializeOwned
{
    let tmp_f: String = fields.to_string();
    let check_res: Vec<T> = conn.query(sql).unwrap_or(vec![]);
    if check_res.len() == 0 {
        (vec![], None)
    } else {
        let check_one = check_res[0].clone();
        let res: Vec<U> = json_res(check_res, tmp_f.as_str());
        (res, Some(check_one))
    }
}

/// ### 事务执行
/// 运行sql语句，返回上一条语句的id，如果上没有，则返回0
/// ### 用于：myset、myupdate、mydel、mysetmany
/// ### 示例
/// ```
/// let id = my_do_id(tran, myset!("feedback", {
///    "content": "ADFaadf",
///     "uid": 9,
/// }));
/// 
/// my_do_id(tran, mydel!("feedback", 50));
/// 
/// my_do_id(tran, myupdate!("feedback", 56, {
///     "content": "更新后的内容，一一一一"
/// }));
/// ```
/// 
/// 
pub fn my_do_id(tran: &mut Transaction, sql: String) -> u64 {
    tran.query_drop(sql).unwrap();
    let id = tran.last_insert_id();
    let id = if let Some(i) = id {i} else {0};
    id
}
/// ### 事务执行
/// 运行sql语句
/// ### 用于：myget、myfind、mycount
/// ### 示例
/// ```
/// let sql1 = myget!("feedback", 33, "id as id, feedback.content as cc");
/// #[derive(Serialize, Deserialize, Debug)]
/// struct Feedback {
///     id: u64,
///     cc: String
/// }
/// let res_get: (Vec<Feedback>, Option<(u64, String)>) = my_do(tran, sql1);
/// println!("get 结果 {:#?}", res_get);
/// ```
/// 
/// 
pub fn my_do<T, U>(tran: &mut Transaction, sql: String) -> (Vec<U>, Option<T>)
where
    T: FromRow + Serialize + Clone + Debug,
    U: DeserializeOwned
{
    let tmp_f: String = get_select_field(sql.clone());
    let check_res: Vec<T> = tran.query(sql).unwrap_or(vec![]);
    if check_res.len() == 0 {
        (vec![], None)
    } else {
        let check_one = check_res[0].clone();
        let res: Vec<U> = json_res(check_res, tmp_f.as_str());
        (res, Some(check_one))
    }
}


/// ### 事务执行
/// 运行sql 语句，自定义返回字段
/// 如果 sql 语句中有多个 select 或 select 为 * 或 重新指定，就要自己修改了，此时就可以用这个函数
/// 一般情况下，用 run() 就够了
pub fn my_do_select<T, U>(tran: &mut PooledConn, sql: String, fields: &str) -> (Vec<U>, Option<T>)
where
    T: FromRow + Serialize + Clone + Debug,
    U: DeserializeOwned
{
    let tmp_f: String = fields.to_string();
    let check_res: Vec<T> = tran.query(sql).unwrap_or(vec![]);
    if check_res.len() == 0 {
        (vec![], None)
    } else {
        let check_one = check_res[0].clone();
        let res: Vec<U> = json_res(check_res, tmp_f.as_str());
        (res, Some(check_one))
    }
}






fn json_res<T, U>(p: Vec<T>, fields: &str) -> Vec<U> 
where
    T: FromRow + Serialize + Debug,
    U: DeserializeOwned
{
    
    let mut j_st = String::from("[");
    let field_string: String = fields.split_whitespace().collect();
    let field_list: Vec<&str> = field_string.split(",").collect();
    for item in p.iter() {
        let v_type = _type_of(item);
        if v_type.contains("(") {
            let tuple_i = serde_json::to_string_pretty(item).unwrap();
            let tm2: Vec<&str> = tuple_i.split("\n").collect();
            let tm = &tm2[1..tm2.len()-1];
            let mut one = "{".to_string();
            for (i, f_name) in field_list.iter().enumerate() {
                let mut tmp = tm[i].to_string();
                let last = &tmp[tmp.len()-1..tmp.len()];
                if last == "," {
                    tmp.pop();
                }
                one = one + "\"" + *f_name + "\": " + tmp.as_str() + ",";
            }
            one.pop();
            one.push('}');
            one.push(',');
            j_st = j_st + one.as_str();
        } else {
            let tuple_i = serde_json::to_string(item).unwrap();
            j_st = j_st + tuple_i.as_str() + ",";
        }
    }
    j_st.pop();
    j_st.push(']');
    let json_result: Vec<U> = serde_json::from_str(j_st.as_str()).unwrap();
    json_result
}