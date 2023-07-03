use std::collections::HashMap;
use std::fs;
use serde::Deserialize;
use rand::Rng;

#[derive(Default)]
pub struct Gacha {
    pub items: Vec<Item>,
    pub five_star_pity_counter: i32,
    pub four_star_pity_counter: i32,
    pub is_last_five_star_item_limited: bool, // TODO: USE THIS
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub item_type: ItemType,
    pub description: String,
    pub image_url: String
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    FiveStarCharacterLimited,
    FiveStarCharacterStandard,
    FiveStarWeapon,
    FourStarCharacter,
    FourStarWeapon,
    ThreeStarItem,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ItemFromJson {
    pub name: String,
    pub item_type: String,
    pub description: String,
    pub image_url: String
}

impl Gacha {

    pub fn roll(&mut self) -> Item {
        let five_star_pity_chances: HashMap<i32, i32> = self.build_five_star_pity_chances();
        let four_star_pity_chances: HashMap<i32, i32> = self.build_four_star_pity_chances();

        // Check for five or four star item roll
        if self.chance_hit(self.five_star_pity_counter, &five_star_pity_chances) {
            println!{"Five star! {}", self.five_star_pity_counter};
            self.five_star_pity_counter = 0;
            return self.pity_roll(ItemType::FiveStarCharacterLimited,
                                  ItemType::FiveStarCharacterStandard,
                                  ItemType::FiveStarWeapon);
        } else {
            self.five_star_pity_counter += 1;
        }

        if self.chance_hit(self.four_star_pity_counter, &four_star_pity_chances) {
            println!("Four star: {}", self.four_star_pity_counter);
            self.four_star_pity_counter = 0;
            return self.pity_roll(ItemType::FourStarCharacter,
                                  ItemType::FourStarWeapon,
                                  ItemType::FourStarCharacter);
        } else {
            self.four_star_pity_counter += 1;
        }

        // Default to a random 3-star item
        return self.random_item_of_type(ItemType::ThreeStarItem);
    }

    pub fn build_five_star_pity_chances(&self) -> HashMap<i32, i32> {
        let mut five_star_pity_chances = HashMap::new();

        for pity in 0..75 {
            five_star_pity_chances.insert(pity, 6);  // 0.6%
        }
    
        // increase of 0.6% per pity after the 75th
        for pity in 75..=90 {
            five_star_pity_chances.insert(pity, 60 + (pity - 75) * 60);
        }

        five_star_pity_chances
    }

    pub fn build_four_star_pity_chances(&self) -> HashMap<i32, i32> {
        let mut four_star_pity_chances = HashMap::new();

        for pity in 0..9 {
            four_star_pity_chances.insert(pity, 51);
        }

        // pity 9 gives 56.1%, and pity 10 gives 100%
        four_star_pity_chances.insert(9, 561);
        four_star_pity_chances.insert(10, 1000);

        four_star_pity_chances
    }

    pub fn chance_hit(&self, pity_counter: i32, pity_chances: &HashMap<i32, i32>) -> bool {
        let rng = rand::thread_rng().gen_range(0..=1000);
        let pity_chance = *pity_chances.get(&pity_counter).unwrap_or(&0);

        rng <= pity_chance
    }

    pub fn multi_roll(&mut self, number_of_rolls: u32) -> Vec<Item> {
        let mut roll_results = Vec::with_capacity(number_of_rolls.try_into().unwrap()); // Preallocating memory       

        for _ in 0..number_of_rolls {
            roll_results.push(self.roll());
            println!("{} | {}", self.four_star_pity_counter, self.five_star_pity_counter)
        }

        return roll_results;
    }

    pub fn random_item_of_type(&self, item_type: ItemType) -> Item {
        let items_of_type: Vec<&Item> = self.items.iter().filter(|item| item.item_type == item_type).collect();
        let idx = rand::thread_rng().gen_range(0..items_of_type.len());
        items_of_type[idx].clone()
    }

    pub fn pity_roll(&mut self, limited: ItemType, standard: ItemType, weapon: ItemType) -> Item {
        let mut rng = rand::thread_rng();

        // 50% chance to get a limited 5-star
        if rng.gen_range(0..=1) == 0 {
            if self.is_last_five_star_item_limited {
                self.is_last_five_star_item_limited = false;
                return self.random_item_of_type(standard);
            } else {
                self.is_last_five_star_item_limited = true;
                return self.random_item_of_type(limited);
            }
        }
        return self.random_item_of_type(weapon);
    }
}

impl Gacha {
    pub fn from_json(filename: &str) -> Vec<Item> {
        let contents = fs::read_to_string(filename).expect("Something went wrong while reading the file");
        let items: Vec<ItemFromJson> = serde_json::from_str(&contents).expect("JSON was not well-formatted");
        
        let items = items.into_iter().map(|item| Item {
            name: item.name,
            item_type: match item.item_type.as_str() {
                "FiveStarCharacterLimited" => ItemType::FiveStarCharacterLimited,
                "FiveStarCharacterStandard" => ItemType::FiveStarCharacterStandard,
                "FiveStarWeapon" => ItemType::FiveStarWeapon,
                "FourStarCharacter" => ItemType::FourStarCharacter,
                "FourStarWeapon" => ItemType::FourStarWeapon,
                "ThreeStarItem" => ItemType::ThreeStarItem,
                _ => panic!("Unrecognized item type."),
            },
            description: item.description,
            image_url: item.image_url
        }).collect();
        items
    }
}