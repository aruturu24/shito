use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json;

use crate::models::Character;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open_or_create(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS characters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                class_name TEXT NOT NULL,
                race TEXT NOT NULL,
                level INTEGER NOT NULL,
                hp_current INTEGER NOT NULL,
                hp_max INTEGER NOT NULL,
                armor_class INTEGER NOT NULL,
                speed INTEGER NOT NULL,
                strength INTEGER NOT NULL,
                dexterity INTEGER NOT NULL,
                constitution INTEGER NOT NULL,
                intelligence INTEGER NOT NULL,
                wisdom INTEGER NOT NULL,
                charisma INTEGER NOT NULL,
                spell_slots TEXT NOT NULL,
                inventory TEXT NOT NULL,
                skill_proficiencies TEXT NOT NULL,
                notes TEXT
            );
            "#,
        )?;
        Ok(())
    }

    pub fn insert_character(&self, character: &mut Character) -> Result<i64> {
        let spell_slots = serde_json::to_string(&character.spell_slots)?;
        let inventory = serde_json::to_string(&character.inventory)?;
        self.conn.execute(
            r#"INSERT INTO characters
                (name, class_name, race, level, hp_current, hp_max, armor_class, speed,
                 strength, dexterity, constitution, intelligence, wisdom, charisma,
                 spell_slots, inventory, skill_proficiencies, notes)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
            "#,
            params![
                character.name,
                character.class_name,
                character.race,
                character.level,
                character.hp_current,
                character.hp_max,
                character.armor_class,
                character.speed,
                character.strength,
                character.dexterity,
                character.constitution,
                character.intelligence,
                character.wisdom,
                character.charisma,
                spell_slots,
                inventory,
                serde_json::to_string(&character.skill_proficiencies)?,
                character.notes,
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        character.id = Some(id);
        Ok(id)
    }

    pub fn update_character(&self, character: &Character) -> Result<()> {
        let id = character.id.expect("character must have id to update");
        let spell_slots = serde_json::to_string(&character.spell_slots)?;
        let inventory = serde_json::to_string(&character.inventory)?;
        self.conn.execute(
            r#"UPDATE characters SET
                name = ?1, class_name = ?2, race = ?3, level = ?4, hp_current = ?5,
                hp_max = ?6, armor_class = ?7, speed = ?8, strength = ?9, dexterity = ?10,
                constitution = ?11, intelligence = ?12, wisdom = ?13, charisma = ?14,
                spell_slots = ?15, inventory = ?16, skill_proficiencies = ?17, notes = ?18
               WHERE id = ?19
            "#,
            params![
                character.name,
                character.class_name,
                character.race,
                character.level,
                character.hp_current,
                character.hp_max,
                character.armor_class,
                character.speed,
                character.strength,
                character.dexterity,
                character.constitution,
                character.intelligence,
                character.wisdom,
                character.charisma,
                spell_slots,
                inventory,
                serde_json::to_string(&character.skill_proficiencies)?,
                character.notes,
                id
            ],
        )?;
        Ok(())
    }

    pub fn delete_character(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM characters WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_character(&self, id: i64) -> Result<Option<Character>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, name, class_name, race, level, hp_current, hp_max, armor_class, speed,
                      strength, dexterity, constitution, intelligence, wisdom, charisma,
                      spell_slots, inventory, skill_proficiencies, notes
                 FROM characters WHERE id = ?1"#,
        )?;
        let row = stmt
            .query_row(params![id], |row| {
                let spell_slots: String = row.get(15)?;
                let inventory: String = row.get(16)?;
                let skills: String = row.get(17)?;
                Ok(Character {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    class_name: row.get(2)?,
                    race: row.get(3)?,
                    level: row.get(4)?,
                    hp_current: row.get(5)?,
                    hp_max: row.get(6)?,
                    armor_class: row.get(7)?,
                    speed: row.get(8)?,
                    strength: row.get(9)?,
                    dexterity: row.get(10)?,
                    constitution: row.get(11)?,
                    intelligence: row.get(12)?,
                    wisdom: row.get(13)?,
                    charisma: row.get(14)?,
                    spell_slots: serde_json::from_str(&spell_slots).unwrap_or_else(|_| vec![0; 9]),
                    inventory: serde_json::from_str(&inventory).unwrap_or_default(),
                    skill_proficiencies: serde_json::from_str(&skills).unwrap_or_default(),
                    notes: row.get(18).ok(),
                })
            })
            .optional()?;
        Ok(row)
    }

    pub fn list_characters(&self) -> Result<Vec<Character>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, name, class_name, race, level, hp_current, hp_max, armor_class, speed,
                      strength, dexterity, constitution, intelligence, wisdom, charisma,
                      spell_slots, inventory, skill_proficiencies, notes
                 FROM characters ORDER BY name ASC"#,
        )?;

        let rows = stmt.query_map([], |row| {
            let spell_slots: String = row.get(15)?;
            let inventory: String = row.get(16)?;
            let skills: String = row.get(17)?;
            Ok(Character {
                id: row.get(0)?,
                name: row.get(1)?,
                class_name: row.get(2)?,
                race: row.get(3)?,
                level: row.get(4)?,
                hp_current: row.get(5)?,
                hp_max: row.get(6)?,
                armor_class: row.get(7)?,
                speed: row.get(8)?,
                strength: row.get(9)?,
                dexterity: row.get(10)?,
                constitution: row.get(11)?,
                intelligence: row.get(12)?,
                wisdom: row.get(13)?,
                charisma: row.get(14)?,
                spell_slots: serde_json::from_str(&spell_slots).unwrap_or_else(|_| vec![0; 9]),
                inventory: serde_json::from_str(&inventory).unwrap_or_default(),
                skill_proficiencies: serde_json::from_str(&skills).unwrap_or_default(),
                notes: row.get(18).ok(),
            })
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }
}
