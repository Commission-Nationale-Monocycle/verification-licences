use crate::fileo::credentials::FileoCredentials;
use crate::membership::grouped_memberships::GroupedMemberships;
use crate::membership::memberships::Memberships;
use crate::tools::log_error_and_return;
use crate::web::api::memberships_state::MembershipsState;
use dto::membership::Membership;
use dto::uda_instance::InstancesList;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::{Request, State};
use rocket_dyn_templates::{Template, context};
use std::sync::{Mutex, MutexGuard};

#[get("/")]
pub async fn index() -> Template {
    Template::render(
        "index",
        context! {
            title: "Index"
        },
    )
}

#[get("/fileo/login")]
pub async fn fileo_login() -> Template {
    Template::render(
        "fileo/fileo-login",
        context! {
            title: "Connexion à Fileo"
        },
    )
}

#[get("/memberships/update")]
pub async fn update_memberships(
    memberships_state: &State<Mutex<MembershipsState>>,
    _credentials: FileoCredentials,
) -> Result<Template, Status> {
    let memberships = unwrap_memberships_state(memberships_state)?;
    let file_details = memberships.file_details();
    let last_update = match file_details {
        None => "Jamais".to_owned(),
        Some(file_details) => file_details.update_date().format("%d/%m/%Y").to_string(),
    };
    Ok(Template::render(
        "member/update-memberships",
        context! {
            title: "Mise à jour de la liste des licences",
            last_update: last_update
        },
    ))
}

#[get("/memberships/update", rank = 2)]
pub async fn update_memberships_unauthenticated() -> Redirect {
    Redirect::to(uri!("/fileo/login/?page=/memberships/update"))
}

#[get("/memberships")]
pub async fn list_memberships(
    memberships_state: &State<Mutex<MembershipsState>>,
    _credentials: FileoCredentials,
) -> Result<Template, Status> {
    let memberships = unwrap_memberships_state(memberships_state)?;
    let memberships: &GroupedMemberships = memberships.memberships();
    let memberships: Vec<&Membership> = memberships
        .values()
        .filter_map(Memberships::find_last_membership)
        .collect();

    Ok(Template::render(
        "fileo/memberships",
        context! {
            title: "Liste des licences",
            memberships: memberships
        },
    ))
}

#[get("/memberships", rank = 2)]
pub async fn list_memberships_unauthenticated() -> Redirect {
    Redirect::to(uri!("/fileo/login/?page=/memberships"))
}

#[get("/csv/check")]
pub async fn check_members_from_csv(
    memberships_state: &State<Mutex<MembershipsState>>,
    _credentials: FileoCredentials,
) -> Result<Template, Status> {
    let memberships = unwrap_memberships_state(memberships_state)?;
    let file_details = memberships.file_details();
    let last_update = match file_details {
        None => "Jamais".to_owned(),
        Some(file_details) => file_details.update_date().format("%d/%m/%Y").to_string(),
    };
    Ok(Template::render(
        "fileo/check",
        context! {
            title: "Vérifier les licences depuis un fichier CSV",
            last_update: last_update
        },
    ))
}

#[get("/csv/check", rank = 2)]
pub async fn check_members_from_csv_unauthenticated() -> Redirect {
    Redirect::to(uri!("/fileo/login/?page=/csv/check"))
}

#[get("/uda/check")]
pub async fn check_members_from_uda(
    _credentials: FileoCredentials, // Fileo credentials are required for importing from UDA as well
    uda_instances_list: &State<Mutex<InstancesList>>,
) -> Result<Template, Status> {
    let instances_list = uda_instances_list
        .lock()
        .map_err(log_error_and_return(Status::InternalServerError))?;

    let last_update = match instances_list.update_date() {
        None => "Jamais".to_owned(),
        Some(update_date) => update_date.format("%d/%m/%Y").to_string(),
    };

    Ok(Template::render(
        "uda/check",
        context! {
            title: "Vérifier les licences depuis un import UDA",
            instances: instances_list.instances(),
            last_update: last_update
        },
    ))
}

