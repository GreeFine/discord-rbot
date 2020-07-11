pub fn add_eat_cmd() {
    let mut db_instance = database::INSTANCE.write().unwrap();
    let author_id = *message.author.id.as_u64() as i64;
    if !db_instance.users.iter().any(|e| e.discordid == author_id) {
        db_instance.user_add(author_id, &*database::Role::Guest.to_string());
    }
    db_instance.message_add(
        *message.id.as_u64() as i64,
        author_id,
        &message.content,
        *message.channel_id.as_u64() as i64,
    );
}
