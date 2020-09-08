pub use super::models::*;
use super::Instance;
use diesel::prelude::*;
use std::error::Error;

pub enum ProjectIds {
  MessageId,
  ChannelId,
}

impl Instance {
  pub fn user_load(&mut self) {
    use super::schema::users::dsl::*;

    let results = users
      .load::<User>(&self.get_connection())
      .expect("Error loading users");

    self.users = results;
  }

  pub fn user_add<'a>(&mut self, discordid: i64, role: &'a str) {
    let new_user = NewUser { discordid, role };

    let newuser: User = diesel::insert_into(users::table)
      .values(&new_user)
      .get_result(&self.get_connection())
      .expect("Error saving new user");
    self.users.push(newuser);
  }

  pub fn user_search_mut(&mut self, discordid: u64) -> Option<&mut User> {
    for user in self.users.iter_mut() {
      if user.discordid == discordid as i64 {
        return Some(user);
      }
    }
    None
  }

  pub fn user_role_update(&mut self, discord_id: u64, new_role: Role) -> String {
    use super::schema::users::dsl::*;

    let conn = self.get_connection();

    let mut user: &mut User = match self.user_search_mut(discord_id) {
      Some(user) => user,
      None => return String::from("User not found"),
    };

    diesel::update(users.find(user.id))
      .set(role.eq(new_role.to_string()))
      .get_result::<User>(&conn)
      .expect("Diesel: Unable to save new role");
    user.role = new_role.to_string();

    format!("Updated {} to {}", user.discordid, user.role)
  }

  pub fn user_search(&self, discordid: u64) -> Option<&User> {
    for user in self.users.iter() {
      if user.discordid == discordid as i64 {
        return Some(user);
      }
    }
    None
  }

  pub fn message_load(&mut self) {
    use super::schema::messages::dsl::*;

    let results = messages
      .load::<Message>(&self.get_connection())
      .expect("Error loading messages");

    self.messages = results;
  }

  pub fn message_add(
    &mut self,
    id: i64,
    author: i64,
    content: &str,
    channel: i64,
    date: Option<std::time::SystemTime>,
  ) {
    let new_message = Message {
      id,
      author,
      content: content.to_string(),
      channel,
      date,
    };

    let new_message = diesel::insert_into(messages::table)
      .values(&new_message)
      .get_result(&self.get_connection())
      .expect("Error saving new user");
    self.messages.push(new_message);
  }

  pub fn airtable_row_add(
    &mut self,
    aid: &str,
    created_time: Option<std::time::SystemTime>,
    content: &str,
  ) -> bool {
    match self.airtable_row_search(aid.to_string()) {
      Some(_) => false,
      None => {
        let new_row = NewAirtableRow {
          aid: aid.to_string(),
          content: content.to_string(),
          created_time,
        };

        let new_row: AirtableRow = diesel::insert_into(airtable::table)
          .values(&new_row)
          .get_result(&self.get_connection())
          .expect("Error saving new airtable_row");
        self.airtable.push(new_row);
        true
      }
    }
  }

  pub fn airtable_row_search(&self, aid: String) -> Option<&AirtableRow> {
    for row in self.airtable.iter() {
      if row.aid == aid {
        return Some(row);
      }
    }
    None
  }

  pub fn airtable_load(&mut self) {
    use super::schema::airtable::dsl::*;

    let results = airtable
      .load::<AirtableRow>(&self.get_connection())
      .expect("Error loading airtable_rows");

    self.airtable = results;
  }

  pub fn project_add(&mut self, project: NewProject) {
    let new_project: Project = diesel::insert_into(projects::table)
      .values(&project)
      .get_result(&self.get_connection())
      .expect("Error saving new airtable_row");
    self.projects.push(new_project);
  }

  pub fn projects_load(&mut self) {
    use super::schema::projects::dsl::*;

    let results = projects
      .load::<Project>(&self.get_connection())
      .expect("Error loading airtable_rows");

    self.projects = results;
  }

  pub fn projects_search(&self, id: i64, typeid: ProjectIds) -> Option<(usize, &Project)> {
    for (index, project) in self.projects.iter().enumerate() {
      match typeid {
        ProjectIds::MessageId => {
          if project.message_id == id {
            return Some((index, project));
          }
        }
        ProjectIds::ChannelId => {
          if project.channel_id == id {
            return Some((index, project));
          }
        }
      }
    }
    None
  }

  pub fn projects_delete(
    &mut self,
    p_channel_id: u64,
  ) -> Result<(&str, Option<Project>), Box<dyn Error + Send + Sync>> {
    use super::schema::projects::dsl::*;

    if let Some((index, project)) = self.projects_search(p_channel_id as i64, ProjectIds::ChannelId)
    {
      diesel::delete(projects.filter(id.eq(project.id))).execute(&self.get_connection())?;
      let project = self.projects.remove(index);
      return Ok(("Done", Some(project)));
    }
    Ok(("Channel wasn't found", None))
  }

  pub fn invites_load(&mut self) {
    use super::schema::invites::dsl::*;

    let results = invites
      .load::<Invite>(&self.get_connection())
      .expect("Error loading airtable_rows");

    self.invites = results;
  }

  pub fn invite_search(&self, code: &str) -> Option<&Invite> {
    for invite in self.invites.iter() {
      if invite.code == code {
        return Some(invite);
      }
    }
    None
  }

  pub fn invite_update(
    &mut self,
    p_code: String,
    p_count: i32,
  ) -> Result<(i32, Invite), Box<dyn Error + Send + Sync>> {
    if let Some(invite) = self.invite_search(&p_code) {
      use super::schema::invites::dsl::*;
      diesel::update(invites.filter(id.eq(invite.id)))
        .set(used_count.eq(p_count))
        .execute(&self.get_connection())?;
      let inviteclone = invite.clone();
      return Ok((p_count - invite.used_count, inviteclone));
    };
    let new_invite = NewInvite {
      code: p_code,
      used_count: p_count,
      actionchannel: None,
      actionrole: None,
    };
    let new_invite: Invite = diesel::insert_into(invites::table)
      .values(&new_invite)
      .get_result(&self.get_connection())
      .expect("Error saving new airtable_row");
    self.invites.push(new_invite.clone());
    Ok((p_count, new_invite))
  }
}
