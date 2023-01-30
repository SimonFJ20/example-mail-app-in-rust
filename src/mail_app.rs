use rand::Rng;

fn generate_id() -> i32 {
    rand::thread_rng().gen()
}

pub type SessionId = i32;
pub struct Session {
    id: SessionId,
    user_id: i32,
}

impl Session {
    fn new(user_id: i32) -> Self {
        Self {
            id: generate_id(),
            user_id,
        }
    }
}

pub type UserId = i32;
pub struct User {
    id: UserId,
    username: String,
    password: String,
}

impl User {
    fn new(username: String, password: String) -> Self {
        Self {
            id: generate_id(),
            username,
            password,
        }
    }
}

pub type MailId = i32;
pub struct Mail {
    id: MailId,
    sender: UserId,
    reciever: UserId,
    subject: String,
    content: String,
    is_read: bool,
}

impl Mail {
    pub fn new(sender: UserId, reciever: UserId, subject: String, content: String) -> Self {
        Self {
            id: generate_id(),
            sender,
            reciever,
            subject,
            content,
            is_read: false,
        }
    }
}

pub struct MailApp {
    users: Vec<User>,
    sessions: Vec<Session>,
    mails: Vec<Mail>,
}

pub enum LoginError {
    UserDoesntExist,
    WrongPassword,
}

pub enum RegisterError {
    UserAlreadyExists,
}

pub struct MailInfo {
    pub id: MailId,
    pub sender: String,
    pub subject: String,
}

impl MailApp {
    pub fn new() -> Self {
        Self {
            users: vec![
                User::new("user1".to_string(), "1234".to_string()),
                User::new("user2".to_string(), "1234".to_string()),
            ],
            sessions: Vec::new(),
            mails: Vec::new(),
        }
    }

    pub fn login(&mut self, username: String, password: String) -> Result<SessionId, LoginError> {
        let maybe_user = self.users.iter().find(|user| user.username == username);
        match maybe_user {
            Some(user) => {
                if user.password == password {
                    let session = Session::new(user.id);
                    let session_id = session.id;
                    self.sessions.push(session);
                    Ok(session_id)
                } else {
                    Err(LoginError::WrongPassword)
                }
            }
            None => Err(LoginError::UserDoesntExist),
        }
    }

    pub fn register(&mut self, username: String, password: String) -> Result<(), RegisterError> {
        let maybe_user = self.users.iter().find(|user| user.username == username);
        match maybe_user {
            Some(_) => Err(RegisterError::UserAlreadyExists),
            None => {
                let user = User::new(username, password);
                self.users.push(user);
                Ok(())
            }
        }
    }

    pub fn logout(&mut self, session_id: SessionId) {
        match self
            .sessions
            .iter()
            .position(|session| session.id == session_id)
        {
            Some(index) => {
                self.sessions.swap_remove(index);
            }
            None => todo!(),
        }
    }

    pub fn list_mails(&self, session_id: SessionId) -> Vec<MailId> {
        let user_id = self
            .user_id_by_session_id(session_id)
            .expect("user not found");
        self.mails
            .iter()
            .filter(|mail| mail.reciever == user_id)
            .map(|mail| mail.id)
            .collect()
    }

    pub fn list_unread_mails(&self, session_id: SessionId) -> Vec<MailId> {
        let user_id = self
            .user_id_by_session_id(session_id)
            .expect("user not found");
        self.mails
            .iter()
            .filter(|mail| mail.reciever == user_id)
            .filter(|mail| !mail.is_read)
            .map(|mail| mail.id)
            .collect()
    }

    pub fn mail_info(&self, mail_id: MailId) -> Option<MailInfo> {
        self.mails
            .iter()
            .find(|mail| mail.id == mail_id)
            .map(|mail| MailInfo {
                id: mail.id,
                sender: self
                    .username_by_user_id(mail.sender)
                    .expect("user not found"),
                subject: mail.subject.clone(),
            })
    }

    pub fn read_mail(&mut self, mail_id: MailId) -> Option<String> {
        match self.mails.iter_mut().find(|mail| mail.id == mail_id) {
            Some(mail) => {
                mail.is_read = true;
                Some(mail.content.clone())
            }
            None => None,
        }
    }

    pub fn write_mail(
        &mut self,
        session_id: SessionId,
        reciever: String,
        subject: String,
        content: String,
    ) {
        let sender = self
            .user_id_by_session_id(session_id)
            .expect("user not found");
        if let Some(reciever) = self.users.iter().find(|user| user.username == reciever) {
            self.mails
                .push(Mail::new(sender, reciever.id, subject, content))
        }
    }

    fn user_id_by_session_id(&self, session_id: SessionId) -> Option<UserId> {
        self.sessions
            .iter()
            .find(|s| s.id == session_id)
            .map(|s| s.user_id)
    }

    fn username_by_user_id(&self, user_id: UserId) -> Option<String> {
        self.users
            .iter()
            .find(|user| user.id == user_id)
            .map(|user| user.username.clone())
    }
}
