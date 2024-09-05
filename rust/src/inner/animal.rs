use super::*;

#[derive(Clone)]
pub enum EntityDataAnimalState {
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

#[derive(Clone)]
pub struct EntityDataAnimal {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub state: EntityDataAnimalState,
}

#[derive(Clone)]
pub struct EntityFeatureAnimal {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
}

impl EntityFeatureTrait for EntityFeatureAnimal {
    fn after_place(&self, root: &mut Root, key: TileKey) {
        root.entity_modify(key, |entity| {
            entity.data = EntityData::Animal(EntityDataAnimal {
                min_rest_secs: self.min_rest_secs,
                max_rest_secs: self.max_rest_secs,
                min_distance: self.min_distance,
                max_distance: self.max_distance,
                speed: self.speed,
                state: EntityDataAnimalState::Init,
            })
        })
        .unwrap();
    }

    fn before_break(&self, _root: &mut Root, _key: TileKey) {}

    fn forward(&self, root: &mut Root, key: TileKey, delta_secs: f32) {
        let mut entity = root.entity_get(key).unwrap().clone();

        let EntityData::Animal(data) = &mut entity.data else {
            return;
        };

        use rand::Rng;
        let mut rng = rand::thread_rng();
        match data.state {
            EntityDataAnimalState::Init => {
                data.state = EntityDataAnimalState::WaitStart;
            }
            EntityDataAnimalState::WaitStart => {
                data.state = EntityDataAnimalState::WaitStart;
                let secs = rng.gen_range(data.min_rest_secs..data.max_rest_secs);
                data.state = EntityDataAnimalState::Wait(secs);
            }
            EntityDataAnimalState::Wait(secs) => {
                let new_secs = secs - delta_secs;
                if secs <= 0.0 {
                    data.state = EntityDataAnimalState::TripStart;
                } else {
                    data.state = EntityDataAnimalState::Wait(new_secs);
                }
            }
            EntityDataAnimalState::TripStart => {
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

                    entity.location = location;
                } else {
                    data.state = EntityDataAnimalState::WaitStart;
                }
            }
        }

        root.entity_modify(key, move |e| *e = entity).unwrap();
    }
}
