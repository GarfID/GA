extern crate rand;
extern crate rayon;

use rand::Rng;
use rayon::prelude::*;

use rand::prelude::ThreadRng;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Clone, Copy)]
struct Weapon {
    dice_count: u32,
    modifier: i32,
    armor_multiplier: f32,
    health_multiplier: f32,
}

impl Weapon {
    fn generate_random(random: &mut ThreadRng) -> Weapon {
        let armor_multiplier_scale: u32 = random.gen_range(1, 21);
        let health_multiplier_scale: u32 = random.gen_range(1, 5);

        Weapon {
            dice_count: random.gen_range(1, 26),
            modifier: random.gen_range(1, 13),
            armor_multiplier: armor_multiplier_scale as f32 * 0.1,
            health_multiplier: health_multiplier_scale as f32 * 0.5,
        }
    }
}

#[derive(Clone, Copy)]
struct Enemy {
    health: u32,
    armor: u32,
}

impl Default for Enemy {
    fn default() -> Enemy {
        Enemy {
            health: 1,
            armor: 1,
        }
    }
}

impl Enemy {
    fn generate_random(random: &mut ThreadRng) -> Enemy {
        Enemy {
            health: random.gen_range(1, 101),
            armor: random.gen_range(0, 101),
        }
    }

    fn get_damage(&self, mut res: i32, weapon: &Weapon) -> u32 {
        res += weapon.modifier;

        let res = if res < 0 { 0 } else { res };

        let mut passed = res - (self.armor as f32 * weapon.armor_multiplier) as i32;
        passed = if passed < 0 { 0 } else { passed };

        (passed as f32 * weapon.health_multiplier) as u32
    }

    fn kill(&self, weapon: &Weapon) -> i32 {
        let mut steps: i32 = 0;
        let mut damage: f32 = 0.0;
        for res in weapon.dice_count..=weapon.dice_count * 6 {
            damage +=
                self.get_damage(res as i32, weapon) * get_roll_posibility(res, weapon.dice_count);
        }

        if damage.abs() < 0.1 {
            steps = -400
        } else {
            steps = (self.health as f32 / damage).round() as i32;

            if steps > 300 {
                steps = -400
            }
        }

        steps
    }
}

#[derive(Clone)]
struct Sample {
    weapons: Vec<Weapon>,
    enemies: Vec<Enemy>,

    score: u32,
}

impl Sample {
    fn breed_with(&self, partner: &Sample) -> Sample {
        unimplemented!()
    }

    fn gen(random: &mut ThreadRng, weapon_count: u32, enemy_count: u32) -> Sample {
        Sample {
            weapons: (0..weapon_count)
                .map(|_| Weapon::generate_random(random))
                .collect(),
            enemies: (0..enemy_count)
                .map(|_| Enemy::generate_random(random))
                .collect(),

            score: 0,
        }
    }
}

struct DiceDistribution {
    chances: Vec<Vec<f32>>,
}

impl DiceDistribution {
    fn precompute() -> DiceDistribution {
        let chances: Vec<Vec<f32>>;

        let mut random = rand::thread_rng();

        for dice_count in 1..26 {
            for _ in 0..10000 {
                let roll = (0..dice_count).map(|_| random.gen_range(1, 7)).sum();
            }
        }

        DiceDistribution { chances }
    }
}

fn get_roll_posibility(roll: u32, count: u32) -> f32 {
    unimplemented!()
}

