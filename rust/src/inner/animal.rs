use super::*;

#[derive(Debug, Clone)]
pub enum EntityDataAnimalState {
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

#[derive(Debug, Clone)]
pub struct EntityDataAnimal {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub state: EntityDataAnimalState,
}

#[derive(Debug, Clone)]
pub struct EntityFeatureAnimal {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub idle_variant: u8,
    pub walk_variant: u8,
}

impl EntityFeatureTrait for EntityFeatureAnimal {
    fn after_place(&self, root: &mut Root, key: EntityKey) {
        root.entity_modify(key, |entity| {
            entity.data = Some(EntityData::Animal(EntityDataAnimal {
                min_rest_secs: self.min_rest_secs,
                max_rest_secs: self.max_rest_secs,
                min_distance: self.min_distance,
                max_distance: self.max_distance,
                speed: self.speed,
                state: EntityDataAnimalState::Init,
            }))
        })
        .unwrap();
    }

    fn before_break(&self, _root: &mut Root, _key: EntityKey) {}

    fn forward(&self, root: &mut Root, key: EntityKey, delta_secs: f32) {
        let mut entity = root.entity_get(key).unwrap().clone();

        let Some(EntityData::Animal(data)) = &mut entity.data else {
            return;
        };

        use rand::Rng;
        let mut rng = rand::thread_rng();
        match data.state {
            EntityDataAnimalState::Init => {
                data.state = EntityDataAnimalState::WaitStart;
            }
            EntityDataAnimalState::WaitStart => {
                entity.variant = Some(self.idle_variant);
                entity.tick = Some(root.tick_get() as u32);

                let secs = rng.gen_range(data.min_rest_secs..data.max_rest_secs);
                data.state = EntityDataAnimalState::Wait(secs);
            }
            EntityDataAnimalState::Wait(secs) => {
                if secs <= 0.0 {
                    data.state = EntityDataAnimalState::TripStart;
                } else {
                    let new_secs = secs - delta_secs;
                    data.state = EntityDataAnimalState::Wait(new_secs);
                }
            }
            EntityDataAnimalState::TripStart => {
                entity.variant = Some(self.walk_variant);
                entity.tick = Some(root.tick_get() as u32);

                let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                let distance = rng.gen_range(data.min_distance..data.max_distance);
                let destination = [
                    entity.location[0] + angle.cos() * distance,
                    entity.location[1] + angle.sin() * distance,
                ];
                data.state = EntityDataAnimalState::Trip(destination);
            }
            EntityDataAnimalState::Trip(destination) => {
                if entity.location != destination {
                    let difference = [
                        destination[0] - entity.location[0],
                        destination[1] - entity.location[1],
                    ];
                    let distance =
                        (difference[0] * difference[0] + difference[1] * difference[1]).sqrt();
                    let direction = [difference[0] / distance, difference[1] / distance];
                    let velocity = distance.min(data.speed * delta_secs);
                    let location = [
                        entity.location[0] + direction[0] * velocity,
                        entity.location[1] + direction[1] * velocity,
                    ];

                    if intersection_guard(root, key, location) {
                        data.state = EntityDataAnimalState::WaitStart;
                    } else {
                        entity.location = location;
                    }
                } else {
                    data.state = EntityDataAnimalState::WaitStart;
                }
            }
        }

        root.entity_modify(key, move |e| *e = entity).unwrap();
    }
}

fn intersection_guard(root: &mut Root, entity_key: EntityKey, new_location: Vec2) -> bool {
    let entity = root.entity_get(entity_key).unwrap();
    let base_rect = root.entity_get_base_collision_rect(entity.id).unwrap();

    #[rustfmt::skip]
    let rect = [[
        new_location[0] + base_rect[0][0],
        new_location[1] + base_rect[0][1], ], [
        new_location[0] + base_rect[1][0],
        new_location[1] + base_rect[1][1],
    ]];

    if root.tile_has_by_collision_rect(rect) {
        return true;
    }

    if root.block_has_by_collision_rect(rect) {
        return true;
    }

    root.entity_get_by_collision_rect(rect)
        .any(|other_key| other_key != entity_key)
}
