use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: Option<i64>,
    pub name: String,
    pub class_name: String,
    pub race: String,
    pub level: i32,
    pub hp_current: i32,
    pub hp_max: i32,
    pub armor_class: i32,
    pub speed: i32,
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
    /// Slots for spell level 1..=9, index 0..=8
    pub spell_slots: Vec<i32>,
    /// Simple inventory list of item names
    pub inventory: Vec<String>,
    /// Names of proficient skills (e.g., "perception")
    pub skill_proficiencies: Vec<String>,
    pub notes: Option<String>,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            id: None,
            name: String::from("Unnamed"),
            class_name: String::from("Fighter"),
            race: String::from("Human"),
            level: 1,
            hp_current: 10,
            hp_max: 10,
            armor_class: 10,
            speed: 30,
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
            spell_slots: vec![0; 9],
            inventory: vec![],
            skill_proficiencies: vec![],
            notes: None,
        }
    }
}

impl Character {
    pub fn ability_mod(score: i32) -> i32 {
        (score - 10).div_euclid(2)
    }

    pub fn str_mod(&self) -> i32 { Self::ability_mod(self.strength) }
    pub fn dex_mod(&self) -> i32 { Self::ability_mod(self.dexterity) }
    pub fn con_mod(&self) -> i32 { Self::ability_mod(self.constitution) }
    pub fn int_mod(&self) -> i32 { Self::ability_mod(self.intelligence) }
    pub fn wis_mod(&self) -> i32 { Self::ability_mod(self.wisdom) }
    pub fn cha_mod(&self) -> i32 { Self::ability_mod(self.charisma) }

    pub fn proficiency_bonus(&self) -> i32 {
        // DnD5e: 2 at levels 1-4, 3 at 5-8, 4 at 9-12, 5 at 13-16, 6 at 17-20
        2 + ((self.level - 1) / 4)
    }

    pub fn level_up(&mut self) {
        self.level += 1;
    }

    pub fn change_hp(&mut self, delta: i32) {
        self.hp_current = (self.hp_current + delta).clamp(0, self.hp_max);
    }

    pub fn set_hp(&mut self, new_hp: i32) {
        self.hp_current = new_hp.clamp(0, self.hp_max);
    }

    pub fn add_item(&mut self, item: String) {
        if !item.trim().is_empty() {
            self.inventory.push(item);
        }
    }

    pub fn remove_item(&mut self, index: usize) {
        if index < self.inventory.len() {
            self.inventory.remove(index);
        }
    }

    pub fn adjust_spell_slot(&mut self, level: usize, delta: i32) {
        if (1..=9).contains(&level) {
            let idx = level - 1;
            let new_val = (self.spell_slots[idx] + delta).max(0);
            self.spell_slots[idx] = new_val;
        }
    }

    pub fn ability_modifier_by_name(&self, name: &str) -> i32 {
        match name.to_lowercase().as_str() {
            "str" | "strength" => Self::ability_mod(self.strength),
            "dex" | "dexterity" => Self::ability_mod(self.dexterity),
            "con" | "constitution" => Self::ability_mod(self.constitution),
            "int" | "intelligence" => Self::ability_mod(self.intelligence),
            "wis" | "wisdom" => Self::ability_mod(self.wisdom),
            "cha" | "charisma" => Self::ability_mod(self.charisma),
            _ => 0,
        }
    }

    pub fn skill_modifier(&self, skill: &str) -> i32 {
        let (ability, key) = skill_to_ability(skill);
        let base = self.ability_modifier_by_name(ability);
        let proficient = self
            .skill_proficiencies
            .iter()
            .any(|s| s.to_lowercase() == key);
        base + if proficient { self.proficiency_bonus() } else { 0 }
    }
}

pub fn all_skills() -> Vec<(&'static str, &'static str)> {
    vec![
        ("acrobatics", "dex"),
        ("animal handling", "wis"),
        ("arcana", "int"),
        ("athletics", "str"),
        ("deception", "cha"),
        ("history", "int"),
        ("insight", "wis"),
        ("intimidation", "cha"),
        ("investigation", "int"),
        ("medicine", "wis"),
        ("nature", "int"),
        ("perception", "wis"),
        ("performance", "cha"),
        ("persuasion", "cha"),
        ("religion", "int"),
        ("sleight of hand", "dex"),
        ("stealth", "dex"),
        ("survival", "wis"),
    ]
}

pub fn skill_to_ability(skill: &str) -> (&'static str, String) {
    let s = skill.to_lowercase();
    for (name, ability) in all_skills() {
        if s == name {
            return (ability, name.to_string());
        }
    }
    ("str", "".to_string())
}
