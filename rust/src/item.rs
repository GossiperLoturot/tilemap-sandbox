use godot::prelude::*;

pub(crate) struct ItemDescriptor {
    pub name_text: String,
    pub desc_text: String,
    pub image: Gd<godot::classes::Image>,
}

pub(crate) struct ItemStoreDescriptor {
    pub items: Vec<ItemDescriptor>,
}

struct ItemProperty {
    name_text: String,
    desc_text: String,
    image: Gd<godot::classes::Image>,
}

#[derive(GodotClass)]
#[class(no_init)]
pub(crate) struct ItemStore {
    props: Vec<ItemProperty>,
}

impl ItemStore {
    pub fn new(desc: ItemStoreDescriptor) -> Self {
        let mut props = Vec::new();
        for desc in desc.items {
            props.push(ItemProperty {
                image: desc.image,
                name_text: desc.name_text,
                desc_text: desc.desc_text,
            });
        }
        Self { props }
    }

    pub fn get_name_text(&self, id: u32) -> Option<String> {
        self.props
            .get(id as usize)
            .map(|prop| prop.name_text.clone())
    }

    pub fn get_desc_text(&self, id: u32) -> Option<String> {
        self.props
            .get(id as usize)
            .map(|prop| prop.desc_text.clone())
    }

    pub fn get_image(&self, id: u32) -> Option<Gd<godot::classes::Image>> {
        self.props.get(id as usize).map(|prop| prop.image.clone())
    }
}
