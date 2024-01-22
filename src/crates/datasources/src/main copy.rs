use async_graphql::{
    dynamic::{indexmap::IndexMap, Field, FieldFuture, FieldValue, InputValue, Object, Schema, SchemaError, TypeRef},
    Executor, Json, Name, UploadValue, Value, Variables,
};
use futures::lock::Mutex;
use slab::Slab;
use tokio::runtime::Builder;

pub type Storage = Mutex<Slab<FileInfo>>;

#[derive(Clone)]
pub struct FileInfo {
    pub id: String,
    url: String,
}

pub fn schema() -> Result<Schema, SchemaError> {
    let file_info = Object::new("FileInfo")
        .field(Field::new("id", TypeRef::named_nn(TypeRef::ID), |ctx| {
            FieldFuture::new(async {
                let file_info = ctx.parent_value.try_downcast_ref::<FileInfo>()?;
                Ok(Some(Value::from(&file_info.id)))
            })
        }))
        .field(Field::new("url", TypeRef::named_nn(TypeRef::STRING), |ctx| {
            FieldFuture::new(async {
                let file_info = ctx.parent_value.try_downcast_ref::<FileInfo>()?;
                Ok(Some(Value::from(&file_info.url)))
            })
        }));

    let query = Object::new("Query").field(Field::new(
        "uploads",
        TypeRef::named_nn_list_nn(file_info.type_name()),
        |ctx| {
            FieldFuture::new(async move {
                let storage = ctx.data_unchecked::<Storage>().lock().await;
                Ok(Some(FieldValue::list(
                    storage.iter().map(|(_, file)| FieldValue::owned_any(file.clone())),
                )))
            })
        },
    ));

    let mutation = Object::new("Mutation")
        .field(
            Field::new("singleUpload", TypeRef::named_nn(file_info.type_name()), |ctx| {
                FieldFuture::new(async move {
                    let mut storage = ctx.data_unchecked::<Storage>().lock().await;
                    let file = ctx.args.try_get("file")?.upload()?;
                    let entry = storage.vacant_entry();

                    let info = FileInfo {
                        id: entry.key().to_string(),
                        url: "".to_owned(),
                    };
                    entry.insert(info.clone());
                    Ok(Some(FieldValue::owned_any(info)))
                })
            })
            .argument(InputValue::new("file", TypeRef::named_nn(TypeRef::UPLOAD))),
        )
        .field(
            Field::new(
                "multipleUpload",
                TypeRef::named_nn_list_nn(file_info.type_name()),
                |ctx| {
                    FieldFuture::new(async move {
                        let mut infos = Vec::new();
                        let mut storage = ctx.data_unchecked::<Storage>().lock().await;
                        for item in ctx.args.try_get("files")?.list()?.iter() {
                            let file = item.upload()?;
                            let entry = storage.vacant_entry();
                            let upload = file.value(&ctx).unwrap();
                            let info = FileInfo {
                                id: entry.key().to_string(),
                                url: upload.filename.clone(),
                            };
                            entry.insert(info.clone());
                            infos.push(FieldValue::owned_any(info))
                        }
                        Ok(Some(infos))
                    })
                },
            )
            .argument(InputValue::new("files", TypeRef::named_nn_list_nn(TypeRef::UPLOAD))),
        );

    Schema::build(query.type_name(), Some(mutation.type_name()), None)
        .enable_uploading()
        .register(file_info)
        .register(query)
        .register(mutation)
        .data(Storage::default())
        .finish()
}

use std::{error::Error, fs::File};

fn main() {
    // create the schema
    let schema = schema().unwrap();
    println!("sdl: {}", schema.sdl());

    let rt2 = Builder::new_current_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let mut map = IndexMap::new();
    map.insert(Name::new("file"), Value::Null);

    let mut req = async_graphql::Request::new("mutation($file: Upload!) { singleUpload(file: $file) { __typename } }")
        .variables(Variables::from_value(Value::Object(map)));

    req.set_upload(
        "variables.file",
        UploadValue {
            filename: "foo.txt".to_owned(),
            content_type: None,
            content: File::create("foo.txt").unwrap(),
        },
    );

    let _ = rt2.block_on(async move {
        let resp1 = schema.execute(req).await;
        println!("resp1: {}, errors: {:?}", resp1.data, resp1.errors);

        let resp2 = schema.execute("{ uploads { id url } }").await;
        println!("resp2: {}", resp2.data);
    });
}
