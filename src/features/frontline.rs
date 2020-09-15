use crate::core::commands::{CallBackParams, CallbackReturn};
// use crate::database;
// use chrono::DateTime;
use ftp::FtpStream;
use log::error;
use serenity::{http, model::id::ChannelId};
use std::collections::HashMap;
use std::env;
// use std::error::Error;
use crate::constants::discordids::ANNOYED_CHAN_HERDINGCHATTE;
use std::sync::{Arc, RwLock};
use std::{thread, time};

lazy_static! {
  pub static ref DIRECTORY_WATCH: RwLock<HashMap<String, usize>> = RwLock::new(HashMap::new());
}

fn ftp_connect() -> FtpStream {
  let host: String = env::var("FRONTLINE_FTP_HOST").expect("FRONTLINE_FTP_HOST isn't set");
  let user = env::var("FRONTLINE_FTP_USER").expect("FRONTLINE_FTP_USER isn't set");
  let password = env::var("FRONTLINE_FTP_PASSWORD").expect("FRONTLINE_FTP_PASSWORD isn't set");
  let mut ftp_stream = FtpStream::connect(host).unwrap();
  let _ = ftp_stream.login(&*user, &*password).unwrap();
  ftp_stream
}

pub fn add_dirrectory(params: CallBackParams) -> CallbackReturn {
  let dir_target = String::from(params.args[1]);
  let mut ftp_stream = ftp_connect();
  let root = ftp_stream.nlst(None).expect("Unable to list ftp dir");
  if !root.contains(&dir_target) {
    return Ok(Some(String::from(
      "I didn't find this directory in the ftp",
    )));
  }

  DIRECTORY_WATCH.write().unwrap().insert(dir_target, 0);
  Ok(Some(String::from("Done")))
}

pub fn check<F>(http: Arc<http::Http>, threads_check: F)
where
  F: for<'a> Fn(),
{
  match env::var("FRONTLINE_FTP_HOST") {
    Ok(token) => token,
    Err(_) => {
      error!("frontile host wasn't set, skiping feature");
      return;
    }
  };
  // {
  //   let mut db_instance = database::INSTANCE.write().unwrap();
  //   db_instance.airtable_load();
  // }

  // let client = reqwest::blocking::Client::new();
  loop {
    threads_check();
    {
      let mut ftp_stream = ftp_connect();
      let mut directories = DIRECTORY_WATCH.write().unwrap();

      for (directorie, count) in directories.iter_mut() {
        let path = format!("{}/Activity", directorie);
        let list = ftp_stream
          .nlst(Some(&*path))
          .expect("Unable to list ftp dir");
        println!("DEBUG {} {}", count, list.len());
        if *count < list.len() {
          *count = list.len();
        } else {
          println!("DEBUG {:#?}", list);
          ChannelId(ANNOYED_CHAN_HERDINGCHATTE)
            .say(
              &http,
              format!("<@344498090801364993> 😱 No update on {}", directorie),
            )
            .unwrap();
        }
      }
      ftp_stream.quit().unwrap();
    }
    thread::sleep(time::Duration::from_secs(60 * 20));
  }
}
