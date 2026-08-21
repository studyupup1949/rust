// Previous code was written yy me (80%), but the ideas and problem solving is more then half done by ai.
// All hardparts are done by AI. (that beign the 50% ), i did only intermidiate parts.
// U saw a vid on 'CognetiveOFfLoading' TypeShi and cauz of htat u want to quitp
#![allow(unused, dead_code)]
pub struct PersonTypeName {
    customer: String,
    share_holder: WorkerType,
    worker: WorkerType,
    manager: ManagerType,
}
pub struct Person {
    _type: PersonTypeName,
    name: String, // <- Make this into a typeTable PersonName, so you can access multiple persons
    pid: u32, // <- Use a hashMap system for this. But hashmap into sqlite. And you can call methods on the struct.
              // which calls to the database which manipulates the database., use a lock system to prevent data. races.
}

pub struct Customer {
    name: String,
    _type: CustomerType,
    level: CustomerLevel,
    bottels_owned: u64, // <- Use a bottels track system. Owned by, liability, still owns money, will give money rather bottels,
    // If x days pased no bottel return only money. If bottel quality below something no return. If dirt.
    // a bottel struct dataBase. So it has 'quality' 'date_taken_on' and then in future a 'days passed'
    ledgers_included_in: crate::acc_soft::l_ident::LPID, // <- Again only  a boilerplate, make database.
    // conncects related APID to LPID.
    a_related_to: PersonTypeName,
}

// TODO!().
// Cauz this is not completed.
pub enum CustomerLevel {}
pub struct CustomerType {}
pub struct WorkerType {}
pub struct ManagerType {}
pub struct Bottels {}
pub struct ShareHolderType {}
