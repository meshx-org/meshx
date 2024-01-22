use async_graphql::{
    dynamic::{indexmap::IndexMap, Field, FieldFuture, FieldValue, Object, Schema, SchemaError, TypeRef},
    Name, Value, Variables,
};
use futures::TryStreamExt;
use meshx_component::ServiceFs;
use midl::{ChildRequestStream, ParentRequest, ParentRequestStream};
use quaint::{
    ast::*,
    prelude::*,
    single::Quaint,
    visitor::{Sqlite, Visitor},
};
use tokio::runtime::Builder;
mod midl;

struct Context {
    pub conn: Quaint,
}

#[derive(Clone)]
pub struct FileInfo {
    pub id: String,
    url: String,
}

#[derive(Clone)]
pub struct Record {
    pub id: String,
}

#[derive(Debug, Clone)]
struct Constraints {
    not_null: Option<bool>,
}

#[derive(Debug, Clone)]
struct Index {
    columns: Vec<String>,
    name: String,
    unique: bool,
}

#[derive(Debug, Clone)]
struct TableColumn {
    name: String,
    r#type: String,
    constraints: Constraints,
}

#[derive(Debug, Clone)]
struct TableSchema {
    primary_key: Vec<String>,
    columns: Vec<TableColumn>,
    indexes: Vec<Index>,
}

#[derive(Debug, Clone)]
struct Table {
    datastore: String,
    name: String,
    schema: TableSchema,
}

fn generate_type(column: &TableColumn) -> TypeRef {
    TypeRef::named_nn(TypeRef::ID)
}

fn genarate_table_type(table: &Table) -> Object {
    let mut obj = Object::new(table.name.clone());

    for column in table.schema.columns.iter() {
        let column_name = column.name.clone();

        obj = obj.field(Field::new(column_name.clone(), generate_type(column), move |ctx| {
            let column_name = column_name.clone();

            FieldFuture::new(async move {
                let row = ctx.parent_value.try_downcast_ref::<ResultRow>()?;

                let val = row.get(&column_name).unwrap();

                Ok(Some(FieldValue::value(val.as_str().unwrap())))
            })
        }));
    }

    obj
}

fn genarate_find_many_query_field(table: Table) -> Field {
    let nt = table.clone();

    Field::new(
        format!("findMany{}", table.name),
        TypeRef::named_nn_list_nn(table.name),
        move |ctx| {
            let table_name = nt.name.clone();

            // let storage = ctx.data_unchecked::<Storage>().lock().await;
            let context = ctx.data_unchecked::<Context>();

            FieldFuture::new(async move {
                let mut selected_columns = vec![];

                for field in ctx.field().selection_set().into_iter() {
                    let name = field.name();

                    if name == "__typename" {
                        continue;
                    }

                    selected_columns.push(Column::new(field.name()));
                }

                let select_ast = Select::from_table(table_name).columns(selected_columns);
                let q = Query::from(select_ast);

                let result_set = context.conn.query(q.clone()).await.unwrap();
                let (sql_str, params) = Sqlite::build(q);

                // `query_raw` does not return column names in `ResultSet` when a call to a stored procedure is done
                let columns: Vec<String> = result_set.columns().iter().map(ToString::to_string).collect();
                // let mut result = Vec::new();
                let mut gql_result = Vec::new();

                // result
                for row in result_set.into_iter() {
                    gql_result.push(FieldValue::owned_any(row))
                }

                let args = ctx.args.as_index_map();
                let selections = ctx.field().selection_set().collect::<Vec<_>>();

                println!("run findManyUser, {:?} {} {:?}", columns, sql_str, params);

                Ok(Some(FieldValue::list(gql_result)))
            })
        },
    )
}

pub fn schema(conn: Quaint) -> Result<Schema, SchemaError> {
    let tables = vec![Table {
        datastore: String::from("postgres-dev"),
        name: String::from("User"),
        schema: TableSchema {
            primary_key: vec!["id".to_owned()],
            indexes: vec![Index {
                name: "idx_users_email".to_owned(),
                unique: true,
                columns: vec!["email".to_owned()],
            }],
            columns: vec![
                TableColumn {
                    name: "id".to_owned(),
                    r#type: "uuid".to_string(),
                    constraints: Constraints { not_null: Some(true) },
                },
                TableColumn {
                    name: "name".to_owned(),
                    r#type: "text".to_string(),
                    constraints: Constraints { not_null: Some(true) },
                },
                TableColumn {
                    name: "email".to_owned(),
                    r#type: "text".to_string(),
                    constraints: Constraints { not_null: Some(true) },
                },
            ],
        },
    }];

    /*let file_info = Object::new("FileInfo")
        .field(Field::new("id", TypeRef::named_nn(TypeRef::ID), |ctx| {
            FieldFuture::new(async {
                println!("run id");

                let file_info = ctx.parent_value.try_downcast_ref::<FileInfo>()?;
                Ok(Some(Value::from(&file_info.id)))
            })
        }))
        .field(Field::new("url", TypeRef::named_nn(TypeRef::STRING), |ctx| {
            FieldFuture::new(async {
                println!("run url");

                let file_info = ctx.parent_value.try_downcast_ref::<FileInfo>()?;
                Ok(Some(Value::from(&file_info.url)))
            })
        }));
    */

    let mut query = Object::new("Query");
    let mut schema = Schema::build(query.type_name(), None, None);

    let mut query_fields = vec![];

    for table in tables {
        let obj = genarate_table_type(&table);
        query_fields.push(genarate_find_many_query_field(table));
        schema = schema.register(obj);
    }

    for query_field in query_fields {
        query = query.field(query_field);
    }

    schema = schema.register(query);
    schema = schema.data(Context { conn });
    schema.finish()
}

//async fn handle_parent_request(mut stream: ParentRequestStream) {
//    while let Some(event) = stream.try_next().await.expect("failed to serve echo service") {
//        let ParentRequest::GetChild { responder } = event;
//        responder.send().expect("failed to send echo response");
//    }
//}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let mut service_fs = ServiceFs::new_local();

    // Serve the Echo protocol
    service_fs.dir("svc").add_fidl_service(IncomingRequest::Echo);
    
    service_fs
        .take_and_serve_directory_handle()
        .context("failed to serve outgoing namespace")?;

    let conn = Quaint::new("file:///tmp/example.db").await.unwrap();
    let result = conn.select(Select::default().value(1)).await.unwrap();

    // create the schema
    let schema = schema(conn).unwrap();
    println!("sdl: {}", schema.sdl());

    let mut map = IndexMap::new();
    map.insert(Name::new("file"), Value::Null);

    let req = async_graphql::Request::new("mutation($file: Upload!) { singleUpload(file: $file) { __typename } }")
        .variables(Variables::from_value(Value::Object(map)));

    let s = result.into_iter().nth(0).and_then(|row| row[0].as_i64());

    println!("result: {:?}", s);

    let resp2 = schema.execute("{ findManyUser { __typename id name email } }").await;
    println!("resp2: {:?}", resp2);

    Ok(())
}
