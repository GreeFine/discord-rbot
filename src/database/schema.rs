table! {
    messages (id) {
        id -> Int8,
        author -> Int8,
        content -> Varchar,
        channel -> Int8,
    }
}

table! {
    users (id) {
        id -> Int4,
        discordid -> Int8,
        role -> Varchar,
    }
}

table! {
  eatbook (id) {
    id -> Int8,
    author -> Int8,
    description -> Varchar,
    address -> Varchar,
    tags -> Array<Varchar>,
  }
}

allow_tables_to_appear_in_same_query!(messages, users,);
