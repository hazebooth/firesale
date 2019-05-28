extern crate libfiresale;
use clap::ArgMatches;
use libfiresale::api::{DatabaseContext, Document};

mod entrypoint;

// basic 1.0 support
// read document path

const GOOGLE_APPLICATION_CREDENTIALS_KEY: &'static str = "GOOGLE_APPLICATION_CREDENTIALS";
const PROJECT_ID_KEY: &'static str = "PROJECT_ID";

#[derive(Debug, Clone)]
struct Environment {
    pub service_account_path: Option<String>,
    pub project_id: Option<String>,
}

// Gathers environment variables before clap parsing to enforce requirements
fn gather_environment() -> Environment {
    use std::env;
    let service_account_path = env::var(GOOGLE_APPLICATION_CREDENTIALS_KEY).ok();
    let project_id = env::var(PROJECT_ID_KEY).ok();
    return Environment {
        service_account_path,
        project_id,
    };
}

// Used to represent root level applications options
#[derive(Debug)]
struct Options {
    environment: Environment, // cli-defined environment
}

// This represents a query for a certain document
pub struct DocumentQuery {
    collection_name: String,
    document_name: String,
}

// This represents a query to view an entire collection
pub struct CollectionQuery {
    collection_name: String,
}

// Numerous fronts for the entrypoint of a program after CLI parsing
enum EntryPoint {
    GetDocument(DocumentQuery),
    ViewCollection(CollectionQuery),
    DeleteDocument(DocumentQuery),
    Usage(String),
}

// root meta
const APP_NAME: &'static str = "firesale";
const APP_VERSION: &'static str = "0.1";
const APP_AUTHOR: &'static str = "Haze Booth <isnt@haze.cool>";
const ABOUT_APP: &'static str = "CLI Firestore Interface";

// application config
const CREDENTIALS_LOCATION_ARG: &'static str = "credentials";
const PROJECT_ID_ARG: &'static str = "project_id";

// subcommands
const GET_SUB_COMMAND: &'static str = "get";
const DELETE_SUB_COMMAND: &'static str = "delete";

const COLLECTION_NAME: &'static str = "collection";
const COLLECTION_NAME_SHORT: &'static str = "c";
const DOCUMENT_NAME: &'static str = "document";
const DOCUMENT_NAME_SHORT: &'static str = "d";

fn setup_arguments(environ: &Environment) -> (Options, EntryPoint) {
    use clap::{App, Arg, SubCommand};
    let matches = App::new(APP_NAME)
        .version(APP_VERSION)
        .author(APP_AUTHOR)
        .about(ABOUT_APP)
        .arg(Arg::with_name(PROJECT_ID_ARG).required(environ.project_id.is_none()))
        .arg(
            Arg::with_name(CREDENTIALS_LOCATION_ARG)
                .required(environ.service_account_path.is_none()),
        )
        .subcommand(
            SubCommand::with_name(GET_SUB_COMMAND)
                .arg(Arg::with_name(COLLECTION_NAME).required(true))
                .arg(Arg::with_name(DOCUMENT_NAME)),
        )
        .subcommand(
            SubCommand::with_name(DELETE_SUB_COMMAND)
                .arg(Arg::with_name(COLLECTION_NAME).required(true))
                .arg(Arg::with_name(DOCUMENT_NAME)),
        )
        .get_matches();
    let environment = {
        // TODO(hazebooth): investigate
        let service_account_path = matches.value_of(CREDENTIALS_LOCATION_ARG).map(String::from);
        let project_id = matches.value_of(PROJECT_ID_ARG).map(String::from);
        Environment {
            service_account_path,
            project_id,
        }
    };
    let options = Options { environment };
    if let Some(get_command) = &matches.subcommand_matches(GET_SUB_COMMAND) {
        if get_command.is_present(DOCUMENT_NAME) {
            let query = DocumentQuery::from_sub_matches(get_command);
            return (options, EntryPoint::GetDocument(query));
        } else {
            let query = CollectionQuery::from_sub_matches(get_command);
            return (options, EntryPoint::ViewCollection(query));
        }
    } else if let Some(delete_command) = &matches.subcommand_matches(DELETE_SUB_COMMAND) {
        let query = DocumentQuery::from_sub_matches(delete_command);
        return (options, EntryPoint::DeleteDocument(query));
    }
    return (options, EntryPoint::Usage(matches.usage().to_string()));
}

impl DocumentQuery {
    fn from_sub_matches(matches: &&ArgMatches) -> DocumentQuery {
        DocumentQuery {
            collection_name: matches.value_of(COLLECTION_NAME).unwrap().to_string(),
            document_name: matches.value_of(DOCUMENT_NAME).unwrap().to_string(),
        }
    }
}

impl CollectionQuery {
    fn from_sub_matches(matches: &&ArgMatches) -> CollectionQuery {
        CollectionQuery {
            collection_name: matches.value_of(COLLECTION_NAME).unwrap().to_string(),
        }
    }
}

fn main() -> Result<(), String> {
    let environment = gather_environment();
    let (options, entrypoint) = setup_arguments(&environment);
    // if the entrypoint is set, use that
    // if the entrypoint is not set, default to env
    let context = {
        if let (Some(service_account_path), Some(project_id)) = (
            options.environment.service_account_path,
            options.environment.project_id,
        ) {
            DatabaseContext::new(project_id, service_account_path)
        } else if let (Some(service_account_path), Some(project_id)) =
            (environment.service_account_path, environment.project_id)
        {
            DatabaseContext::new(project_id, service_account_path)
        } else {
            Err(String::from("Failed to create database context, not provided in environment variables or cli args"))
        }
    }?;
    match entrypoint {
        EntryPoint::GetDocument(query) => entrypoint::handle_document_get(query, context),
        EntryPoint::ViewCollection(query) => entrypoint::handle_document_view(query, context),
        EntryPoint::DeleteDocument(query) => entrypoint::handle_document_delete(query, context),
        _ => Ok(()),
    }
}
