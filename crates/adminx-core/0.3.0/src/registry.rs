// adminx-core/src/registry.rs
use crate::menu::MenuItem;
use crate::resource::Resource;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;

lazy_static! {
    static ref RESOURCE_REGISTRY: RwLock<Vec<Box<dyn Resource>>> = RwLock::new(vec![]);
}

/// Register a resource globally.
pub fn register_resource(resource: Box<dyn Resource>) {
    RESOURCE_REGISTRY.write().unwrap().push(resource);
}

pub fn all_resources() -> Vec<Box<dyn Resource>> {
    RESOURCE_REGISTRY
        .read()
        .unwrap()
        .iter()
        .map(|r| r.clone_box())
        .collect()
}

/// Collect and group menus from every registered resource.
pub fn get_registered_menus() -> Vec<MenuItem> {
    let resources = RESOURCE_REGISTRY.read().unwrap();
    let mut grouped: HashMap<String, Vec<MenuItem>> = HashMap::new();
    let mut ungrouped: Vec<MenuItem> = Vec::new();

    for resource in resources.iter() {
        if let Some(item) = resource.generate_menu() {
            match resource.menu_group() {
                Some(group) => grouped.entry(group.to_string()).or_default().push(item),
                None => ungrouped.push(item),
            }
        }
    }

    let mut final_menus = Vec::new();
    for (group_name, mut children) in grouped {
        children.sort_by(|a, b| a.title.cmp(&b.title));
        final_menus.push(MenuItem {
            title: group_name,
            path: String::new(),
            icon: Some("folder".to_string()),
            order: Some(5),
            children: Some(children),
        });
    }
    final_menus.extend(ungrouped);

    final_menus.sort_by(|a, b| match (a.order, b.order) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.title.cmp(&b.title),
    });

    final_menus
}

pub fn clear_registry() {
    RESOURCE_REGISTRY.write().unwrap().clear();
}

pub fn resource_count() -> usize {
    RESOURCE_REGISTRY.read().unwrap().len()
}
