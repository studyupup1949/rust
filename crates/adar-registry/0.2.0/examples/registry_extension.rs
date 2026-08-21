use adar_registry::prelude::*;
struct MenuItem(pub &'static str);
struct StyleSheet(pub &'static str);

fn main() {
    // Original website setup
    let menu = Registry::<MenuItem>::new();
    let styles = Registry::<StyleSheet>::new();
    let mut website_store = vec![];

    website_store.push(menu.register(MenuItem("Home")).as_generic());
    website_store.push(menu.register(MenuItem("About")).as_generic());
    website_store.push(styles.register(StyleSheet("website.css")).as_generic());

    print_state("Original website", &menu, &styles);

    // Loading an extension which registers new resources
    let mut extension_store = vec![];
    extension_store.push(menu.register(MenuItem("Weather")).as_generic());
    extension_store.push(menu.register(MenuItem("News")).as_generic());
    extension_store.push(styles.register(StyleSheet("extension.css")).as_generic());

    print_state("After extension is loaded", &menu, &styles);

    // Unloading the extension
    drop(extension_store);

    print_state("After extension is unloaded", &menu, &styles);
}

fn print_state(step: &'static str, menu: &Registry<MenuItem>, styles: &Registry<StyleSheet>) {
    println!("{}", step);
    println!("\tMenu:");
    for (e, item) in menu.read().iter() {
        println!("\t\t{}: {}", e, item.0);
    }
    println!("\tStyleSheets:");
    for (e, item) in styles.read().iter() {
        println!("\t\t{}: {}", e, item.0);
    }
}
