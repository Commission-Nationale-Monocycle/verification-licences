use crate::member::members::Members;
use crate::member::memberships::Memberships;
use crate::tools::log_error_and_return;
use crate::web::api::members_state::MembersState;
use crate::web::credentials::FileoCredentials;
use dto::membership::Membership;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::{Request, State};
use rocket_dyn_templates::{Template, context};
use std::sync::Mutex;

#[get("/fileo/login")]
pub async fn fileo_login() -> Template {
    Template::render(
        "fileo-login",
        context! {
            title: "Connexion à Fileo"
        },
    )
}

#[get("/memberships")]
pub async fn list_memberships(
    members_state: &State<Mutex<MembersState>>,
    _credentials: FileoCredentials,
) -> Result<Template, Status> {
    let lock_result = members_state.lock();
    if let Err(error) = lock_result {
        log_error_and_return(Err(Status::InternalServerError))(error)
    } else {
        let members = lock_result.unwrap();
        let members: &Members = members.members();
        let memberships: Vec<&Membership> = members
            .values()
            .filter_map(Memberships::find_last_membership)
            .collect();

        Ok(Template::render(
            "memberships",
            context! {
                title: "Liste des licences",
                memberships: memberships
            },
        ))
    }
}

#[get("/memberships", rank = 2)]
pub async fn list_memberships_unauthenticated() -> Redirect {
    Redirect::to(uri!("/fileo/login/?page=/memberships"))
}

#[get("/check-memberships")]
pub async fn check_memberships(
    members_state: &State<Mutex<MembersState>>,
    _credentials: FileoCredentials,
) -> Result<Template, Status> {
    let lock_result = members_state.lock();
    if let Err(error) = lock_result {
        log_error_and_return(Err(Status::InternalServerError))(error)
    } else {
        let members_state = lock_result.unwrap();
        let file_details = members_state.file_details();
        let last_update = match file_details {
            None => "Jamais".to_owned(),
            Some(file_details) => file_details.update_date().format("%d/%m/%Y").to_string(),
        };
        Ok(Template::render(
            "check-memberships",
            context! {
                title: "Vérifier les licences",
                last_update: last_update
            },
        ))
    }
}

#[get("/check-memberships", rank = 2)]
pub async fn check_memberships_unauthenticated() -> Redirect {
    Redirect::to(uri!("/fileo/login/?page=/check-memberships"))
}

#[catch(404)]
pub async fn not_found(req: &Request<'_>) -> Template {
    Template::render(
        "error/404",
        context! {
            uri: req.uri()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::member::file_details::FileDetails;
    use crate::web::authentication::FILEO_AUTHENTICATION_COOKIE;
    use crate::web::credentials::CredentialsStorage;
    use chrono::Utc;
    use rocket::http::Cookie;
    use rocket::local::asynchronous::Client;
    use std::ffi::OsString;

    // region fileo_login
    #[async_test]
    async fn should_render_fileo_login() {
        let rocket = rocket::build()
            .mount("/", routes![fileo_login])
            .attach(Template::fairing());

        let client = Client::tracked(rocket).await.unwrap();
        let request = client.get("/fileo/login");

        let response = request.dispatch().await;
        assert_eq!(Status::Ok, response.status());
    }
    // endregion

    // region list_memberships
    #[async_test]
    async fn should_render_membership_list() {
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
        let mut credentials_storage = CredentialsStorage::default();
        let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
        credentials_storage.store_fileo(uuid.clone(), credentials);
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let members_sate_mutex = Mutex::new(MembersState::new(None, Members::default()));

        let rocket = rocket::build()
            .mount(
                "/",
                routes![list_memberships, list_memberships_unauthenticated],
            )
            .manage(members_sate_mutex)
            .manage(credentials_storage_mutex)
            .attach(Template::fairing());

        let client = Client::tracked(rocket).await.unwrap();
        let cookie = Cookie::new(FILEO_AUTHENTICATION_COOKIE, uuid);

        let request = client.get("/memberships").cookie(cookie.clone());

        let response = request.dispatch().await;
        assert_eq!(Status::Ok, response.status());
    }

    #[async_test]
    async fn should_not_render_membership_list_when_unauthenticated() {
        let members_sate_mutex = Mutex::new(MembersState::new(None, Members::default()));

        let rocket = rocket::build()
            .mount(
                "/",
                routes![list_memberships, list_memberships_unauthenticated],
            )
            .manage(members_sate_mutex)
            .attach(Template::fairing());

        let client = Client::tracked(rocket).await.unwrap();
        let request = client.get("/memberships");

        let response = request.dispatch().await;
        assert_eq!(Status::SeeOther, response.status());
        assert_eq!(
            "/fileo/login?page=/memberships",
            response.headers().get_one("location").unwrap()
        );
    }
    // endregion

    // region check_memberships
    #[async_test]
    async fn should_render_check_memberships() {
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
        let mut credentials_storage = CredentialsStorage::default();
        let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
        credentials_storage.store_fileo(uuid.clone(), credentials);
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let members_sate_mutex = Mutex::new(MembersState::new(
            Some(FileDetails::new(
                Utc::now().date_naive(),
                OsString::from(""),
            )),
            Members::default(),
        ));

        let rocket = rocket::build()
            .mount(
                "/",
                routes![check_memberships, check_memberships_unauthenticated],
            )
            .manage(members_sate_mutex)
            .manage(credentials_storage_mutex)
            .attach(Template::fairing());

        let client = Client::tracked(rocket).await.unwrap();
        let cookie = Cookie::new(FILEO_AUTHENTICATION_COOKIE, uuid);

        let request = client.get("/check-memberships").cookie(cookie.clone());

        let response = request.dispatch().await;
        assert_eq!(Status::Ok, response.status());
        let body = response.into_string().await.unwrap();
        assert!(!body.contains("Jamais"));
    }

    #[async_test]
    async fn should_render_check_memberships_when_no_file() {
        let credentials =
            FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
        let mut credentials_storage = CredentialsStorage::default();
        let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
        credentials_storage.store_fileo(uuid.clone(), credentials);
        let credentials_storage_mutex = Mutex::new(credentials_storage);

        let members_sate_mutex = Mutex::new(MembersState::new(None, Members::default()));

        let rocket = rocket::build()
            .mount(
                "/",
                routes![check_memberships, check_memberships_unauthenticated],
            )
            .manage(members_sate_mutex)
            .manage(credentials_storage_mutex)
            .attach(Template::fairing());

        let client = Client::tracked(rocket).await.unwrap();
        let cookie = Cookie::new(FILEO_AUTHENTICATION_COOKIE, uuid);

        let request = client.get("/check-memberships").cookie(cookie.clone());

        let response = request.dispatch().await;
        assert_eq!(Status::Ok, response.status());
        let body = response.into_string().await.unwrap();
        assert!(body.contains("Jamais"));
    }

    #[async_test]
    async fn should_not_render_check_memberships_when_unauthenticated() {
        let members_sate_mutex = Mutex::new(MembersState::new(None, Members::default()));

        let rocket = rocket::build()
            .mount(
                "/",
                routes![check_memberships, check_memberships_unauthenticated],
            )
            .manage(members_sate_mutex)
            .attach(Template::fairing());

        let client = Client::tracked(rocket).await.unwrap();
        let request = client.get("/check-memberships");

        let response = request.dispatch().await;
        assert_eq!(Status::SeeOther, response.status());
        assert_eq!(
            "/fileo/login?page=/check-memberships",
            response.headers().get_one("location").unwrap()
        );
    }
    // endregion
}
