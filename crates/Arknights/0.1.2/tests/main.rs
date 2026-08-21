use arknights::{DoctorLevelDB, LevelUpCostDB};

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn test_cost() {
    println!("6 星从 001 升到 210 需要:");
    let model = LevelUpCostDB::star6_cash();
    let exp = model.cost_detail((0, 01), (2, 10)).unwrap();
    println!("龙门币: {:?}", exp.sum());
    let model = LevelUpCostDB::star6_exp();
    let exp = model.cost_detail((0, 01), (2, 10)).unwrap();
    println!("经验值: {:?}", exp.sum());
}

#[test]
fn test_ap() {
    let model = DoctorLevelDB::default();
    let level = 20;
    let exp = model.cost(1, level).unwrap();
    println!("博士从 1 级升到 {} 级需要 {} 经验", level, exp);
    let time = model.action_point_regen_time(level, 0);
    println!("{} 级博士从 0 回满理智需要 {:?}", level, time);
}
