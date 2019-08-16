extern crate rand;
extern crate rayon;

use std::io;
use std::collections::HashMap;
use std::time::SystemTime;

use rand::prelude::ThreadRng;
use rand::Rng;
use std::mem;
use rand::seq::index::sample;
use std::process::exit;

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
struct Enemy {
    health: u32,
    armor: u32,
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

    fn get_average_time_to_kill(&self, distribution_cache: &DiceDistributionCache, weapon: &Weapon) -> f32 {
        let distribution = distribution_cache.distribution_for(weapon.dice_count as usize);
        let mut average_damage: f32 = 0.0;
        for res in weapon.dice_count..=weapon.dice_count * 6 {
            let probability = distribution.probability_of(res as usize);
            let damage = self.get_damage(res as i32, weapon);
            average_damage += damage as f32 * probability;
        }

        let health = self.health as f32;
        if average_damage.abs() < 0.1 || health > 300f32 * average_damage {
            -400f32
        } else {
            (health / average_damage).round()
        }
    }
}

#[derive(Clone, Debug)]
struct Sample {
    weapons: Vec<Weapon>,
    enemies: Vec<Enemy>,

    score: f32,
}

impl Sample {
    fn breed_with(&self, random: &mut ThreadRng, partner: &Sample) -> Sample {
        let mut child: Sample = self.clone();

        for (index, _) in self.weapons.iter().enumerate() {
            let mut weapon: Weapon = child.weapons.get(index).unwrap().clone();

            if random.gen_bool(0.5) {
                if random.gen_bool(0.01) {
                    weapon.dice_count = random.gen_range(1, 26);
                } else {
                    weapon.dice_count = partner.weapons.get(index).unwrap().dice_count;
                }
            }
            if random.gen_bool(0.5) {
                if random.gen_bool(0.01) {
                    weapon.modifier = random.gen_range(1, 13);
                } else {
                    weapon.modifier = partner.weapons.get(index).unwrap().modifier;
                }
            }
            if random.gen_bool(0.5) {
                if random.gen_bool(0.01) {
                    let armor_multiplier_scale: u32 = random.gen_range(1, 21);

                    weapon.armor_multiplier = armor_multiplier_scale as f32 * 0.1;
                } else {
                    weapon.armor_multiplier = partner.weapons.get(index).unwrap().armor_multiplier;
                }
            }
            if random.gen_bool(0.5) {
                if random.gen_bool(0.01) {
                    let health_multiplier_scale: u32 = random.gen_range(1, 5);

                    weapon.health_multiplier = health_multiplier_scale as f32 * 0.5;
                } else {
                    weapon.health_multiplier = partner.weapons.get(index).unwrap().health_multiplier;
                }
            }

            child.weapons[index] = weapon.clone();
        }

        for (index, _) in self.enemies.iter().enumerate() {
            let mut enemy: Enemy = child.enemies.get(index).unwrap().clone();

            if random.gen_bool(0.5) {
                if random.gen_bool(0.01) {
                    enemy.health = random.gen_range(0, 101);
                } else {
                    enemy.health = partner.enemies.get(index).unwrap().health;
                }
            }
            if random.gen_bool(0.5) {
                if random.gen_bool(0.01) {
                    enemy.armor = random.gen_range(0, 101);
                } else {
                    enemy.armor = partner.enemies.get(index).unwrap().armor;
                }
            }

            child.enemies[index] = enemy.clone();
        }

        child
    }

    fn gen(random: &mut ThreadRng, weapon_count: u32, enemy_count: u32) -> Sample {
        Sample {
            weapons: (0..weapon_count)
                .map(|_| Weapon::generate_random(random))
                .collect(),
            enemies: (0..enemy_count)
                .map(|_| Enemy::generate_random(random))
                .collect(),

            score: 0f32,
        }
    }
}

#[derive(Debug, Clone)]
struct DiceDistribution {
    dice_count: usize,
    chances: Vec<f32>,
}

#[derive(Debug, Clone)]
struct DiceDistributionCache {
    distributions: Vec<DiceDistribution>,
}

impl DiceDistribution {
    fn precomputed_for(dice_count: usize) -> Self {
        let mut random = rand::thread_rng();

        let total_count = 10000;

        // At i'th index, we store count of rolls with sum equal to i + dice_count
        let mut roll_count: Vec<usize> = (dice_count..=dice_count * 6).map(|_| 0).collect();
        for _ in 0..total_count {
            let roll: usize = (0..dice_count).map(|_| random.gen_range(1, 7)).sum();
            roll_count[roll - dice_count] += 1;
        }

        Self {
            dice_count,
            chances: roll_count
                .into_iter()
                .map(|x| x as f32 / total_count as f32)
                .collect(),
        }
    }

    fn probability_of(&self, roll: usize) -> f32 {
        self.chances[roll - self.dice_count]
    }
}

impl DiceDistributionCache {
    fn precomputed(max_dice_count: usize) -> Self {
        Self {
            distributions: (1..=max_dice_count)
                .map(|dc| DiceDistribution::precomputed_for(dc))
                .collect(),
        }
    }

    fn distribution_for(&self, dice_count: usize) -> &DiceDistribution {
        &self.distributions[dice_count - 1]
    }
}

fn main() -> Result<(), io::Error> {
    let backend = RustboxBackend::new()?;
    let mut terminal = Terminal::new(backend);
    Ok(())
}

