use crate::database::dao;
use crate::database::dao::last_update::UpdatableElement;
use crate::fileo::credentials::FileoCredentials;
use crate::tools::log_error_and_return;
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dto::uda_instance::InstancesList;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::{Request, State};
use rocket_dyn_templates::{Template, context};
use std::sync::Mutex;

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
    pool: &State<Pool<ConnectionManager<SqliteConnection>>>,
    _credentials: FileoCredentials,
) -> Result<Template, Status> {
    let last_update = retrieve_last_update(pool)?;
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
    pool: &State<Pool<ConnectionManager<SqliteConnection>>>,
    _credentials: FileoCredentials,
) -> Result<Template, Status> {
    let mut connection = pool
        .get()
        .map_err(log_error_and_return(Status::InternalServerError))?;
    let memberships = dao::membership::retrieve_memberships(&mut connection)
        .map_err(log_error_and_return(Status::InternalServerError))?;

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

#[get("/memberships/lookup")]
pub async fn look_membership_up(_credentials: FileoCredentials) -> Template {
    Template::render(
        "member/lookup-member",
        context! {
            title: "Recherche d'adhésion",
        },
    )
}

#[get("/memberships/lookup", rank = 2)]
pub async fn look_membership_up_unauthenticated() -> Redirect {
    Redirect::to(uri!("/fileo/login/?page=/memberships/lookup"))
}

#[get("/csv/check")]
pub async fn check_members_from_csv(
    pool: &State<Pool<ConnectionManager<SqliteConnection>>>,
    _credentials: FileoCredentials,
) -> Result<Template, Status> {
    let last_update = retrieve_last_update(pool)?;
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

fn retrieve_last_update(
    pool: &State<Pool<ConnectionManager<SqliteConnection>>>,
) -> Result<String, Status> {
    let mut connection = pool
        .get()
        .map_err(log_error_and_return(Status::InternalServerError))?;
    let last_update =
        dao::last_update::get_last_update(&mut connection, &UpdatableElement::Memberships)
            .map_err(log_error_and_return(Status::InternalServerError))?;
    let last_update = match last_update {
        None => "Jamais".to_owned(),
        Some(last_update) => last_update.format("%d/%m/%Y").to_string(),
    };
    Ok(last_update)
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
        use crate::database::with_temp_database;
        use crate::fileo::authentication::AUTHENTICATION_COOKIE;
        use crate::fileo::credentials::FileoCredentials;
        use crate::web::credentials_storage::CredentialsStorage;
        use crate::web::frontend::frontend_controller::{
            list_memberships, list_memberships_unauthenticated,
        };
        use diesel::SqliteConnection;
        use diesel::r2d2::{ConnectionManager, Pool};
        use rocket::http::{Cookie, Status};
        use rocket::local::asynchronous::Client;
        use rocket::tokio::runtime::Runtime;
        use rocket_dyn_templates::Template;
        use std::sync::Mutex;

        #[test]
        fn should_render_membership_list() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let credentials =
                    FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
                let mut credentials_storage = CredentialsStorage::default();
                let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
                credentials_storage.store(uuid.clone(), credentials);
                let credentials_storage_mutex = Mutex::new(credentials_storage);

                let rocket = rocket::build()
                    .mount(
                        "/",
                        routes![list_memberships, list_memberships_unauthenticated],
                    )
                    .manage(pool)
                    .manage(credentials_storage_mutex)
                    .attach(Template::fairing());

                let client = Client::tracked(rocket).await.unwrap();
                let cookie = Cookie::new(AUTHENTICATION_COOKIE, uuid);

                let request = client.get("/memberships").cookie(cookie.clone());

                let response = request.dispatch().await;
                assert_eq!(Status::Ok, response.status());
            }
            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }

        #[test]
        fn should_not_render_membership_list_when_unauthenticated() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let rocket = rocket::build()
                    .mount(
                        "/",
                        routes![list_memberships, list_memberships_unauthenticated],
                    )
                    .manage(pool)
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

            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }
    }

    mod check_members_from_csv {
        use crate::database::{dao, with_temp_database};
        use crate::fileo::authentication::AUTHENTICATION_COOKIE;
        use crate::fileo::credentials::FileoCredentials;
        use crate::web::credentials_storage::CredentialsStorage;
        use crate::web::frontend::frontend_controller::{
            check_members_from_csv, check_members_from_csv_unauthenticated,
        };
        use diesel::SqliteConnection;
        use diesel::r2d2::{ConnectionManager, Pool};
        use rocket::http::{Cookie, Status};
        use rocket::local::asynchronous::Client;
        use rocket::tokio::runtime::Runtime;
        use rocket_dyn_templates::Template;
        use std::sync::Mutex;

        #[test]
        fn success() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let credentials =
                    FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
                let mut credentials_storage = CredentialsStorage::default();
                let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
                credentials_storage.store(uuid.clone(), credentials);
                let credentials_storage_mutex = Mutex::new(credentials_storage);

                let mut connection = pool.get().unwrap();
                dao::membership::replace_memberships(&mut connection, &[]).unwrap(); // Updating last update date

                let rocket = rocket::build()
                    .mount(
                        "/",
                        routes![
                            check_members_from_csv,
                            check_members_from_csv_unauthenticated
                        ],
                    )
                    .manage(pool)
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

            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }

        #[test]
        fn success_when_never_updated() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let credentials =
                    FileoCredentials::new("test_login".to_owned(), "test_password".to_owned());
                let mut credentials_storage = CredentialsStorage::default();
                let uuid = "0ea9a5fb-0f46-4057-902a-2552ed956bde".to_owned();
                credentials_storage.store(uuid.clone(), credentials);
                let credentials_storage_mutex = Mutex::new(credentials_storage);

                let rocket = rocket::build()
                    .mount(
                        "/",
                        routes![
                            check_members_from_csv,
                            check_members_from_csv_unauthenticated
                        ],
                    )
                    .manage(pool)
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

            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }

        #[test]
        fn fail_when_unauthenticated() {
            async fn test(pool: Pool<ConnectionManager<SqliteConnection>>) {
                let rocket = rocket::build()
                    .mount(
                        "/",
                        routes![
                            check_members_from_csv,
                            check_members_from_csv_unauthenticated
                        ],
                    )
                    .manage(pool)
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

            with_temp_database(|pool| Runtime::new().unwrap().block_on(test(pool)));
        }
    }
}
