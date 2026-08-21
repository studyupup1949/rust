use adar::prelude::*;

#[TraitRef]
trait Module {
    fn update(&self);
}

struct DefaultModule;

impl Module for DefaultModule {
    fn update(&self) {
        println!("DefaultModule update!");
    }
}

struct DefaultModule2;

impl Module for DefaultModule2 {
    fn update(&self) {
        println!("DefaultModule2 update!");
    }
}

fn default_modules() -> (DefaultModule, DefaultModule2) {
    (DefaultModule, DefaultModule2)
}

struct ExternalModule;

impl Module for ExternalModule {
    fn update(&self) {
        println!("ExternalModule update!");
    }
}

struct Application<T>
where
    T: TupleTraitIter,
{
    modules: T,
}

impl<T> Application<T>
where
    T: TupleTraitIter + TupleAtTrait<dyn Module>,
{
    fn new(external_modules: T) -> Self {
        Self {
            modules: external_modules,
        }
    }

    fn update(&self) {
        for module in self.modules.iter_trait::<dyn Module>() {
            module.update();
        }
    }
}

fn main() {
    let app = Application::new(default_modules().concat((ExternalModule,)));
    app.update();
}