#[get("/uda/check", rank = 2)]
pub async fn check_members_from_uda_unauthenticated() -> Redirect {
    Redirect::to(uri!("/fileo/login/?page=/uda/check"))
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

fn unwrap_memberships_state(
    memberships_state: &State<Mutex<MembershipsState>>,
) -> Result<MutexGuard<MembershipsState>, Status> {
    let lock_result = memberships_state.lock();
    if let Err(error) = lock_result {
        log_error_and_return(Err(Status::InternalServerError))(error)
    } else {
        let memberships_state = lock_result.unwrap();
        Ok(memberships_state)
    }
}

#[cfg(test)]
mod tests {
    mod fileo_login {
        use crate::web::frontend::frontend_controller::fileo_login;
        use rocket::http::Status;
        use rocket::local::asynchronous::Client;
        use rocket_dyn_templates::Template;

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
    }

    mod list_memberships {
        use crate::fileo::authentication::AUTHENTICATION_COOKIE;
        use crate::fileo::credentials::FileoCredentials;
        use crate::membership::grouped_memberships::GroupedMemberships;
        use crate::web::api::memberships_state::MembershipsState;
        use crate::web::credentials_storage::CredentialsStorage;
        use crate::web::frontend::frontend_controller::{
            list_memberships, list_memberships_unauthenticated,
        };
        use rocket::http::{Cookie, Status};
        use rocket::local::asynchronous::Client;
        use rocket_dyn_templates::Template;
        use std::sync::Mutex;

        #[async_test]
        async fn should_render_membership_list() {
            let credentials =
                FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
            let mut credentials_storage = CredentialsStorage::default();
            let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
            credentials_storage.store(uuid.clone(), credentials);
            let credentials_storage_mutex = Mutex::new(credentials_storage);

            let members_sate_mutex =
                Mutex::new(MembershipsState::new(None, GroupedMemberships::default()));

            let rocket = rocket::build()
                .mount(
                    "/",
                    routes![list_memberships, list_memberships_unauthenticated],
                )
                .manage(members_sate_mutex)
                .manage(credentials_storage_mutex)
                .attach(Template::fairing());

            let client = Client::tracked(rocket).await.unwrap();
            let cookie = Cookie::new(AUTHENTICATION_COOKIE, uuid);

            let request = client.get("/memberships").cookie(cookie.clone());

            let response = request.dispatch().await;
            assert_eq!(Status::Ok, response.status());
        }

        #[async_test]
        async fn should_not_render_membership_list_when_unauthenticated() {
            let members_sate_mutex =
                Mutex::new(MembershipsState::new(None, GroupedMemberships::default()));

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
    }

    mod check_members_from_csv {
        use crate::fileo::authentication::AUTHENTICATION_COOKIE;
        use crate::fileo::credentials::FileoCredentials;
        use crate::membership::file_details::FileDetails;
        use crate::membership::grouped_memberships::GroupedMemberships;
        use crate::web::api::memberships_state::MembershipsState;
        use crate::web::credentials_storage::CredentialsStorage;
        use crate::web::frontend::frontend_controller::{
            check_members_from_csv, check_members_from_csv_unauthenticated,
        };
        use chrono::Utc;
        use rocket::http::{Cookie, Status};
        use rocket::local::asynchronous::Client;
        use rocket_dyn_templates::Template;
        use std::ffi::OsString;
        use std::sync::Mutex;

        #[async_test]
        async fn success() {
            let credentials =
                FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
            let mut credentials_storage = CredentialsStorage::default();
            let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
            credentials_storage.store(uuid.clone(), credentials);
            let credentials_storage_mutex = Mutex::new(credentials_storage);

            let memberships_state = Mutex::new(MembershipsState::new(
                Some(FileDetails::new(
                    Utc::now().date_naive(),
                    OsString::from(""),
                )),
                GroupedMemberships::default(),
            ));

            let rocket = rocket::build()
                .mount(
                    "/",
                    routes![
                        check_members_from_csv,
                        check_members_from_csv_unauthenticated
                    ],
                )
                .manage(memberships_state)
                .manage(credentials_storage_mutex)
                .attach(Template::fairing());

            let client = Client::tracked(rocket).await.unwrap();
            let cookie = Cookie::new(AUTHENTICATION_COOKIE, uuid);

            let request = client.get("/csv/check").cookie(cookie.clone());

            let response = request.dispatch().await;
            assert_eq!(Status::Ok, response.status());
            let body = response.into_string().await.unwrap();
            assert!(!body.contains("Jamais"));
        }

        #[async_test]
        async fn success_when_no_file() {
            let credentials =
                FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
            let mut credentials_storage = CredentialsStorage::default();
            let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
            credentials_storage.store(uuid.clone(), credentials);
            let credentials_storage_mutex = Mutex::new(credentials_storage);

            let memberships_sate_mutex =
                Mutex::new(MembershipsState::new(None, GroupedMemberships::default()));

            let rocket = rocket::build()
                .mount(
                    "/",
                    routes![
                        check_members_from_csv,
                        check_members_from_csv_unauthenticated
                    ],
                )
                .manage(memberships_sate_mutex)
                .manage(credentials_storage_mutex)
                .attach(Template::fairing());

            let client = Client::tracked(rocket).await.unwrap();
            let cookie = Cookie::new(AUTHENTICATION_COOKIE, uuid);

            let request = client.get("/csv/check").cookie(cookie.clone());

            let response = request.dispatch().await;
            assert_eq!(Status::Ok, response.status());
            let body = response.into_string().await.unwrap();
            assert!(body.contains("Jamais"));
        }

        #[async_test]
        async fn fail_when_unauthenticated() {
            let members_sate_mutex =
                Mutex::new(MembershipsState::new(None, GroupedMemberships::default()));

            let rocket = rocket::build()
                .mount(
                    "/",
                    routes![
                        check_members_from_csv,
                        check_members_from_csv_unauthenticated
                    ],
                )
                .manage(members_sate_mutex)
                .attach(Template::fairing());

            let client = Client::tracked(rocket).await.unwrap();
            let request = client.get("/csv/check");

            let response = request.dispatch().await;
            assert_eq!(Status::SeeOther, response.status());
            assert_eq!(
                "/fileo/login?page=/csv/check",
                response.headers().get_one("location").unwrap()
            );
        }
    }
}