fn _main() {
    const POPULATION_SIZE: u32 = 150;

    let distributions_cache = DiceDistributionCache::precomputed(26);
    let perfect_json = r#"{"Дробовик":{"Грант":9,"Титан":60,"Ховер":-400,"Громила":15,"Ремонтник":3,"Берсерк":15,"Гладиатор":30,"Банши":12,"Монстр":-400,"Надзиратель":6,"Брак":3,"Хаунд":21,"Турель":6,"Зомби":3,"Шмель":-400,"Дропер":-400,"Мясо":3,"Тактик":6,"Процессор":3,"Колобок":20},"Гвоздевик":{"Грант":7,"Титан":25,"Ховер":60,"Громила":10,"Ремонтник":1,"Берсерк":6,"Гладиатор":17,"Банши":5,"Монстр":40,"Надзиратель":4,"Брак":1,"Хаунд":13,"Турель":4,"Зомби":1,"Шмель":30,"Дропер":35,"Мясо":2,"Тактик":2,"Процессор":3,"Колобок":10},"Гипербластер":{"Грант":30,"Титан":80,"Ховер":220,"Громила":40,"Ремонтник":4,"Берсерк":18,"Гладиатор":60,"Банши":24,"Монстр":140,"Надзиратель":12,"Брак":2,"Хаунд":48,"Турель":12,"Зомби":2,"Шмель":125,"Дропер":110,"Мясо":4,"Тактик":8,"Процессор":12,"Колобок":50},"Клиповик":{"Грант":25,"Титан":0,"Ховер":0,"Громила":35,"Ремонтник":2,"Берсерк":12,"Гладиатор":57,"Банши":18,"Монстр":0,"Надзиратель":8,"Брак":2,"Хаунд":43,"Турель":8,"Зомби":1,"Шмель":0,"Дропер":0,"Мясо":3,"Тактик":5,"Процессор":8,"Колобок":20},"Бластер":{"Грант":-400,"Титан":-400,"Ховер":-400,"Громила":-400,"Ремонтник":5,"Берсерк":30,"Гладиатор":-400,"Банши":-400,"Монстр":-400,"Надзиратель":-400,"Брак":4,"Хаунд":-400,"Турель":-400,"Зомби":2,"Шмель":-400,"Дропер":-400,"Мясо":7,"Тактик":12,"Процессор":20,"Колобок":-400}}"#;
    let weapons_json =
        r#"["Бластер","Клиповик","Дробовик","Гипербластер","Гвоздевик"]"#;
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

    let mut samples: Vec<Sample> = (0..POPULATION_SIZE)
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
        score: 0f32,
    };

    let mut generation_number = 1;

    while /*best_sample.score < 1f32*/ true {
        let mut total_score: f32 = 0f32;

        samples.iter_mut().for_each(|sample| {
            let mut total_error = 0f32;

            for (weapon_type, weapon) in sample.weapons.iter().enumerate() {
                for (enemy_type, enemy) in sample.enemies.iter().enumerate() {
                    let current_time_to_kill = enemy.get_average_time_to_kill(&distributions_cache, &weapon);

                    let perfect_time_to_kill = perfect[&(weapon_type as u32, enemy_type as u32)];
                    let stat_error = (current_time_to_kill - perfect_time_to_kill as f32).powi(2);

                    total_error += stat_error;
                }
            }

            sample.score = 1f32 / total_error;

            total_score += sample.score;
        });

        let mut random = rand::thread_rng();
        let mut temp_samples: Vec<Sample> = Vec::new();

        samples.iter().for_each(|sample| {
            if sample.score > total_score / POPULATION_SIZE as f32 {
                if (sample.score - best_sample.score) > 0f32 {
                    best_sample = sample.clone();
                    temp_samples.push(sample.clone());

                    println!("New best score {}", best_sample.score);
                } else {
                    let roulette_score = sample.score / total_score;

                    let rand_float: f32 = random.gen_range(0f32, 1f32);

                    if roulette_score > rand_float {
                        temp_samples.push(sample.clone());
                    }
                }
            }

            total_score -= sample.score;
        });

        samples.push(best_sample.clone());
        samples = temp_samples;

        let lack = POPULATION_SIZE - samples.len() as u32;

        let mut temp_samples = samples.clone();

        if samples.len() > 1 {
            for _ in 0..(lack as f32 * 0.1) as u32 {
                let father: Sample = samples.get(random.gen_range(1, samples.len())).unwrap().clone();

                let new_sample = best_sample.breed_with(&mut random.clone(), &father);
                temp_samples.push(new_sample);
            }

            for _ in 0..(lack as f32 * 0.6) as u32 {
                let mother: Sample = samples.get(random.gen_range(1, samples.len())).unwrap().clone();
                let father: Sample = samples.get(random.gen_range(1, samples.len())).unwrap().clone();

                let new_sample = mother.breed_with(&mut random.clone(), &father);
                temp_samples.push(new_sample);
            }
        }


        while temp_samples.len() < POPULATION_SIZE as usize {
            let new_sample = Sample::gen(&mut random, weapon_types_vec.len() as u32, enemy_types_vec.len() as u32);
            temp_samples.push(new_sample)
        }

        samples = temp_samples;

        generation_number += 1;
    }
}
