use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Connection {
    id: usize,
    generation: usize,
}

impl Connection {
    pub(crate) fn new(id: usize, generation: usize) -> Self {
        Self {
            id,
            generation,
        }
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Connection({},{})", self.id, self.generation)
    }
}

pub struct ConnectionList {
    connections: Vec<Connection>,
    empty: Vec<usize>,
}

impl ConnectionList {
    pub fn new(size: usize) -> Self {

        let mut connections = Vec::with_capacity(size);
        let mut empty = Vec::with_capacity(size);
        for i in 0..size {
            connections.push(Connection::new(i, 0));
            empty.push(i);
        }

        Self {
            connections,
            empty,
        }
    }

    pub fn is_alive(&self, connection: Connection) -> bool {
        let id = connection.id;

        if self.connections[id].generation != connection.generation {
            return false;
        }

        connection.generation % 2 == 1
    }

    pub fn create_connection(&mut self) -> Option<Connection> {

        let id = self.empty.pop()?;

        let old_connection = self.connections[id];
        let new_connection = Connection::new(id, old_connection.generation + 1);

        self.connections[id] = new_connection;

        Some(new_connection)
    }

    pub fn delete_connection(&mut self, connection: Connection) -> Result<(), ()> {
        let id = connection.id;

        let old_connection = self.connections[id];

        if connection.generation != old_connection.generation {
            return Err(())
        }

        let new_connection = Connection::new(id, old_connection.generation + 1);
        self.connections[id] = new_connection;
        self.empty.push(id);

        Ok(())
    }
}

pub struct ConnectionIterator<'a> {
    list: &'a ConnectionList,
    index: usize,
}

impl<'a> Iterator for ConnectionIterator<'a> {
    type Item = Connection;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.list.connections.len() {
            let connection = self.list.connections[self.index];
            self.index += 1;
            
            if self.list.is_alive(connection) {
                return Some(connection);
            }
        }

        None
    }
}

impl<'a> IntoIterator for &'a ConnectionList {
    type Item = Connection;
    type IntoIter = ConnectionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ConnectionIterator {
            list: self,
            index: 0,
        }
    }
}

pub struct ConnectionDataList<T> {
    items: Vec<Option<T>>,
    generations: Vec<usize>,
}

impl<T> ConnectionDataList<T> {
    pub fn new(size: usize) -> Self {

        let mut items = Vec::with_capacity(size);
        let mut generations = Vec::with_capacity(size);
        for _ in 0..size {
            items.push(None);
            generations.push(0);
        }

        Self {
            items,
            generations,
        }
    }

    pub fn get(&self, connection: Connection) -> Option<&T> {
        let id = connection.id;
        if connection.generation != self.generations[id] {
            return None;
        }

        match self.items[id] {
            Some(ref item) => Some(item),
            None => None,
        }
    }

    pub fn set(&mut self, connection: Connection, item: T) -> Result<(), ()> {
        let id = connection.id;
        if connection.generation < self.generations[id] {
            return Err(())
        }

        self.generations[id] = connection.generation;
        self.items[id] = Some(item);

        Ok(())
    }
}
