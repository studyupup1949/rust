/// The type this module spins on.
///
/// Right now it's very, very sparse. I want to make sure I can get the basics
/// right before I go chasing after isomers and isotopes and so on.
///
/// The main thing to note is that groups, the optional member describing the
/// *functional groups* of a Substance, is Boxed (heap allocation). That's to
/// prevent the infinite recursion of allocating memory for groups on the stack.
///
/// This means that accessing the functional groups requires an access, an
/// unwrap/Option check, and then a dereference. Wheee!
///
/// # Examples
///
/// Here's an (blithe, reckless, unwrapping) example of group access:
///
/// ```
///
/// use acetylene_parser::tokenize;
/// use acetylene_parser::types::Substance;
///
/// let benzene_ring = r"c1ccccc1";
/// let res = tokenize(benzene_ring, "smiles");
/// 
/// println!("Main substance: {:?}", &res.symbol);
/// println!("Groups: ");
/// for sub in *(res.groups.unwrap()) {
///   println!("{:?}", sub);
/// }
/// ```
///
#[derive(Debug, Eq, Clone)]
pub struct Substance {
  pub symbol: String,
  pub quantity: u32,
  pub charge: Option<i32>,
  pub groups: Option<Box<Vec<Substance>>>
  //   isotope: Option<u32>,
  //   isomers: Option<Box<Vec<Substance>>>,
  //   mass: Option<f32>,
  //   atomic_number: u32
}

impl PartialEq for Substance {
  fn eq(&self, other: &Substance) -> bool {
    ((self.symbol == other.symbol) &
     (self.quantity == other.quantity) &
     (self.charge == other.charge))
  }
}
