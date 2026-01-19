pub mod inventory {
    use rusqlite::{Connection, Result};
    #[derive(Debug)]
    pub struct Inventory {
        db: Connection,
    }

    #[derive(Debug, PartialEq)]
    pub struct Item {
        pub id: i64,
        pub name: String,
        pub comment: Option<String>,
        pub location_id: Option<i64>,
    }

    #[derive(Debug, PartialEq)]
    pub struct Location {
        pub id: i64,
        pub name: String,
        pub comment: Option<String>,
    }
    impl Inventory {
        pub fn open_in_memory() -> Result<Self> {
            let db = Connection::open_in_memory()?;
            let inv = Inventory { db };
            inv.init()?;

            Ok(inv)
        }

        pub fn open_in_file(filename: &str) -> Result<Self> {
            let db = Connection::open(filename)?;
            let inv = Inventory { db };
            inv.init()?;
            Ok(inv)
        }

        fn init(&self) -> Result<()> {
            self.db.execute("PRAGMA foreign_keys = ON", ())?;
            self.db.execute(
                "CREATE TABLE IF NOT EXISTS locations (
                    id INTEGER PRIMARY KEY,
                    name TEXT,
                    comment TEXT)",
                (),
            )?;

            self.db.execute(
                "CREATE TABLE IF NOT EXISTS items (
                    id INTEGER PRIMARY KEY,
                    name TEXT,
                    comment TEXT,
                    location_id INTEGER REFERENCES locations(id) )",
                (),
            )?;
            Ok(())
        }

        pub fn get_all_items(&self) -> Result<Vec<Item>> {
            let mut stmt = self
                .db
                .prepare("SELECT id, name, comment, location_id FROM items")?;
            let itm_iter = stmt.query_map([], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    comment: row.get(2)?,
                    location_id: row.get(3)?,
                })
            })?;
            Ok(itm_iter.map(|itm| itm.unwrap()).collect())
        }

        pub fn get_all_locations(&self) -> Result<Vec<Location>> {
            let mut stmt = self.db.prepare("SELECT id, name, comment FROM locations")?;
            let loc_iter = stmt.query_map([], |row| {
                Ok(Location {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    comment: row.get(2)?,
                })
            })?;
            Ok(loc_iter.map(|loc| loc.unwrap()).collect())
        }

        pub fn add_item(&self, i: &Item) -> Result<()> {
            self.db.execute(
                "INSERT INTO items (id, name, comment, location_id) VALUES (?1, ?2, ?3, ?4)",
                (&i.id, &i.name.as_str(), &i.comment, &i.location_id),
            )?;
            Ok(())
        }

        pub fn add_location(&self, l: &Location) -> Result<()> {
            if l.comment.is_some() {
                self.db.execute(
                    "INSERT INTO locations (id, name, comment) VALUES (?1, ?2, ?3)",
                    (l.id, l.name.as_str(), l.comment.as_ref().unwrap().as_str()),
                )?;
            } else {
                self.db.execute(
                    "INSERT INTO locations (id, name) VALUES (?1, ?2)",
                    (l.id, l.name.as_str()),
                )?;
            }

            Ok(())
        }

        pub fn items_by_location_id(&self, location_id: i64) -> Result<Vec<Item>> {
            let mut stmt = self.db.prepare(
                "SELECT id, name, comment, location_id FROM items WHERE location_id = ?1",
            )?;
            let itm_iter = stmt.query_map([location_id], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    comment: row.get(2)?,
                    location_id: row.get(3)?,
                })
            })?;
            Ok(itm_iter.map(|itm| itm.unwrap()).collect())
        }

        pub fn search_locations(&self, search_term: &str) -> Result<Vec<Location>> {
            let mut stmt = self.db.prepare(
                "SELECT id, name, comment FROM locations WHERE name LIKE ?1 OR comment LIKE ?2",
            )?;
            let loc_iter = stmt.query_map(
                [format!("%{}%", search_term), format!("%{}%", search_term)],
                |row| {
                    Ok(Location {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        comment: row.get(2)?,
                    })
                },
            )?;
            Ok(loc_iter.map(|loc| loc.unwrap()).collect())
        }

        pub fn search_items(&self, search_term: &str) -> Result<Vec<Item>> {
            let mut stmt = self.db.prepare(
                "SELECT id, name, comment, location_id FROM items WHERE name LIKE ?1 OR comment LIKE ?2",
            )?;
            let itm_iter = stmt.query_map(
                [format!("%{}%", search_term), format!("%{}%", search_term)],
                |row| {
                    Ok(Item {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        comment: row.get(2)?,
                        location_id: row.get(3)?,
                    })
                },
            )?;
            Ok(itm_iter.map(|itm| itm.unwrap()).collect())
        }

        pub fn search_item_id(&self, id: i64) -> Option<Item> {
            let mut stmt = self
                .db
                .prepare("SELECT id, name, comment, location_id FROM items WHERE id = ?1")
                .ok()?;
            match stmt.query_row([id], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    comment: row.get(2)?,
                    location_id: row.get(3)?,
                })
            }) {
                Ok(item) => Some(item),
                Err(_) => None,
            }
        }

        pub fn search_location_id(&self, id: i64) -> Option<Location> {
            let mut stmt = self
                .db
                .prepare("SELECT id, name, comment FROM locations WHERE id = ?1")
                .ok()?;
            match stmt.query_row([id], |row| {
                Ok(Location {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    comment: row.get(2)?,
                })
            }) {
                Ok(location) => Some(location),
                Err(_) => None,
            }
        }

        pub fn item_exists(&self, id: i64) -> bool {
            self.db
                .query_row("SELECT 1 FROM items WHERE id = ?1", [id], |_| Ok(true))
                .is_ok()
        }

        pub fn location_exists(&self, id: i64) -> bool {
            self.db
                .query_row("SELECT 1 FROM locations WHERE id = ?1", [id], |_| Ok(true))
                .is_ok()
        }

        pub fn edit_item(&self, new_item: &Item) -> Result<()> {
            if !self.item_exists(new_item.id) {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            self.db.execute(
                "UPDATE items SET name = ?1, comment = ?2, location_id = ?3 WHERE id = ?4",
                (
                    &new_item.name,
                    &new_item.comment,
                    &new_item.location_id,
                    &new_item.id,
                ),
            )?;
            Ok(())
        }

        pub fn edit_location(&self, new_location: &Location) -> Result<()> {
            if !self.location_exists(new_location.id) {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            self.db.execute(
                "UPDATE locations SET name = ?1, comment=?2 WHERE id = ?3",
                (&new_location.name, &new_location.comment, &new_location.id),
            )?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::inventory::*;
    #[test]
    fn test_db_creation() {
        let _my_inv: Inventory = Inventory::open_in_memory().unwrap();
        assert!(true);
    }
    #[test]
    fn test_location_insertion() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: None,
        };
        let l2 = Location {
            id: 102,
            name: "location2".to_string(),
            comment: Some("with comment".to_string()),
        };
        assert!(my_inv.add_location(&l1).is_ok());
        assert!(my_inv.add_location(&l2).is_ok());
        let res = my_inv.get_all_locations();
        assert!(res.is_ok());
        let locs = res.unwrap();
        assert_eq!(locs.len(), 2);

        assert_eq!(locs[0], l1);
        assert_eq!(locs[1], l2);
    }

    #[test]
    fn test_item_insertion() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: Some("comment".to_string()),
        };
        let i1 = Item {
            id: 201,
            name: "item1".to_string(),
            comment: None,
            location_id: None,
        };
        let i2 = Item {
            id: 202,
            name: "item2".to_string(),
            comment: Some("with_comment".to_string()),
            location_id: Some(101),
        };
        assert!(my_inv.add_location(&l1).is_ok());
        assert!(my_inv.add_item(&i1).is_ok());
        assert!(my_inv.add_item(&i2).is_ok());

        let res = my_inv.get_all_items();
        assert!(res.is_ok());
        let itms = res.unwrap();
        assert_eq!(itms.len(), 2);
        assert_eq!(itms[0], i1);
        assert_eq!(itms[1], i2);
    }

    #[test]
    fn test_improper_location_insertion() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: None,
        };
        let l2 = Location {
            id: 101,
            name: "location2".to_string(),
            comment: Some("with comment".to_string()),
        };
        assert!(my_inv.add_location(&l1).is_ok());
        assert!(my_inv.add_location(&l2).is_err())
    }

    #[test]
    fn test_improper_item_insertion() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: None,
        };
        let i1 = Item {
            id: 201,
            name: "item1".to_string(),
            comment: None,
            location_id: Some(101),
        };
        let i2 = Item {
            id: 201,
            name: "item2".to_string(),
            comment: Some("with_comment".to_string()),
            location_id: None,
        };
        let i3 = Item {
            id: 203,
            name: "item3".to_string(),
            comment: None,
            location_id: Some(102),
        };
        assert!(my_inv.add_location(&l1).is_ok());
        assert!(my_inv.add_item(&i1).is_ok());
        assert!(my_inv.add_item(&i2).is_err());
        assert!(my_inv.add_item(&i3).is_err());
    }

    #[test]
    fn test_items_by_location() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: None,
        };
        let i1 = Item {
            id: 201,
            name: "item1".to_string(),
            comment: None,
            location_id: Some(101),
        };
        let i2 = Item {
            id: 202,
            name: "item2".to_string(),
            comment: Some("with_comment".to_string()),
            location_id: None,
        };
        let i3 = Item {
            id: 203,
            name: "item3".to_string(),
            comment: None,
            location_id: Some(101),
        };
        assert!(my_inv.add_location(&l1).is_ok());
        assert!(my_inv.add_item(&i1).is_ok());
        assert!(my_inv.add_item(&i2).is_ok());
        assert!(my_inv.add_item(&i3).is_ok());

        let res = my_inv.items_by_location_id(101);
        assert!(res.is_ok());
        let itms = res.unwrap();
        assert_eq!(itms.len(), 2);
        assert_eq!(itms[0], i1);
        assert_eq!(itms[1], i3);
    }

    #[test]
    fn test_search_locations() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: None,
        };
        let l2 = Location {
            id: 102,
            name: "Location2".to_string(),
            comment: Some("with comment".to_string()),
        };
        assert!(my_inv.add_location(&l1).is_ok());
        assert!(my_inv.add_location(&l2).is_ok());

        let res = my_inv.search_locations("LOCATION1");
        assert!(res.is_ok());
        let locs = res.unwrap();
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0], l1);

        let res = my_inv.search_locations("atio");
        assert!(res.is_ok());
        let locs = res.unwrap();
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0], l1);
        assert_eq!(locs[1], l2);

        let res = my_inv.search_locations("nonexistent");
        assert!(res.is_ok());
        let locs = res.unwrap();
        assert_eq!(locs.len(), 0);
    }

    #[test]
    fn test_search_items() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: None,
        };
        let i1 = Item {
            id: 201,
            name: "item1".to_string(),
            comment: None,
            location_id: Some(101),
        };
        let i2 = Item {
            id: 202,
            name: "Item2".to_string(),
            comment: Some("with_comment".to_string()),
            location_id: None,
        };
        let i3 = Item {
            id: 203,
            name: "item3".to_string(),
            comment: None,
            location_id: Some(101),
        };
        assert!(my_inv.add_location(&l1).is_ok());
        assert!(my_inv.add_item(&i1).is_ok());
        assert!(my_inv.add_item(&i2).is_ok());
        assert!(my_inv.add_item(&i3).is_ok());

        let res = my_inv.search_items("ITEM1");
        assert!(res.is_ok());
        let itms = res.unwrap();
        assert_eq!(itms.len(), 1);
        assert_eq!(itms[0], i1);

        let res = my_inv.search_items("em");
        assert!(res.is_ok());
        let itms = res.unwrap();
        assert_eq!(itms.len(), 3);
        assert_eq!(itms[0], i1);
        assert_eq!(itms[1], i2);
        assert_eq!(itms[2], i3);

        let res = my_inv.search_items("nonexistent");
        assert!(res.is_ok());
        let itms = res.unwrap();
        assert_eq!(itms.len(), 0);
    }

    #[test]
    fn test_item_exists() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: None,
        };
        let i1 = Item {
            id: 201,
            name: "item1".to_string(),
            comment: None,
            location_id: Some(101),
        };
        assert!(my_inv.add_location(&l1).is_ok());
        assert!(my_inv.add_item(&i1).is_ok());

        assert!(my_inv.item_exists(201));
        assert!(!my_inv.item_exists(202));
    }

    #[test]
    fn test_location_exists() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: None,
        };
        assert!(my_inv.add_location(&l1).is_ok());

        assert!(my_inv.location_exists(101));
        assert!(!my_inv.location_exists(102));
    }

    #[test]
    fn test_edit_item() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: Some("comment".to_string()),
        };
        let mut i1 = Item {
            id: 201,
            name: "item1".to_string(),
            comment: None,
            location_id: Some(101),
        };
        let i2 = Item {
            id: 202,
            name: "item2".to_string(),
            comment: None,
            location_id: Some(101),
        };

        assert!(my_inv.add_location(&l1).is_ok());
        assert!(my_inv.add_item(&i1).is_ok());

        i1.name = "newname".to_string();
        i1.comment = Some("newComment".to_string());
        i1.location_id = None;

        assert!(my_inv.edit_item(&i1).is_ok());
        let updated_item = my_inv.search_item_id(201);
        assert!(updated_item.is_some());
        assert_eq!(i1, updated_item.unwrap());

        // Check for changing invalid locationID
        i1.location_id = Some(40);
        assert!(my_inv.edit_item(&i1).is_err());
        i1.location_id = None;
        let updated_item = my_inv.search_item_id(201);
        assert!(updated_item.is_some());
        assert_eq!(i1, updated_item.unwrap());

        // Check for editing item that does not exist
        assert!(my_inv.edit_item(&i2).is_err())
    }

    #[test]
    fn test_edit_location() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        let mut l1 = Location {
            id: 101,
            name: "location1".to_string(),
            comment: Some("comment".to_string()),
        };
        let l2 = Location {
            id: 102,
            name: "Location2".to_string(),
            comment: Some("with comment".to_string()),
        };

        assert!(my_inv.add_location(&l1).is_ok());

        l1.name = "newname".to_string();
        l1.comment = Some("newComment".to_string());

        assert!(my_inv.edit_location(&l1).is_ok());
        let updated_location = my_inv.search_location_id(101);
        assert!(updated_location.is_some());
        assert_eq!(l1, updated_location.unwrap());

        assert!(my_inv.edit_location(&l2).is_err());
    }
}
