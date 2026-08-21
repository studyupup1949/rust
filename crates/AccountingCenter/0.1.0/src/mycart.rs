#[derive(Debug)]
#[derive(PartialEq)]
pub struct Product{
	pub id:i32,
	pub name:String,
	pub price:f32,
	pub num:f32,
}
#[derive(Debug)]
pub struct Shoppingcart{
	pub items:Vec<Product>,
}
impl Shoppingcart{
	pub fn new()->Self{
		Shoppingcart{items:vec![]}
	}
	pub fn addproduct(id:i32,name:String,price:f32,num:f32)->Product{
		Product{id:id,name:name,price:price,num:num}
	}
	pub fn showeachproduct(&self){
		for e in self.items.iter(){
			println!("商品信息：{:?}",e);
		}
		println!("{}",getlines());
		println!("一共购买了种 {} 商品",self.items.len());
		println!("{}",getlines());
	}
	pub fn totalnum(&self)->f32{
		let t = self.items.iter().fold(0.0, |sum,e|sum+e.price * e.num);
		println!("应支付总额 {} ",t);
		println!("{}",getlines());
		t
	}
}
pub fn getlines()->String{
	let mut lines = String::new();
	let n = 66;
	for _ in 1..=n{
		lines.push_str("-")
	}
	lines
}