fn main() {
    let perfect_json = r#"{"Дробовик":{"Грант":9,"Титан":60,"Ховер":-400,"Громила":15,"Ремонтник":3,"Берсерк":15,"Гладиатор":30,"Банши":12,"Монстр":-400,"Надзиратель":6,"Брак":3,"Хаунд":21,"Турель":6,"Зомби":3,"Шмель":-400,"Дропер":-400,"Мясо":3,"Тактик":6,"Процессор":3,"Колобок":20},"Гвоздевик":{"Грант":7,"Титан":25,"Ховер":60,"Громила":10,"Ремонтник":1,"Берсерк":6,"Гладиатор":17,"Банши":5,"Монстр":40,"Надзиратель":4,"Брак":1,"Хаунд":13,"Турель":4,"Зомби":1,"Шмель":30,"Дропер":35,"Мясо":2,"Тактик":2,"Процессор":3,"Колобок":10},"Гипербластер":{"Грант":30,"Титан":80,"Ховер":220,"Громила":40,"Ремонтник":4,"Берсерк":18,"Гладиатор":60,"Банши":24,"Монстр":140,"Надзиратель":12,"Брак":2,"Хаунд":48,"Турель":12,"Зомби":2,"Шмель":125,"Дропер":110,"Мясо":4,"Тактик":8,"Процессор":12,"Колобок":50},"Клиповик":{"Грант":25,"Титан":0,"Ховер":0,"Громила":35,"Ремонтник":2,"Берсерк":12,"Гладиатор":57,"Банши":18,"Монстр":0,"Надзиратель":8,"Брак":2,"Хаунд":43,"Турель":8,"Зомби":1,"Шмель":0,"Дропер":0,"Мясо":3,"Тактик":5,"Процессор":8,"Колобок":20},"Бластер":{"Грант":-400,"Титан":-400,"Ховер":-400,"Громила":-400,"Ремонтник":5,"Берсерк":30,"Гладиатор":-400,"Банши":-400,"Монстр":-400,"Надзиратель":-400,"Брак":4,"Хаунд":-400,"Турель":-400,"Зомби":2,"Шмель":-400,"Дропер":-400,"Мясо":7,"Тактик":12,"Процессор":20,"Колобок":-400}}"#;
    let weapons_json = r#"["Бластер","Клиповик","Дробовик","Гипербластер","Гвоздевик"]"#;
    let enemies_json = r#"["Зомби","Брак","Мясо","Тактик","Турель","Надзиратель","Берсерк","Грант","Процессор","Громила","Банши","Гладиатор","Хаунд","Титан","Монстр","Ховер","Ремонтник","Колобок","Шмель","Дропер"]"#;

    let weapon_types_vec: Vec<String> = serde_json::from_str(weapons_json).unwrap();
    let enemy_types_vec: Vec<String> = serde_json::from_str(enemies_json).unwrap();

    let mut weapon_name_to_id: HashMap<String, u32> = HashMap::new();
    let mut enemy_name_to_id: HashMap<String, u32> = HashMap::new();

    for (index, weapon_type_vec) in weapon_types_vec.iter().enumerate() {
        weapon_name_to_id.insert(weapon_type_vec.clone(), index as u32);
    }
    for (index, enemy_type_vec) in enemy_types_vec.iter().enumerate() {
        enemy_name_to_id.insert(enemy_type_vec.clone(), index as u32);
    }

    let perfect_fucked_up_view: HashMap<String, HashMap<String, i32>> =
        serde_json::from_str(perfect_json).unwrap();

    let mut perfect: HashMap<(u32, u32), i32> = HashMap::new();
    for (t_weapon_type, t_enemy_ttks) in perfect_fucked_up_view {
        for (t_enemy_type, time_to_kill) in t_enemy_ttks {
            perfect.insert(
                (
                    *weapon_name_to_id.get(&t_weapon_type).unwrap(),
                    *enemy_name_to_id.get(&t_enemy_type).unwrap(),
                ),
                time_to_kill,
            );
        }
    }

    let mut samples: Vec<Sample> = (0..500)
        .map(|_| {
            Sample::gen(
                &mut rand::thread_rng(),
                weapon_types_vec.len() as u32,
                enemy_types_vec.len() as u32,
            )
        })
        .collect();

    let mut best_sample: Sample = Sample {
        weapons: Vec::new(),
        enemies: Vec::new(),
        score: 10000,
    };

    let generation_number = 1;

    while best_sample.score > 10 {
        println!("Поколение {}", generation_number);

        let total_start = SystemTime::now();

        samples.iter_mut().for_each(|sample| {
            let start = SystemTime::now();

            for (weapon_type, weapon) in sample.weapons.iter().enumerate() {
                for (enemy_type, enemy) in sample.enemies.iter().enumerate() {
                    let current_enemy: Enemy = enemy.clone();
                    let current_time_to_kill = current_enemy.kill(&weapon);

                    let perfect_time_to_kill = *perfect
                        .get(&(weapon_type as u32, enemy_type as u32))
                        .unwrap();
                    let stat_error = (current_time_to_kill - perfect_time_to_kill).pow(2) as u32;

                    sample.score += stat_error;
                }
            }

            let stop = SystemTime::now();
            println!("Done evaluating in {:?}", stop.duration_since(start));
        });

        let total_stop = SystemTime::now();

        //println!("Complete in {:?}", total_stop.duration_since(total_start));
    }
}
