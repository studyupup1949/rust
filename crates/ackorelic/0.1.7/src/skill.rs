#[derive(Queryable)]
pub struct Skill {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub allocation_logic: String,
}

//impl Skill{
//    pub fn all(conn: &NRConnection) -> Result<Vec<Skill>> {
//        use crate::tables::users_skill::dsl::*;
//        users_skill
//            .load::<Skill>(conn)
//            .map_err(|e| e.context(ErrorKind::DoesNotExist).into())
//    }
//}
