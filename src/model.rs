use crate::data::DataInfra;

pub(crate) struct Model {
    data: DataInfra,
}

impl Model {
    pub(crate) fn new(data: DataInfra) -> Self {
        Model { data }
    }

    pub(crate) fn get(&self, key: &str) -> Option<String> {
        self.data.find(key).ok().unwrap_or(None)
    }

    pub(crate) fn add(&mut self, key: String, value: String) -> Result<(), ()> {
        if key.len() >= 1000 || value.len() >= 4000 {
            return Err(());
        }

        self.delete_old_entries()?;

        self.data.delete(&key)?;
        self.data.insert(key, value)?;

        Ok(())
    }

    pub(crate) fn delete_old_entries(&self) -> Result<(), ()> {
        const THRESHOLD: usize = 1000;
        const RETAIN: usize = 100;;

        let count = self.data.count()?;
        if count < THRESHOLD {
            return Ok(());
        }

        self.data.delete_old_entries(RETAIN)?;
        Ok(())
    }
}
